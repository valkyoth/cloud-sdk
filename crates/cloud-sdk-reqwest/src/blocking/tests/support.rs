use std::{string::String, vec::Vec};

use cloud_sdk::authentication::BlockingAuthenticatedTransport;
use cloud_sdk::rate_limit::RateLimit;
use cloud_sdk::transport::{ResponseBuffer, StatusCode, TransportRequest, TransportResponse};

use super::super::{BlockingClient, TransportError};
use super::authenticated;

pub(super) struct CapturedResponse {
    status: StatusCode,
    body: Vec<u8>,
    content_type: Option<String>,
    rate_limit: Option<RateLimit>,
    rate_limit_remaining: Option<Vec<u8>>,
    content_type_header: Option<Vec<u8>>,
}

impl CapturedResponse {
    fn capture(response: TransportResponse<'_, '_>) -> Self {
        Self {
            status: response.status(),
            body: response.body().to_vec(),
            content_type: response
                .content_type()
                .ok()
                .flatten()
                .map(|content_type| String::from(content_type.as_str())),
            rate_limit: response.rate_limit(),
            rate_limit_remaining: response
                .headers()
                .get("ratelimit-remaining")
                .map(|header| header.value().to_vec()),
            content_type_header: response
                .headers()
                .get("content-type")
                .map(|header| header.value().to_vec()),
        }
    }

    pub(super) const fn status(&self) -> StatusCode {
        self.status
    }

    pub(super) fn body(&self) -> &[u8] {
        &self.body
    }

    pub(super) fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    pub(super) const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }

    pub(super) fn rate_limit_remaining_header(&self) -> Option<&[u8]> {
        self.rate_limit_remaining.as_deref()
    }

    pub(super) fn content_type_header(&self) -> Option<&[u8]> {
        self.content_type_header.as_deref()
    }
}

pub(super) fn send_test(
    client: &BlockingClient,
    request: TransportRequest<'_>,
    output: &mut [u8],
) -> Result<CapturedResponse, TransportError> {
    let capacity = output.len();
    let mut headers = [0_u8; 8192];
    let mut response = ResponseBuffer::new(output, capacity, &mut headers);
    BlockingAuthenticatedTransport::send_authenticated(
        client,
        authenticated(request),
        response.writer(),
    )?;
    response
        .with_response(CapturedResponse::capture)
        .map_err(|_| TransportError::ResponseCommitFailed)
}
