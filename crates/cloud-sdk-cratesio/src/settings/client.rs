use super::{SettingsError as Error, SettingsPermit, SettingsResponse};
use crate::{
    credentials::{CredentialContext, CredentialOrigin, ScopedCredentialMaterial},
    discovery::{DiscoveryClient, DiscoveryExecutionError as Failure},
    query::MAX_TARGET_BYTES,
    wire::IdentifyingUserAgent,
};
use cloud_sdk::transport::{
    BoundTransport, BoundUserAgent, MediaType, RawResponsePolicy, RequestHeader, RequestHeaders,
    ResponseBuffer, ResponseWriter, TransportRequest,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

/// Caller storage cleared on every exit. Credential inputs must be separate.
pub struct SettingsBuffers<'a> {
    /// Authorization scratch.
    pub credential: &'a mut [u8],
    /// Bounded JSON request scratch.
    pub body: &'a mut [u8],
    /// Bounded response scratch.
    pub response: &'a mut [u8],
    /// Retained response headers.
    pub headers: &'a mut [u8],
}
/// Single-attempt blocking mutation runner over an explicitly trusted adapter.
pub struct SettingsClient<'a, T: ?Sized>(DiscoveryClient<'a, T>);
impl<T: ?Sized> core::fmt::Debug for SettingsClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SettingsClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> SettingsClient<'a, T> {
    /// Fixed production authority with identifying user agent.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Fixed staging authority; production tokens cannot be sent here.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
    /// Consumes explicit consent, sends once, and checks exact response postconditions.
    /// The trusted callback must enforce the raw policy, add sensitive Authorization
    /// exactly once to the bound executor, and disable redirects, cookies and retries.
    /// It must not log/copy credentials. An error can follow an upstream mutation.
    ///
    /// ```compile_fail
    /// use cloud_sdk_cratesio::settings::*;
    /// use cloud_sdk::transport::{BoundTransport, BoundUserAgent};
    /// fn replay<T: BoundTransport + BoundUserAgent>(c: &SettingsClient<'_, T>, p: SettingsPermit<'_>, a: SettingsBuffers<'_>, b: SettingsBuffers<'_>) {
    ///     let _ = c.execute(p, a, |_, _, _, _, _| Ok::<_, ()>(()));
    ///     let _ = c.execute(p, b, |_, _, _, _, _| Ok::<_, ()>(()));
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use cloud_sdk_cratesio::settings::*;
    /// use cloud_sdk::transport::{BoundTransport, BoundUserAgent};
    /// fn unconfirmed<T: BoundTransport + BoundUserAgent>(c: &SettingsClient<'_, T>, r: SettingsRequest<'_>, b: SettingsBuffers<'_>) {
    ///     let _ = c.execute(r, b, |_, _, _, _, _| Ok::<_, ()>(()));
    /// }
    /// ```
    pub fn execute<E>(
        &self,
        permit: SettingsPermit<'_>,
        buffers: SettingsBuffers<'_>,
        send: impl for<'r, 'p, 'w, 'b> FnOnce(
            &T,
            ScopedCredentialMaterial<'r>,
            TransportRequest<'r>,
            RawResponsePolicy<'p>,
            &'w mut ResponseWriter<'b>,
        ) -> Result<(), E>,
    ) -> Result<SettingsResponse, Failure<E>> {
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
        if permit.credential.origin() != origin {
            return Err(Failure::Model(Error::Binding));
        }
        let length = permit
            .request
            .body(body.as_mut_slice())
            .map_err(Failure::Model)?;
        let payload = body
            .as_slice()
            .get(..length)
            .ok_or(Failure::Model(Error::Limit))?;
        let mut target = [0; MAX_TARGET_BYTES];
        let context = CredentialContext::api(
            origin,
            permit.operation().method(),
            permit
                .request
                .write_target(&mut target)
                .map_err(|_| Failure::Model(Error::Binding))?,
        )
        .map_err(|_| Failure::Model(Error::Binding))?;
        let headers = [
            RequestHeader::accept(MediaType::JSON),
            RequestHeader::new("accept-encoding", "identity")
                .map_err(|_| Failure::Model(Error::Value))?,
            RequestHeader::new("content-type", "application/json")
                .map_err(|_| Failure::Model(Error::Value))?,
        ];
        let headers = RequestHeaders::new(&headers).map_err(|_| Failure::Model(Error::Value))?;
        let policy = self.0.policy().map_err(Failure::Model)?;
        let mut attempt = self.0.gate.begin().map_err(Failure::Schedule)?;
        permit
            .credential
            .with_material_for_adapter(
                &context,
                self.0.executor,
                secret.as_mut_slice(),
                |executor, material| {
                    let request = TransportRequest::new(material.method(), material.target())
                        .with_body(payload)
                        .with_headers(headers);
                    send(executor, material, request, policy, response.writer())
                },
            )
            .map_err(|_| Failure::Model(Error::Binding))?
            .map_err(Failure::Transport)?;
        permit
            .decode_response(self.0.admit(response, &mut attempt)?)
            .map_err(Failure::Model)
    }
}
