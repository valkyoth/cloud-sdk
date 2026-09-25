use super::{
    PublishError as Error, PublishPermit, PublishResponse, PublishUpload,
    request::PublishCredential,
};
use crate::{
    credentials::{CredentialContext, ScopedCredentialMaterial},
    discovery::{DiscoveryClient, DiscoveryExecutionError as Failure},
    endpoint::ApiRequestTarget,
    wire::IdentifyingUserAgent,
};
use cloud_sdk::{
    Method,
    transport::{
        BlockingStreamSource, BoundTransport, BoundUserAgent, MediaType, RawResponsePolicy,
        RequestHeader, RequestHeaders, ResponseBuffer, ResponseWriter, TransportRequest,
    },
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

/// Storage cleared on every exit. Source metadata/archive storage is caller-owned.
pub struct PublishBuffers<'a> {
    /// Authorization storage.
    pub credential: &'a mut [u8],
    /// Bounded response bytes.
    pub response: &'a mut [u8],
    /// Retained response headers.
    pub headers: &'a mut [u8],
}
/// One-shot official-origin publisher over an explicitly trusted streaming callback.
pub struct PublishClient<'a, T: ?Sized>(DiscoveryClient<'a, T>);
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> PublishClient<'a, T> {
    /// Fixed production origin and shared admission gate.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Fixed staging origin; production credentials are rejected before dispatch.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
    /// Consume explicit authority. The trusted adapter must open exactly one
    /// PUT with sensitive Authorization, octet-stream and upload.content_length(),
    /// call transfer_to once on that connection's direct sink, then commit the
    /// response. Disable redirects/cookies/retries and enforce TLS/deadlines.
    /// TransportRequest's body is empty because the upload is streamed separately.
    /// Acknowledgement or transport failure never permits replay.
    ///
    /// ```compile_fail
    /// use cloud_sdk_cratesio::publishing::*;
    /// fn duplicate(p: PublishPermit<'_>) { let a = p; let b = p; }
    /// ```
    pub fn execute<S: BlockingStreamSource, E>(
        &self,
        permit: PublishPermit<'_>,
        source: &mut S,
        buffers: PublishBuffers<'_>,
        send: impl for<'r, 'p, 'w, 'b, 's> FnOnce(
            &T,
            ScopedCredentialMaterial<'r>,
            TransportRequest<'r>,
            &mut PublishUpload<'s, S>,
            RawResponsePolicy<'p>,
            &'w mut ResponseWriter<'b>,
        ) -> Result<(), E>,
    ) -> Result<PublishResponse, Failure<E>> {
        let mut secret = SecretBuffer::new(buffers.credential);
        sanitize_bytes(secret.as_mut_slice());
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
        let policy = self.0.policy().map_err(Failure::Model)?;
        let mut attempt = self.0.gate.begin().map_err(Failure::Schedule)?;
        let mut upload = PublishUpload::new(&permit.request, source).map_err(Failure::Model)?;
        let apply = |executor: &T, material: ScopedCredentialMaterial<'_>| {
            send(
                executor,
                material,
                request,
                &mut upload,
                policy,
                response.writer(),
            )
        };
        match &permit.credential {
            PublishCredential::Api(token) => token.with_material_for_adapter(
                &CredentialContext::api(permit.origin(), Method::Put, target)
                    .map_err(|_| Failure::Model(Error::Binding))?,
                self.0.executor,
                secret.as_mut_slice(),
                apply,
            ),
            PublishCredential::Trusted(token) => token.with_material_for_adapter(
                &CredentialContext::publish(permit.origin()),
                self.0.executor,
                secret.as_mut_slice(),
                apply,
            ),
        }
        .map_err(|_| Failure::Model(Error::Binding))?
        .map_err(Failure::Transport)?;
        if !upload.complete {
            return Err(Failure::Model(Error::Binding));
        }
        permit
            .decode_response(self.0.admit(response, &mut attempt)?)
            .map_err(Failure::Model)
    }
}
