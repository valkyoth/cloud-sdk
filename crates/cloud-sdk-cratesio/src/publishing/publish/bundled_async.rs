use super::{
    PublishBuffers, PublishClient, PublishError as Error, PublishPermit, PublishResponse,
    request::PublishCredential,
    stream::{Framed, asynchronous::LocalFramed},
};
use crate::{
    credentials::CredentialContext, discovery::DiscoveryExecutionError as Failure,
    endpoint::ApiRequestTarget,
};
use cloud_sdk::{
    Method,
    transport::{
        AsyncStreamSource, LocalAsyncStreamSource, MediaType, RequestHeader, RequestHeaders,
        ResponseBuffer, TransportRequest,
    },
};
use cloud_sdk_reqwest::asynchronous::{RawAsyncClient, RawTransportFailure, RawUpload};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};
use core::future::Future;

macro_rules! execute {
    ($name:ident, $source:path, $framed:ident, $execute:ident $(, $send:ident)?) => {
        /// Consumes one publish permit and streams Cargo framing through the
        /// bundled adapter. All scratch guards precede future creation. No
        /// implicit retry, packaging, callback or whole-archive buffering occurs.
        /// Source storage remains caller-owned; cancellation cannot undo bytes
        /// already sent or determine whether an upstream mutation committed.
        pub fn $name<'s, S: $source $(+ $send)? + 's>(
            &'s self, permit: PublishPermit<'s>, source: &'s mut S,
            buffers: PublishBuffers<'s>, upload_scratch: &'s mut [u8],
        ) -> impl Future<Output = Result<PublishResponse, Failure<RawTransportFailure>>> $(+ $send)? + 's {
            sanitize_bytes(buffers.credential);
            sanitize_bytes(upload_scratch);
            let mut credential = SecretBuffer::new(buffers.credential);
            let mut scratch = SecretBuffer::new(upload_scratch);
            let mut response = ResponseBuffer::new(buffers.response, self.0.maximum, buffers.headers);
            async move {
                self.0.verify().map_err(Failure::Model)?;
                if permit.origin().endpoint() != self.0.endpoint { return Err(Failure::Model(Error::Binding)); }
                let target = ApiRequestTarget::new("/api/v1/crates/new").map_err(|_| Failure::Model(Error::Binding))?;
                let headers = [RequestHeader::accept(MediaType::JSON),
                    RequestHeader::new("content-type", "application/octet-stream").map_err(|_| Failure::Model(Error::Value))?,
                    RequestHeader::new("accept-encoding", "identity").map_err(|_| Failure::Model(Error::Value))?];
                let request = TransportRequest::new(Method::Put, target.as_request_target())
                    .with_headers(RequestHeaders::new(&headers).map_err(|_| Failure::Model(Error::Value))?);
                let material = match &permit.credential {
                    PublishCredential::Api(token) => token.stage_for_adapter(&CredentialContext::api(permit.origin(), Method::Put, target).map_err(|_| Failure::Model(Error::Binding))?, self.0.executor, &mut credential),
                    PublishCredential::Trusted(token) => token.stage_for_adapter(&CredentialContext::publish(permit.origin()), self.0.executor, &mut credential),
                }.map_err(|_| Failure::Model(Error::Binding))?;
                let authorization = material.authorization().ok_or(Failure::Model(Error::Binding))?;
                let policy = self.0.policy().map_err(Failure::Model)?;
                let mut source = $framed::new(&permit.request, source).map_err(Failure::Model)?;
                let upload = RawUpload::new(material.endpoint(), authorization, &mut source, permit.request.policy, scratch.as_mut_slice());
                let mut attempt = self.0.gate.begin().map_err(Failure::Schedule)?;
                self.0.executor.$execute(request, policy, upload, response.writer()).await.map_err(Failure::Transport)?;
                permit.decode_response(self.0.admit(response, &mut attempt)?).map_err(Failure::Model)
            }
        }
    }
}
impl PublishClient<'_, RawAsyncClient> {
    execute!(
        execute_bundled_async,
        AsyncStreamSource,
        Framed,
        execute_upload,
        Send
    );
    execute!(
        execute_bundled_local,
        LocalAsyncStreamSource,
        LocalFramed,
        execute_upload_local
    );
}
