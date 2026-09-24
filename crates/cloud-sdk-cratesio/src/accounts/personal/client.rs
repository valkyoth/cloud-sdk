use super::{PersonalPermit, PersonalResponse, permit::Authority};
use crate::{
    credentials::{CredentialContext, CredentialOrigin, ScopedCredentialMaterial},
    discovery::{DiscoveryClient, DiscoveryError as Error, DiscoveryExecutionError as Failure},
    query::MAX_TARGET_BYTES,
    wire::IdentifyingUserAgent,
};
use cloud_sdk::transport::{
    BoundTransport, BoundUserAgent, MediaType, RawResponsePolicy, RequestHeader, RequestHeaders,
    ResponseBuffer, ResponseWriter, TransportRequest,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

/// Caller-owned scratch, all cleared on every execution exit, including failures
/// before admission and unwinding. Do not reuse the input email as scratch.
pub struct PersonalBuffers<'a> {
    /// Authorization or secret path material.
    pub credential: &'a mut [u8],
    /// Serialized request body (up to MAX_PERSONAL_BODY_BYTES).
    pub body: &'a mut [u8],
    /// Bounded response body.
    pub response: &'a mut [u8],
    /// Response header storage.
    pub headers: &'a mut [u8],
}
/// One-attempt personal workflow runner over an explicitly trusted blocking
/// credential adapter. Uses the same process-wide API gate as anonymous clients.
pub struct PersonalClient<'a, T: ?Sized>(DiscoveryClient<'a, T>);
impl<T: ?Sized> core::fmt::Debug for PersonalClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PersonalClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> PersonalClient<'a, T> {
    /// Fixed production API and identifying user-agent binding.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Fixed staging API. Production credentials are rejected before dispatch.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
    /// Consumes one explicit action permit. The trusted callback must send exactly
    /// the request to the bound executor, add the material's Authorization once as
    /// sensitive (or omit it for path tokens), enforce the raw response policy,
    /// and disable redirects, cookies and retries. It must never log targets or
    /// copy secrets to unprotected storage. No SDK-supplied async token adapter is
    /// claimed by this API. Callbacks cannot return a future borrowing the secret.
    ///
    /// Transport failures may follow an upstream mutation. No retry is attempted.
    pub fn execute<E>(
        &self,
        permit: PersonalPermit<'_>,
        buffers: PersonalBuffers<'_>,
        send: impl for<'r, 'p, 'w, 'b> FnOnce(
            &T,
            ScopedCredentialMaterial<'r>,
            TransportRequest<'r>,
            RawResponsePolicy<'p>,
            &'w mut ResponseWriter<'b>,
        ) -> Result<(), E>,
    ) -> Result<PersonalResponse, Failure<E>> {
        let mut secret = SecretBuffer::new(buffers.credential);
        let mut body = SecretBuffer::new(buffers.body);
        sanitize_bytes(secret.as_mut_slice());
        sanitize_bytes(body.as_mut_slice());
        let mut response = ResponseBuffer::new(buffers.response, self.0.maximum, buffers.headers);
        self.0.verify().map_err(Failure::Model)?;
        let origin =
            if self.0.endpoint == crate::endpoint::OfficialCratesIoEndpoint::production_api() {
                CredentialOrigin::Production
            } else {
                CredentialOrigin::Staging
            };
        let credential_origin = match &permit.0 {
            Authority::Api(_, token) => token.origin(),
            Authority::Email(token) => token.origin(),
            Authority::Invitation(token, _) => token.origin(),
        };
        if credential_origin != origin {
            return Err(Failure::Model(Error::Binding));
        }
        let length = match &permit.0 {
            Authority::Api(request, _) => {
                request.body(body.as_mut_slice()).map_err(Failure::Model)?
            }
            _ => 0,
        };
        let payload = body
            .as_slice()
            .get(..length)
            .ok_or(Failure::Model(Error::Limit))?;
        let headers = [
            RequestHeader::accept(MediaType::JSON),
            RequestHeader::new("accept-encoding", "identity")
                .map_err(|_| Failure::Model(Error::Value))?,
            RequestHeader::new("content-type", "application/json")
                .map_err(|_| Failure::Model(Error::Value))?,
        ];
        let count = if payload.is_empty() { 2 } else { 3 };
        let headers =
            RequestHeaders::new(headers.get(..count).ok_or(Failure::Model(Error::Limit))?)
                .map_err(|_| Failure::Model(Error::Value))?;
        let policy = self.0.policy().map_err(Failure::Model)?;
        let mut target = [0; MAX_TARGET_BYTES];
        // Build/validate the credential context before occupying the shared gate.
        let api_context = match &permit.0 {
            Authority::Api(request, _) => Some(
                CredentialContext::api(
                    origin,
                    request.operation().method(),
                    request
                        .write_target(&mut target)
                        .map_err(|_| Failure::Model(Error::Binding))?,
                )
                .map_err(|_| Failure::Model(Error::Binding))?,
            ),
            _ => None,
        };
        let mut attempt = self.0.gate.begin().map_err(Failure::Schedule)?;
        let dispatch = |executor: &T, material: ScopedCredentialMaterial<'_>| {
            let wire = TransportRequest::new(material.method(), material.target())
                .with_body(payload)
                .with_headers(headers);
            send(executor, material, wire, policy, response.writer())
        };
        let sent = match &permit.0 {
            Authority::Api(_, token) => token.with_material_for_adapter(
                api_context.as_ref().ok_or(Failure::Model(Error::Binding))?,
                self.0.executor,
                secret.as_mut_slice(),
                dispatch,
            ),
            Authority::Email(token) => token.with_material_for_adapter(
                &CredentialContext::confirm_email(origin),
                self.0.executor,
                secret.as_mut_slice(),
                dispatch,
            ),
            Authority::Invitation(token, _) => token.with_material_for_adapter(
                &CredentialContext::accept_invitation(origin),
                self.0.executor,
                secret.as_mut_slice(),
                dispatch,
            ),
        };
        sent.map_err(|_| Failure::Model(Error::Binding))?
            .map_err(Failure::Transport)?;
        permit
            .decode_response(self.0.admit(response, &mut attempt)?)
            .map_err(Failure::Model)
    }
}
