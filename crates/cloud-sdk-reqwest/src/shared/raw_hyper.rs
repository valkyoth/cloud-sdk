use core::future::{Future, poll_fn};
use core::task::Poll;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::vec::Vec;

use bytes::Bytes;
use cloud_sdk::Method;
use cloud_sdk::transport::{
    AsyncResponseStaging, RawResponsePolicy, ResponseAttempt, ResponseCompletion, ResponseHeaders,
    ResponseMetadata, ResponseWriterError, StatusCode, TransportFailure, TransportRequest,
};
use cloud_sdk_sanitization::sanitize_bytes;
use http::header::{AUTHORIZATION, HeaderName, HeaderValue, USER_AGENT};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
#[cfg(any(
    feature = "async-rustls",
    all(
        feature = "blocking-rustls",
        not(feature = "blocking-rustls-webpki-roots")
    )
))]
use hyper_rustls::ConfigBuilderExt;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::{TokioExecutor, TokioTimer};
use rustls::ClientConfig;
use tokio::sync::Notify;

use super::raw::ResponseBodyBudget;
use super::{
    BuildError, HttpsEndpoint, RawHttpError, RawTransportFailure, RequestTimeouts, UserAgent,
    inspect_response_head,
};

type HttpClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

pub(crate) trait RawResponseSink<'buffer> {
    fn body_capacity(&self) -> usize;
    fn body_mut(&mut self) -> Result<&mut [u8], ResponseWriterError>;
    fn headers_mut(&mut self) -> Result<&mut ResponseHeaders<'buffer>, ResponseWriterError>;
}

impl<'buffer> RawResponseSink<'buffer> for ResponseAttempt<'_, 'buffer> {
    fn body_capacity(&self) -> usize {
        ResponseAttempt::body_capacity(self)
    }

    fn body_mut(&mut self) -> Result<&mut [u8], ResponseWriterError> {
        ResponseAttempt::body_mut(self)
    }

    fn headers_mut(&mut self) -> Result<&mut ResponseHeaders<'buffer>, ResponseWriterError> {
        ResponseAttempt::headers_mut(self)
    }
}

impl<'buffer> RawResponseSink<'buffer> for AsyncResponseStaging<'_, 'buffer> {
    fn body_capacity(&self) -> usize {
        AsyncResponseStaging::body_capacity(self)
    }

    fn body_mut(&mut self) -> Result<&mut [u8], ResponseWriterError> {
        AsyncResponseStaging::body_mut(self)
    }

    fn headers_mut(&mut self) -> Result<&mut ResponseHeaders<'buffer>, ResponseWriterError> {
        AsyncResponseStaging::headers_mut(self)
    }
}

pub(super) struct ResponseState {
    informational_limit: u8,
    informational_count: AtomicU8,
    informational_rejection: AtomicU8,
    rejection_notification: Notify,
    final_started: AtomicBool,
}

impl ResponseState {
    pub(super) const fn new(informational_limit: u8) -> Self {
        Self {
            informational_limit,
            informational_count: AtomicU8::new(0),
            informational_rejection: AtomicU8::new(0),
            rejection_notification: Notify::const_new(),
            final_started: AtomicBool::new(false),
        }
    }

    pub(super) fn observe_informational(&self, status: u16) {
        if status == 101 {
            self.reject_informational(2);
            return;
        }
        let previous = self.increment_informational_count();
        if previous.is_none() || previous.is_some_and(|value| value >= self.informational_limit) {
            self.reject_informational(1);
        }
    }

