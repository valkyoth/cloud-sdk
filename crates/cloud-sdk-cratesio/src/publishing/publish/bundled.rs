use super::{
    PublishBuffers, PublishClient, PublishError as Error, PublishPermit, PublishResponse,
    request::PublishCredential, stream::Framed,
};
use crate::{
    credentials::CredentialContext, discovery::DiscoveryExecutionError as Failure,
    endpoint::ApiRequestTarget,
};
use cloud_sdk::{
    Method,
    transport::{
        AuthorizedUpload, BlockingRawUploadExecutor, BlockingStreamSource, BoundUserAgent,
        MediaType, RequestHeader, RequestHeaders, ResponseBuffer, TransportRequest,
    },
};
use cloud_sdk_sanitization::SecretBuffer;

impl<T: BlockingRawUploadExecutor + BoundUserAgent + ?Sized> PublishClient<'_, T> {
    /// Streams Cargo framing through a trusted upload executor. The bundled
    /// adapter implements the same contract. No callback, implicit packaging,
    /// polling or retries. Errors may follow a committed remote mutation.
    pub fn execute_bundled<S: BlockingStreamSource>(
        &self,
        permit: PublishPermit<'_>,
        source: &mut S,
        buffers: PublishBuffers<'_>,
        upload_scratch: &mut [u8],
    ) -> Result<PublishResponse, Failure<T::Error>> {
        let mut credential = SecretBuffer::new(buffers.credential);
        let mut scratch = SecretBuffer::new(upload_scratch);
        let mut response = ResponseBuffer::new(buffers.response, self.0.maximum, buffers.headers);
        self.0.verify().map_err(Failure::Model)?;
        if permit.origin().endpoint() != self.0.endpoint {
            return Err(Failure::Model(Error::Binding));
        }
        let target = ApiRequestTarget::new("/api/v1/crates/new")
            .map_err(|_| Failure::Model(Error::Binding))?;
        let headers = [
            RequestHeader::accept(MediaType::JSON),
            RequestHeader::new("content-type", "application/octet-stream")
                .map_err(|_| Failure::Model(Error::Value))?,
            RequestHeader::new("accept-encoding", "identity")
                .map_err(|_| Failure::Model(Error::Value))?,
        ];
        let request = TransportRequest::new(Method::Put, target.as_request_target())
            .with_headers(RequestHeaders::new(&headers).map_err(|_| Failure::Model(Error::Value))?);
        let material = match &permit.credential {
            PublishCredential::Api(token) => token.stage_for_adapter(
                &CredentialContext::api(permit.origin(), Method::Put, target)
                    .map_err(|_| Failure::Model(Error::Binding))?,
                self.0.executor,
                &mut credential,
            ),
            PublishCredential::Trusted(token) => token.stage_for_adapter(
                &CredentialContext::publish(permit.origin()),
                self.0.executor,
                &mut credential,
            ),
        }
        .map_err(|_| Failure::Model(Error::Binding))?;
        let authorization = material
            .authorization()
            .ok_or(Failure::Model(Error::Binding))?;
        let policy = self.0.policy().map_err(Failure::Model)?;
        let mut source = Framed::new(&permit.request, source).map_err(Failure::Model)?;
        let upload = AuthorizedUpload {
            expected: material.endpoint(),
            authorization,
            source: &mut source,
            policy: permit.request.policy,
            scratch: scratch.as_mut_slice(),
        };
        let mut attempt = self.0.gate.begin().map_err(Failure::Schedule)?;
        self.0
            .executor
            .upload(request, policy, upload, response.writer())
            .map_err(Failure::Transport)?;
        permit
            .decode_response(self.0.admit(response, &mut attempt)?)
            .map_err(Failure::Model)
    }
}
