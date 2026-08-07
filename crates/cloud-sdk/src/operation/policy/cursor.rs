use core::fmt;

use super::{CheckedResponse, CheckedResponseGuard};

impl<'body> CheckedResponse<'body> {
    /// Returns the transport-admitted response headers.
    #[must_use]
    pub const fn headers(&self) -> &crate::transport::ResponseHeaders<'body> {
        self.headers
    }
}

impl fmt::Debug for CheckedResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CheckedResponse")
            .field("status", &self.status())
            .field("body_len", &self.body().len())
            .field("body", &"[redacted]")
            .field("content_type", &self.content_type())
            .field("rate_limit", &self.rate_limit())
            .field("request_id", &"[redacted]")
            .finish()
    }
}

impl CheckedResponseGuard<'_> {
    pub(crate) const fn response_headers(&self) -> &crate::transport::ResponseHeaders<'_> {
        self.writer.headers()
    }
}