    fn increment_informational_count(&self) -> Option<u8> {
        let mut current = self.informational_count.load(Ordering::Acquire);
        loop {
            let next = current.checked_add(1)?;
            match self.informational_count.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => return Some(previous),
                Err(observed) => current = observed,
            }
        }
    }

    fn reject_informational(&self, reason: u8) {
        if self
            .informational_rejection
            .compare_exchange(0, reason, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            self.rejection_notification.notify_one();
        }
    }

    pub(super) fn informational_rejection(&self) -> Option<RawHttpError> {
        match self.informational_rejection.load(Ordering::Acquire) {
            1 => Some(RawHttpError::TooManyInformationalResponses),
            2 => Some(RawHttpError::SwitchingProtocols),
            _ => None,
        }
    }

    async fn wait_for_informational_rejection(&self) -> RawHttpError {
        loop {
            let notified = self.rejection_notification.notified();
            if let Some(error) = self.informational_rejection() {
                return error;
            }
            notified.await;
        }
    }

    fn response_started(&self) -> bool {
        self.final_started.load(Ordering::Acquire)
            || self.informational_count.load(Ordering::Acquire) != 0
            || self.informational_rejection.load(Ordering::Acquire) != 0
    }
}

/// Shared bounded HTTP/1 engine used by raw blocking and async adapters.
#[derive(Clone)]
pub(crate) struct RawHyperClient {
    client: HttpClient,
    endpoint: HttpsEndpoint,
    user_agent: HeaderValue,
    timeouts: RequestTimeouts,
}

impl RawHyperClient {
    pub(crate) fn new(
        endpoint: HttpsEndpoint,
        user_agent: &UserAgent,
        timeouts: RequestTimeouts,
        tls_config: ClientConfig,
        https_only: bool,
    ) -> Result<Self, BuildError> {
        if !tls_config.alpn_protocols.is_empty() {
            return Err(BuildError::ClientBuildFailed);
        }
        let mut connector = HttpConnector::new();
        connector.enforce_http(false);
        connector.set_connect_timeout(Some(timeouts.connect()));
        connector.set_nodelay(true);
        let builder = hyper_rustls::HttpsConnectorBuilder::new().with_tls_config(tls_config);
        let connector = if https_only {
            builder
                .https_only()
                .enable_http1()
                .wrap_connector(connector)
        } else {
            builder
                .https_or_http()
                .enable_http1()
                .wrap_connector(connector)
        };
        let mut builder = Client::builder(TokioExecutor::new());
        builder
            .http1_max_headers(super::MAX_UPSTREAM_HTTP1_HEADERS)
            .http1_max_buf_size(super::MAX_UPSTREAM_HTTP1_HEAD_BYTES)
            .pool_max_idle_per_host(0)
            .retry_canceled_requests(false)
            .timer(TokioTimer::new());
        Ok(Self {
            client: builder.build(connector),
            endpoint,
            user_agent: user_agent.value.clone(),
            timeouts,
        })
    }

