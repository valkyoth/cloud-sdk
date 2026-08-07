use super::CheckedResponseGuard;

impl CheckedResponseGuard<'_> {
    pub(crate) const fn response_headers(&self) -> &crate::transport::ResponseHeaders<'_> {
        self.writer.headers()
    }
}
