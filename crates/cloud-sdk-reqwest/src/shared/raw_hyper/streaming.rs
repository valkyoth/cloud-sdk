use super::*;
use cloud_sdk::transport::{AsyncStreamSource, StreamRead, StreamReplayability};
use tokio::time::{Instant, timeout_at};

/// Anonymous HTTP/1 response body, read incrementally under the original
/// request deadline. Dropping this value closes the unpooled exchange. No
/// credentials, decompression, redirects, trailers, cookies or retries are used.
/// Each source is single-use, including after a read error or cancellation.
pub struct StreamingResponse {
    body: Option<Incoming>,
    pending: Bytes,
    deadline: Instant,
    maximum: u64,
    observed: u64,
    frames: usize,
    length: Option<u64>,
    ended: bool,
}

impl core::fmt::Debug for StreamingResponse {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("StreamingResponse([redacted])")
    }
}
impl StreamingResponse {
    /// Validated optional Content-Length; the source also enforces it at EOF.
    #[must_use]
    pub const fn content_length(&self) -> Option<u64> {
        self.length
    }

    async fn read(&mut self, output: &mut [u8]) -> Result<StreamRead, RawTransportFailure> {
        if output.is_empty() || (self.body.is_none() && !self.ended) {
            return Err(TransportFailure::response_started(
                RawHttpError::RequestFailed,
            ));
        }
        if Instant::now() >= self.deadline {
            return Err(TransportFailure::response_started(RawHttpError::TimedOut));
        }
        if self.ended {
            return Ok(StreamRead::End);
        }
        if self.pending.is_empty() {
            // Take ownership before awaiting: cancellation drops the live body
            // and cannot resume a partially consumed frame as a fresh read.
            let mut body = self
                .body
                .take()
                .ok_or_else(|| TransportFailure::response_started(RawHttpError::RequestFailed))?;
            let frame = timeout_at(self.deadline, body.frame())
                .await
                .map_err(|_| TransportFailure::response_started(RawHttpError::TimedOut))?;
            match frame {
                None => {
                    if self.length.is_some_and(|length| length != self.observed) {
                        return Err(TransportFailure::response_started(
                            RawHttpError::ResponseReadFailed,
                        ));
                    }
                    self.ended = true;
                    return Ok(StreamRead::End);
                }
                Some(frame) => {
                    let data = frame
                        .map_err(|_| {
                            TransportFailure::response_started(RawHttpError::ResponseReadFailed)
                        })?
                        .into_data()
                        .map_err(|_| {
                            TransportFailure::response_started(
                                RawHttpError::ResponseTrailersRejected,
                            )
                        })?;
                    self.frames = self
                        .frames
                        .checked_add(1)
                        .filter(|v| *v <= cloud_sdk::transport::MAX_RESPONSE_CHUNKS)
                        .ok_or_else(|| {
                            TransportFailure::response_started(
                                RawHttpError::ResponseChunkLimitExceeded,
                            )
                        })?;
                    self.observed = self
                        .observed
                        .checked_add(u64::try_from(data.len()).map_err(|_| {
                            TransportFailure::response_started(RawHttpError::ResponseTooLarge)
                        })?)
                        .filter(|len| {
                            *len <= self.maximum
                                && self.length.is_none_or(|declared| *len <= declared)
                        })
                        .ok_or_else(|| {
                            TransportFailure::response_started(RawHttpError::ResponseTooLarge)
                        })?;
                    self.pending = data;
                    self.body = Some(body);
                    if self.pending.is_empty() {
                        return Ok(StreamRead::Wait);
                    }
                }
            }
        }
        let len = output.len().min(self.pending.len());
        let bytes = self.pending.split_to(len);
        output
            .get_mut(..len)
            .ok_or_else(|| TransportFailure::response_started(RawHttpError::RequestFailed))?
            .copy_from_slice(&bytes);
        Ok(StreamRead::Chunk(len))
    }
}

