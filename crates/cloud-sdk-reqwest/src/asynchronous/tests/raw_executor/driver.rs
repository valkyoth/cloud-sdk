use cloud_sdk::transport::{
    AsyncExecutionError, RawResponsePolicy, ResponseWriter, TransportFailure, TransportRequest,
    drive_async_raw,
};

use super::RawAsyncClient;
use crate::asynchronous::RawHttpError;

pub(crate) trait RawAsyncTestExt {
    async fn execute_checked<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        request: TransportRequest<'request>,
        policy: RawResponsePolicy<'policy>,
        response: &'writer mut ResponseWriter<'buffer>,
    ) -> Result<(), crate::shared::RawTransportFailure>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer;
}

impl RawAsyncTestExt for RawAsyncClient {
    async fn execute_checked<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        request: TransportRequest<'request>,
        policy: RawResponsePolicy<'policy>,
        response: &'writer mut ResponseWriter<'buffer>,
    ) -> Result<(), crate::shared::RawTransportFailure>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        drive_async_raw(self, request, policy, response)
            .await
            .map_err(|error| match error {
                AsyncExecutionError::Transport(error) => error,
                AsyncExecutionError::Response(error) => TransportFailure::not_sent(match error {
                    cloud_sdk::transport::ResponseWriterError::AlreadyCommitted => {
                        RawHttpError::ResponseAlreadyCommitted
                    }
                    _ => RawHttpError::ResponseCommitFailed,
                }),
            })
    }
}
