use super::{PublishBuffers, PublishClient, PublishPermit, PublishResponse};
use crate::discovery::DiscoveryExecutionError;
use cloud_sdk::transport::{BlockingStreamSource, TransportFailure};
use cloud_sdk_reqwest::blocking::{
    RawBlockingClient, RawHttpError, RawTransportFailure, RawUpload,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

impl PublishClient<'_, RawBlockingClient> {
    /// Publishes one explicitly authorized package through the bundled live
    /// HTTP upload. Metadata and the archive are streamed in Cargo framing,
    /// never concatenated into a whole-body allocation. No callback, retry,
    /// redirect, implicit packaging or polling is involved. An error can follow
    /// a remote side effect; do not automatically create a replacement permit.
    /// All supplied scratch clears, including preflight failures.
    pub fn execute_bundled<S: BlockingStreamSource>(
        &self,
        permit: PublishPermit<'_>,
        source: &mut S,
        buffers: PublishBuffers<'_>,
        upload_scratch: &mut [u8],
    ) -> Result<PublishResponse, DiscoveryExecutionError<RawTransportFailure>> {
        sanitize_bytes(upload_scratch);
        let mut scratch = SecretBuffer::new(upload_scratch);
        self.execute(
            permit,
            source,
            buffers,
            |executor, material, request, upload, policy, writer| {
                let authorization = material
                    .authorization()
                    .ok_or_else(|| TransportFailure::not_sent(RawHttpError::HeaderRejected))?;
                if material.method() != request.method()
                    || material.target() != request.target()
                    || material.json_body().is_some()
                {
                    return Err(TransportFailure::not_sent(RawHttpError::TargetRejected));
                }
                let (mut source, stream_policy) = upload
                    .take_framed()
                    .map_err(|_| TransportFailure::not_sent(RawHttpError::InvalidStreamState))?;
                executor.execute_upload(
                    request,
                    policy,
                    RawUpload::new(
                        material.endpoint(),
                        authorization,
                        &mut source,
                        stream_policy,
                        scratch.as_mut_slice(),
                    ),
                    writer,
                )?;
                upload.complete = true;
                Ok(())
            },
        )
    }
}