    pub(crate) async fn execute<'buffer>(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut impl RawResponseSink<'buffer>,
    ) -> Result<ResponseCompletion, RawTransportFailure> {
        self.execute_inner(request, policy, None, response).await
    }

    pub(crate) async fn execute_authenticated<'buffer>(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        authorization: HeaderValue,
        response: &mut impl RawResponseSink<'buffer>,
    ) -> Result<ResponseCompletion, RawTransportFailure> {
        self.execute_inner(request, policy, Some(authorization), response)
            .await
    }

    async fn execute_inner<'buffer>(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        authorization: Option<HeaderValue>,
        response: &mut impl RawResponseSink<'buffer>,
    ) -> Result<ResponseCompletion, RawTransportFailure> {
        let method = request.method();
        let request = self.prepare_request(request, authorization)?;
        let state = Arc::new(ResponseState::new(policy.informational_limit()));
        let observer = Arc::clone(&state);
        let mut request = request;
        hyper::ext::on_informational(&mut request, move |head| {
            observer.observe_informational(head.status().as_u16());
        });
        let operation = self.execute_timed(method, request, policy, response, state.as_ref());
        match tokio::time::timeout(self.timeouts.total(), operation).await {
            Ok(result) => result,
            Err(_) if state.response_started() => {
                Err(TransportFailure::response_started(RawHttpError::TimedOut))
            }
            Err(_) => Err(TransportFailure::possibly_sent(RawHttpError::TimedOut)),
        }
    }

    fn prepare_request(
        &self,
        request: TransportRequest<'_>,
        authorization: Option<HeaderValue>,
    ) -> Result<http::Request<Full<Bytes>>, RawTransportFailure> {
        validate_request_body_len(request.body()).map_err(TransportFailure::not_sent)?;
        let url = self
            .endpoint
            .compose(request.target())
            .map_err(|_| TransportFailure::not_sent(RawHttpError::TargetRejected))?;
        let method = http::Method::from_bytes(request.method().as_str().as_bytes())
            .map_err(|_| TransportFailure::not_sent(RawHttpError::MethodRejected))?;
        let uri = url
            .as_str()
            .parse::<http::Uri>()
            .map_err(|_| TransportFailure::not_sent(RawHttpError::TargetRejected))?;
        let mut builder = http::Request::builder().method(method).uri(uri);
        let Some(headers) = builder.headers_mut() else {
            return Err(TransportFailure::not_sent(RawHttpError::RequestBuildFailed));
        };
        if let Some(mut authorization) = authorization {
            authorization.set_sensitive(true);
            headers.insert(AUTHORIZATION, authorization);
        }
        for header in request.headers().as_slice() {
            let name = HeaderName::from_bytes(header.name().as_str().as_bytes())
                .map_err(|_| TransportFailure::not_sent(RawHttpError::HeaderRejected))?;
            let value_storage = SanitizedBody::copy_from(header.value().as_str().as_bytes())
                .map_err(|_| {
                    TransportFailure::not_sent(RawHttpError::RequestHeaderAllocationFailed)
                })?
                .into_bytes();
            let mut value = HeaderValue::from_maybe_shared(value_storage)
                .map_err(|_| TransportFailure::not_sent(RawHttpError::HeaderRejected))?;
            value.set_sensitive(matches!(
                header.sensitivity(),
                cloud_sdk::transport::HeaderSensitivity::Sensitive
            ));
            headers.insert(name, value);
        }
        headers.insert(USER_AGENT, self.user_agent.clone());
        if !request.body().is_empty() && request.headers().get("content-type").is_none() {
            return Err(TransportFailure::not_sent(RawHttpError::MissingContentType));
        }
        let body = if request.body().is_empty() {
            Bytes::new()
        } else {
            stage_request_body(request.body()).map_err(TransportFailure::not_sent)?
        };
        builder
            .body(Full::new(body))
            .map_err(|_| TransportFailure::not_sent(RawHttpError::RequestBuildFailed))
    }

    async fn execute_timed<'buffer>(
        &self,
        method: Method,
        request: http::Request<Full<Bytes>>,
        policy: RawResponsePolicy<'_>,
        sink: &mut impl RawResponseSink<'buffer>,
        state: &ResponseState,
    ) -> Result<ResponseCompletion, RawTransportFailure> {
        let mut request = core::pin::pin!(self.client.request(request));
        let mut rejection = core::pin::pin!(state.wait_for_informational_rejection());
        let response = poll_fn(|context| {
            if let Poll::Ready(error) = rejection.as_mut().poll(context) {
                return Poll::Ready(Err(TransportFailure::response_started(error)));
            }
            request.as_mut().poll(context).map(|result| {
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
        })
        .await?;
        if let Some(error) = state.informational_rejection() {
            return Err(TransportFailure::response_started(error));
        }
        state.final_started.store(true, Ordering::Release);
        let status = StatusCode::new(response.status().as_u16())
            .ok_or_else(|| TransportFailure::response_started(RawHttpError::InvalidStatus))?;
        let writer_capacity = sink.body_capacity();
        let body_limit = {
            let headers = sink.headers_mut().map_err(|_| {
                TransportFailure::response_started_with_status(
                    status,
                    RawHttpError::ResponseCommitFailed,
                )
            })?;
            inspect_response_head(
                method,
                status,
                response.headers(),
                policy,
                headers,
                writer_capacity,
            )
            .map_err(|error| TransportFailure::response_started_with_status(status, error))?
        };
        let body_len = read_bounded_body(response.into_body(), sink, body_limit)
            .await
            .map_err(|error| error.with_observed_status(status))?;
        Ok(ResponseCompletion::new(
            status,
            body_len,
            ResponseMetadata::EMPTY,
        ))
    }
}

#[cfg(any(
    feature = "async-rustls",
    all(
        feature = "blocking-rustls",
        not(feature = "blocking-rustls-webpki-roots")
    )
))]
pub(crate) fn platform_client_config() -> Result<ClientConfig, BuildError> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    Ok(ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|_| BuildError::ProtocolConfigurationFailed)?
        .try_with_platform_verifier()
        .map_err(|_| BuildError::PlatformVerifierConfigurationFailed)?
        .with_no_client_auth())
}

