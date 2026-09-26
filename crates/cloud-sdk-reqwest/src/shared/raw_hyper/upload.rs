use super::*;
use cloud_sdk::transport::{
    AsyncStreamSink, EndpointIdentity, LocalAsyncStreamSource, StreamFraming, StreamKind,
    StreamOutcome, StreamPartialState, StreamPolicy, StreamSinkMode, drive_local_stream,
};
#[cfg(feature = "async-rustls")]
use cloud_sdk::transport::{AsyncStreamSource, drive_async_stream};
use cloud_sdk_sanitization::SecretBuffer;
use core::{pin::Pin, task::Context};
use hyper::body::{Body, Frame, SizeHint};
use tokio::sync::mpsc;

/// One-shot, destination-bound upload over caller-owned source and scratch.
/// Construction immediately arms scratch cleanup, including unpolled drops.
/// Authorization is the complete provider-validated value, without an inferred
/// prefix. Caller-owned source and credential storage remain caller obligations.
pub struct RawUpload<'a, S> {
    expected: EndpointIdentity<'a>,
    authorization: cloud_sdk::transport::HeaderValue<'a>,
    source: &'a mut S,
    policy: StreamPolicy,
    scratch: SecretBuffer<'a>,
}
impl<'a, S> RawUpload<'a, S> {
    /// Arms one finite, declared-length direct upload. Invalid policies are
    /// rejected before dispatch. Reusing the source never authorizes replay.
    pub fn new(
        expected: EndpointIdentity<'a>,
        authorization: cloud_sdk::transport::HeaderValue<'a>,
        source: &'a mut S,
        policy: StreamPolicy,
        scratch: &'a mut [u8],
    ) -> Self {
        cloud_sdk_sanitization::sanitize_bytes(scratch);
        Self {
            expected,
            authorization,
            source,
            policy,
            scratch: SecretBuffer::new(scratch),
        }
    }
}
impl<S> core::fmt::Debug for RawUpload<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("RawUpload([redacted])")
    }
}

pub(super) struct UploadBody {
    receiver: mpsc::Receiver<Bytes>,
    state: Arc<AtomicU8>,
    remaining: u64,
}
impl Body for UploadBody {
    type Data = Bytes;
    type Error = RawHttpError;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, RawHttpError>>> {
        if self.state.load(Ordering::Acquire) == 2 {
            return Poll::Ready(Some(Err(RawHttpError::UploadFailed)));
        }
        match self.receiver.poll_recv(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(bytes)) => {
                let Some(remaining) = u64::try_from(bytes.len())
                    .ok()
                    .and_then(|len| self.remaining.checked_sub(len))
                else {
                    return Poll::Ready(Some(Err(RawHttpError::UploadFailed)));
                };
                self.remaining = remaining;
                Poll::Ready(Some(Ok(Frame::data(bytes))))
            }
            Poll::Ready(None) if self.remaining == 0 && self.state.load(Ordering::Acquire) == 1 => {
                Poll::Ready(None)
            }
            Poll::Ready(None) => Poll::Ready(Some(Err(RawHttpError::UploadFailed))),
        }
    }
    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.remaining)
    }
}

struct UploadSink {
    sender: Option<mpsc::Sender<Bytes>>,
    state: Arc<AtomicU8>,
}
impl AsyncStreamSink for UploadSink {
    type Error = RawHttpError;
    async fn write_chunk<'a>(&'a mut self, input: &'a [u8]) -> Result<usize, Self::Error> {
        let permit = self
            .sender
            .as_ref()
            .ok_or(RawHttpError::UploadFailed)?
            .reserve()
            .await
            .map_err(|_| RawHttpError::UploadFailed)?;
        let bytes = SanitizedBody::copy_from(input)
            .map_err(|_| RawHttpError::RequestBodyAllocationFailed)?
            .into_bytes();
        permit.send(bytes);
        Ok(input.len())
    }
    async fn commit(&mut self) -> Result<(), Self::Error> {
        self.state.store(1, Ordering::Release);
        self.sender.take();
        Ok(())
    }
    fn abort(&mut self, _: StreamPartialState) {
        self.state.store(2, Ordering::Release);
        self.sender.take();
    }
}
impl Drop for UploadSink {
    fn drop(&mut self) {
        if self.sender.is_some() {
            self.state.store(2, Ordering::Release);
        }
    }
}