impl AsyncStreamSource for StreamingResponse {
    type Error = RawTransportFailure;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    async fn read_chunk<'a>(&'a mut self, output: &'a mut [u8]) -> Result<StreamRead, Self::Error> {
        let result = self.read(output).await;
        if result.is_err() {
            self.body = None;
            self.pending = Bytes::new();
            self.ended = false;
        }
        result
    }
}

impl RawHyperClient {
    pub(crate) async fn open_stream(
        &self,
        request: TransportRequest<'_>,
        maximum: u64,
    ) -> Result<StreamingResponse, RawTransportFailure> {
        if request.method() != Method::Get || !request.body().is_empty() || maximum == 0 {
            return Err(TransportFailure::not_sent(RawHttpError::RequestBuildFailed));
        }
        let deadline = Instant::now()
            .checked_add(self.timeouts.total())
            .ok_or_else(|| TransportFailure::not_sent(RawHttpError::TimedOut))?;
        let mut request = self.prepare_request(request, None)?;
        request.headers_mut().insert(
            http::header::ACCEPT_ENCODING,
            HeaderValue::from_static("identity"),
        );
        let state = Arc::new(ResponseState::new(
            cloud_sdk::transport::MAX_INFORMATIONAL_RESPONSES,
        ));
        let observer = Arc::clone(&state);
        hyper::ext::on_informational(&mut request, move |head| {
            observer.observe_informational(head.status().as_u16());
        });
        let mut exchange = core::pin::pin!(self.client.request(request));
        let mut rejection = core::pin::pin!(state.wait_for_informational_rejection());
        let response = timeout_at(
            deadline,
            poll_fn(|context| {
                if let Poll::Ready(error) = rejection.as_mut().poll(context) {
                    return Poll::Ready(Err(TransportFailure::response_started(error)));
                }
                exchange.as_mut().poll(context).map(|result| {
                    result.map_err(|error| {
                        if state.response_started() {
                            TransportFailure::response_started(RawHttpError::RequestFailed)
                        } else if error.is_connect() {
                            TransportFailure::not_sent(RawHttpError::ConnectFailed)
                        } else {
                            TransportFailure::possibly_sent(RawHttpError::RequestFailed)
                        }
                    })
                })
            }),
        )
        .await
        .map_err(|_| {
            if state.response_started() {
                TransportFailure::response_started(RawHttpError::TimedOut)
            } else {
                TransportFailure::possibly_sent(RawHttpError::TimedOut)
            }
        })??;
        if let Some(error) = state.informational_rejection() {
            return Err(TransportFailure::response_started(error));
        }
        let status = StatusCode::new(response.status().as_u16())
            .ok_or_else(|| TransportFailure::response_started(RawHttpError::InvalidStatus))?;
        let length = stream_head(status, response.headers(), maximum)
            .map_err(|error| TransportFailure::response_started_with_status(status, error))?;
        Ok(StreamingResponse {
            body: Some(response.into_body()),
            pending: Bytes::new(),
            deadline,
            maximum,
            observed: 0,
            frames: 0,
            length,
            ended: false,
        })
    }
}

fn stream_head(
    status: StatusCode,
    headers: &http::HeaderMap,
    maximum: u64,
) -> Result<Option<u64>, RawHttpError> {
    super::super::raw::validate_wire_head(headers)?;
    if status != StatusCode::OK {
        return Err(RawHttpError::InvalidStatus);
    }
    if headers.contains_key(http::header::TRAILER) {
        return Err(RawHttpError::ResponseTrailersRejected);
    }
    if headers
        .get(http::header::CONTENT_ENCODING)
        .is_some_and(|v| v.as_bytes() != b"identity")
    {
        return Err(RawHttpError::InvalidResponseHeader);
    }
    let length = super::super::raw::declared_content_length(headers)?;
    if headers
        .get(http::header::TRANSFER_ENCODING)
        .is_some_and(|v| v.as_bytes() != b"chunked" || length.is_some())
    {
        return Err(RawHttpError::InvalidNoBodyFraming);
    }
    if length.is_some_and(|len| len > maximum) {
        return Err(RawHttpError::ResponseTooLarge);
    }
    Ok(length)
}