pub(super) async fn read_bounded_body<'buffer>(
    mut body: Incoming,
    writer: &mut impl RawResponseSink<'buffer>,
    limit: usize,
) -> Result<usize, RawTransportFailure> {
    let mut budget = ResponseBodyBudget::new(limit);
    while let Some(frame) = body.frame().await {
        let frame = frame
            .map_err(|_| TransportFailure::response_started(RawHttpError::ResponseReadFailed))?;
        let data = match frame.into_data() {
            Ok(data) => data,
            Err(frame) if frame.is_trailers() => {
                return Err(TransportFailure::response_started(
                    RawHttpError::ResponseTrailersRejected,
                ));
            }
            Err(_) => continue,
        };
        let range = budget
            .observe(data.len())
            .map_err(TransportFailure::response_started)?;
        let output = writer
            .body_mut()
            .map_err(|_| TransportFailure::response_started(RawHttpError::ResponseCommitFailed))?;
        output
            .get_mut(range)
            .ok_or_else(|| TransportFailure::response_started(RawHttpError::ResponseTooLarge))?
            .copy_from_slice(&data);
    }
    Ok(budget.len())
}

struct SanitizedBody {
    bytes: Vec<u8>,
}

impl SanitizedBody {
    fn copy_from(source: &[u8]) -> Result<Self, ()> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(source.len()).map_err(|_| ())?;
        bytes.extend_from_slice(source);
        Ok(Self { bytes })
    }

    fn into_bytes(self) -> Bytes {
        Bytes::from_owner(self)
    }
}

impl AsRef<[u8]> for SanitizedBody {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Drop for SanitizedBody {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.bytes);
    }
}

fn validate_request_body_len(source: &[u8]) -> Result<(), RawHttpError> {
    if source.len() > super::MAX_RAW_REQUEST_BODY_BYTES {
        Err(RawHttpError::RequestBodyTooLarge)
    } else {
        Ok(())
    }
}

fn stage_request_body(source: &[u8]) -> Result<Bytes, RawHttpError> {
    validate_request_body_len(source)?;
    SanitizedBody::copy_from(source)
        .map(SanitizedBody::into_bytes)
        .map_err(|()| RawHttpError::RequestBodyAllocationFailed)
}

#[cfg(test)]
mod tests {
    use super::{stage_request_body, validate_request_body_len};
    use crate::shared::{MAX_RAW_REQUEST_BODY_BYTES, RawHttpError};

    #[test]
    fn raw_request_body_bound_accepts_exact_and_rejects_plus_one() {
        let exact = std::vec![0x5a; MAX_RAW_REQUEST_BODY_BYTES];
        let staged = stage_request_body(&exact);
        assert!(staged.is_ok_and(|body| body.len() == MAX_RAW_REQUEST_BODY_BYTES));

        let oversized = std::vec![0x5a; MAX_RAW_REQUEST_BODY_BYTES.saturating_add(1)];
        assert_eq!(
            validate_request_body_len(&oversized),
            Err(RawHttpError::RequestBodyTooLarge)
        );
    }
}