macro_rules! execute_upload {
    ($name:ident, $source:path, $driver:path $(, $send:ident)?) => {
        pub(crate) async fn $name<'buffer, S: $source $(+ $send)?>(
            &self, request: TransportRequest<'_>, policy: RawResponsePolicy<'_>,
            mut upload: RawUpload<'_, S>, response: &mut impl RawResponseSink<'buffer>,
        ) -> Result<ResponseCompletion, RawTransportFailure> {
            let length = self.validate_upload(&request, &upload).map_err(TransportFailure::not_sent)?;
            let authorization = super::super::sensitive_header_value(upload.authorization.as_str().as_bytes())
                .map_err(|_| TransportFailure::not_sent(RawHttpError::HeaderRejected))?;
            let method = request.method();
            let mut request = self.prepare_request(request, Some(authorization))?;
            let (sender, receiver) = mpsc::channel(1);
            let upload_state = Arc::new(AtomicU8::new(0));
            *request.body_mut() = RequestBody::Upload(UploadBody { receiver, state: Arc::clone(&upload_state), remaining: length });
            request.headers_mut().insert(http::header::CONTENT_LENGTH, HeaderValue::from(length));
            let state = Arc::new(ResponseState::new(policy.informational_limit()));
            let observer = Arc::clone(&state);
            hyper::ext::on_informational(&mut request, move |head| observer.observe_informational(head.status().as_u16()));
            let operation = async {
                let mut sink = UploadSink { sender: Some(sender), state: upload_state };
                let mut outcome = StreamOutcome::new();
                let mut producer = core::pin::pin!($driver(upload.policy, upload.source, &mut sink, upload.scratch.as_mut_slice(), &mut outcome));
                let mut exchange = core::pin::pin!(self.execute_timed(method, request, policy, response, &state));
                let mut uploaded = false;
                poll_fn(|cx| {
                    if !uploaded {
                        match producer.as_mut().poll(cx) {
                            Poll::Ready(Ok(_)) => uploaded = true,
                            Poll::Ready(Err(error)) => {
                                let error = match error {
                                    cloud_sdk::transport::StreamExecutionError::Sink(error) => error,
                                    _ => RawHttpError::UploadFailed,
                                };
                                return Poll::Ready(Err(upload_failure(&state, error)));
                            }
                            Poll::Pending => {}
                        }
                    }
                    let result = exchange.as_mut().poll(cx);
                    if !uploaded && state.final_started.load(Ordering::Acquire) {
                        return Poll::Ready(Err(TransportFailure::response_started(RawHttpError::UploadIncomplete)));
                    }
                    match result {
                        Poll::Ready(Ok(_)) if !uploaded => Poll::Ready(Err(TransportFailure::response_started(RawHttpError::UploadIncomplete))),
                        other => other,
                    }
                }).await
            };
            tokio::time::timeout(self.timeouts.total(), operation).await
                .unwrap_or_else(|_| Err(upload_failure(&state, RawHttpError::TimedOut)))
        }
    }
}
impl RawHyperClient {
    #[cfg(feature = "async-rustls")]
    execute_upload!(execute_upload, AsyncStreamSource, drive_async_stream, Send);
    execute_upload!(
        execute_upload_local,
        LocalAsyncStreamSource,
        drive_local_stream
    );

    fn validate_upload<S>(
        &self,
        request: &TransportRequest<'_>,
        upload: &RawUpload<'_, S>,
    ) -> Result<u64, RawHttpError> {
        if self.endpoint.identity().ok() != Some(upload.expected) {
            return Err(RawHttpError::TargetRejected);
        }
        if !request.body().is_empty()
            || matches!(request.method(), Method::Get | Method::Head)
            || upload.scratch.as_slice().is_empty()
            || upload.policy.kind() != StreamKind::FiniteUpload
            || upload.policy.sink_mode() != StreamSinkMode::Direct
        {
            return Err(RawHttpError::InvalidStreamState);
        }
        if request.headers().get("content-type").is_none() {
            return Err(RawHttpError::MissingContentType);
        }
        match upload.policy.framing() {
            StreamFraming::Declared(length) if length > 0 => Ok(length),
            _ => Err(RawHttpError::InvalidStreamState),
        }
    }
}

#[cfg(any(feature = "blocking-rustls", feature = "blocking-rustls-webpki-roots"))]
impl RawHyperClient {
    pub(crate) async fn execute_upload_blocking<
        'buffer,
        S: cloud_sdk::transport::BlockingStreamSource,
    >(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        upload: RawUpload<'_, S>,
        response: &mut impl RawResponseSink<'buffer>,
    ) -> Result<ResponseCompletion, RawTransportFailure> {
        struct Source<'a, S>(&'a mut S);
        impl<S: cloud_sdk::transport::BlockingStreamSource> LocalAsyncStreamSource for Source<'_, S> {
            type Error = S::Error;
            fn replayability(&self) -> cloud_sdk::transport::StreamReplayability<'_> {
                self.0.replayability()
            }
            async fn read_chunk_local<'a>(
                &'a mut self,
                output: &'a mut [u8],
            ) -> Result<cloud_sdk::transport::StreamRead, Self::Error> {
                self.0.read_chunk(output)
            }
        }
        let RawUpload {
            expected,
            authorization,
            source,
            policy: stream_policy,
            scratch,
        } = upload;
        let mut source = Source(source);
        self.execute_upload_local(
            request,
            policy,
            RawUpload {
                expected,
                authorization,
                source: &mut source,
                policy: stream_policy,
                scratch,
            },
            response,
        )
        .await
    }
}
fn upload_failure(state: &ResponseState, error: RawHttpError) -> RawTransportFailure {
    if state.response_started() {
        TransportFailure::response_started(error)
    } else {
        TransportFailure::possibly_sent(error)
    }
}
