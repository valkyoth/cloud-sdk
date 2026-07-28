use std::vec::Vec;

use cloud_sdk::rate_limit::RateLimit;
use cloud_sdk::transport::{
    BlockingTransport, ResponseBuffer, ResponseContentType, ResponseHeaders, StatusCode,
    TransportRequest, TransportResponse,
};

use super::super::{BlockingClient, TransportError};

pub(super) struct CapturedResponse {
    status: StatusCode,
    body: Vec<u8>,
    content_type: Option<ResponseContentType>,
    rate_limit: Option<RateLimit>,
    headers: ResponseHeaders,
}

impl CapturedResponse {
    fn capture(response: TransportResponse<'_>) -> Self {
        Self {
            status: response.status(),
            body: response.body().to_vec(),
            content_type: response
                .content_type()
                .map(ResponseContentType::retain_copy),
            rate_limit: response.rate_limit(),
            headers: response.headers().retain_copy(),
        }
    }

    pub(super) const fn status(&self) -> StatusCode {
        self.status
    }

    pub(super) fn body(&self) -> &[u8] {
        &self.body
    }

    pub(super) const fn content_type(&self) -> Option<&ResponseContentType> {
        self.content_type.as_ref()
    }

    pub(super) const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }

    pub(super) const fn headers(&self) -> &ResponseHeaders {
        &self.headers
    }
}

pub(super) fn send_test(
    client: &BlockingClient,
    request: TransportRequest<'_>,
    output: &mut [u8],
) -> Result<CapturedResponse, TransportError> {
    let capacity = output.len();
    let mut response = ResponseBuffer::new(output, capacity);
    BlockingTransport::send(client, request, response.writer())?;
    response
        .with_response(CapturedResponse::capture)
        .map_err(|_| TransportError::ResponseCommitFailed)
}
