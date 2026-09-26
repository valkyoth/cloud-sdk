use super::{RawBlockingClient, RawHttpError, RawTransportFailure, RawUpload};
use cloud_sdk::transport::{
    BlockingStreamSource, RawResponsePolicy, ResponseWriter, TransportFailure, TransportRequest,
};

impl RawBlockingClient {
    /// Executes one finite, declared-length authenticated upload without
    /// buffering the whole body, redirects or retries. Partial remote writes
    /// cannot be rolled back. The caller's synchronous source must not block
    /// indefinitely: an executor timeout cannot preempt caller code.
    pub fn execute_upload<S: BlockingStreamSource>(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        upload: RawUpload<'_, S>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), RawTransportFailure> {
        let mut attempt = response
            .begin_attempt()
            .map_err(|_| TransportFailure::not_sent(RawHttpError::ResponseAlreadyCommitted))?;
        let runtime = super::runtime::Runtime::new()?;
        let completion = runtime.block_on(self.inner.execute_upload_blocking(
            request,
            policy,
            upload,
            &mut attempt,
        ))??;
        let status = completion.status();
        attempt.commit_completion(completion).map_err(|_| {
            TransportFailure::response_started_with_status(
                status,
                RawHttpError::ResponseCommitFailed,
            )
        })
    }
}

impl cloud_sdk::transport::BlockingRawUploadExecutor for RawBlockingClient {
    fn upload<S: BlockingStreamSource>(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        upload: cloud_sdk::transport::AuthorizedUpload<'_, S>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), RawTransportFailure> {
        self.execute_upload(
            request,
            policy,
            RawUpload::new(
                upload.expected,
                upload.authorization,
                upload.source,
                upload.policy,
                upload.scratch,
            ),
            response,
        )
    }
}
