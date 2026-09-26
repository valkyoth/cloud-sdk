use super::{RawAsyncClient, RawHttpError, RawTransportFailure, RawUpload};
use cloud_sdk::transport::{
    AsyncStreamSource, LocalAsyncStreamSource, RawResponsePolicy, ResponseWriter, TransportFailure,
    TransportRequest,
};
use core::future::Future;

macro_rules! execute {
    ($name:ident, $inner:ident, $source:path $(, $send:ident)?) => {
        /// Executes one declared-length authenticated streaming upload. No
        /// redirect, retry, cookie or whole-body buffering is performed.
        /// Response commitment requires complete source framing and a checked
        /// response. Partial remote writes cannot be rolled back. Caller source
        /// and authorization storage are not owned or erased by the adapter.
        pub fn $name<'a, 'buffer: 'a, S: $source $(+ $send)? + 'a>(
            &'a self, request: TransportRequest<'a>, policy: RawResponsePolicy<'a>,
            upload: RawUpload<'a, S>, response: &'a mut ResponseWriter<'buffer>,
        ) -> impl Future<Output = Result<(), RawTransportFailure>> $(+ $send)? + 'a {
            let attempt = response.begin_attempt();
            async move {
                let mut attempt = attempt.map_err(|_| TransportFailure::not_sent(RawHttpError::ResponseAlreadyCommitted))?;
                let completion = self.inner.$inner(request, policy, upload, &mut attempt).await?;
                let status = completion.status();
                attempt.commit_completion(completion).map_err(|_| TransportFailure::response_started_with_status(status, RawHttpError::ResponseCommitFailed))
            }
        }
    }
}
impl RawAsyncClient {
    execute!(execute_upload, execute_upload, AsyncStreamSource, Send);
    execute!(
        execute_upload_local,
        execute_upload_local,
        LocalAsyncStreamSource
    );
}

impl cloud_sdk::transport::AsyncRawUploadExecutor for RawAsyncClient {
    fn upload<'a, 'b: 'a, S: AsyncStreamSource + Send + 'a>(
        &'a self,
        request: TransportRequest<'a>,
        policy: RawResponsePolicy<'a>,
        upload: cloud_sdk::transport::AuthorizedUpload<'a, S>,
        response: &'a mut ResponseWriter<'b>,
    ) -> impl Future<Output = Result<(), RawTransportFailure>> + Send + 'a {
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
impl cloud_sdk::transport::LocalRawUploadExecutor for RawAsyncClient {
    fn upload_local<'a, 'b: 'a, S: LocalAsyncStreamSource + 'a>(
        &'a self,
        request: TransportRequest<'a>,
        policy: RawResponsePolicy<'a>,
        upload: cloud_sdk::transport::AuthorizedUpload<'a, S>,
        response: &'a mut ResponseWriter<'b>,
    ) -> impl Future<Output = Result<(), RawTransportFailure>> + 'a {
        self.execute_upload_local(
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
