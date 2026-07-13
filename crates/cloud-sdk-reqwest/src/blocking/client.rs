use core::fmt;
use std::io::Read;

use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, StatusCode, TransportRequest, TransportResponse};
use cloud_sdk_sanitization::sanitize_bytes;
use reqwest::blocking::{Body, Client};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderValue};

use super::body::{ReadBodyError, SanitizedRequestBody, read_bounded};
use crate::shared::{BearerToken, HttpsEndpoint, TransportError, parse_rate_limit};

/// Hardened provider-neutral reqwest blocking transport.
pub struct BlockingClient {
    client: Client,
    endpoint: HttpsEndpoint,
    token: BearerToken,
}

impl BlockingClient {
    pub(super) const fn new(client: Client, endpoint: HttpsEndpoint, token: BearerToken) -> Self {
        Self {
            client,
            endpoint,
            token,
        }
    }

    fn send_inner<'buffer>(
        &mut self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, TransportError> {
        sanitize_bytes(response_body);
        let url = self
            .endpoint
            .compose(request.target())
            .map_err(|_| TransportError::TargetRejected)?;
        let method = map_method(request.method());
        let authorization = self
            .token
            .header_value()
            .map_err(|_| TransportError::HeaderRejected)?;
        let mut outbound = self
            .client
            .request(method, url)
            .header(AUTHORIZATION, authorization)
            .header(ACCEPT, HeaderValue::from_static("application/json"));

        if let Some(content_type) = request.content_type() {
            let value = HeaderValue::from_str(content_type.as_str())
                .map_err(|_| TransportError::HeaderRejected)?;
            outbound = outbound.header(CONTENT_TYPE, value);
        } else if !request.body().is_empty() {
            return Err(TransportError::MissingContentType);
        }

        if !request.body().is_empty() {
            let body = SanitizedRequestBody::new(request.body())
                .map_err(|_| TransportError::RequestBodyAllocationFailed)?;
            let body_len = u64::try_from(request.body().len())
                .map_err(|_| TransportError::RequestBodyTooLarge)?;
            outbound = outbound.body(Body::sized(body, body_len));
        }

        let mut response = outbound.send().map_err(classify_reqwest_error)?;
        self.endpoint
            .verify_origin(response.url())
            .map_err(|_| TransportError::ResponseOriginChanged)?;
        if response.content_length().is_some_and(|length| {
            u64::try_from(response_body.len()).map_or(true, |cap| length > cap)
        }) {
            return Err(TransportError::ResponseTooLarge);
        }
        let status =
            StatusCode::new(response.status().as_u16()).ok_or(TransportError::InvalidStatus)?;
        let rate_limit = parse_rate_limit(response.headers())?;
        let body_len = read_response(&mut response, response_body)?;
        let initialized = response_body
            .get(..body_len)
            .ok_or(TransportError::ResponseReadFailed)?;
        let response = TransportResponse::new(status, initialized);
        Ok(rate_limit.map_or(response, |value| response.with_rate_limit(value)))
    }
}

impl BlockingTransport for BlockingClient {
    type Error = TransportError;

    fn send<'buffer>(
        &mut self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error> {
        self.send_inner(request, response_body)
    }
}

impl fmt::Debug for BlockingClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlockingClient")
            .field("endpoint", &"[redacted]")
            .field("token", &"[redacted]")
            .finish_non_exhaustive()
    }
}

fn read_response(response: &mut impl Read, output: &mut [u8]) -> Result<usize, TransportError> {
    match read_bounded(response, output) {
        Ok(len) => Ok(len),
        Err(error) => {
            sanitize_bytes(output);
            Err(match error {
                ReadBodyError::TooLarge => TransportError::ResponseTooLarge,
                ReadBodyError::ReadFailed => TransportError::ResponseReadFailed,
            })
        }
    }
}

const fn map_method(method: Method) -> reqwest::Method {
    match method {
        Method::Get => reqwest::Method::GET,
        Method::Post => reqwest::Method::POST,
        Method::Put => reqwest::Method::PUT,
        Method::Delete => reqwest::Method::DELETE,
    }
}

fn classify_reqwest_error(error: reqwest::Error) -> TransportError {
    if error.is_timeout() {
        TransportError::TimedOut
    } else if error.is_connect() {
        TransportError::ConnectFailed
    } else {
        TransportError::RequestFailed
    }
}
