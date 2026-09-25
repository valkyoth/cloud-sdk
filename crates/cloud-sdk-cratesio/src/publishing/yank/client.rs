use super::{YankAcknowledgement, YankError as Error, YankPermit};
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

/// Caller-owned scratch cleared on every exit. Cargo yank requests have no body.
pub struct YankBuffers<'a> {
    /// Sensitive authorization scratch, separate from the source credential.
    pub credential: &'a mut [u8],
    /// Bounded response storage.
    pub response: &'a mut [u8],
    /// Retained response headers.
    pub headers: &'a mut [u8],
}
/// Single-attempt runner using the official shared admission gate and trusted adapter.
pub struct YankClient<'a, T: ?Sized>(DiscoveryClient<'a, T>);
impl<T: ?Sized> core::fmt::Debug for YankClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("YankClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> YankClient<'a, T> {
    /// Fixed production authority with identifying user agent.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Fixed staging authority; production tokens are rejected before dispatch.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
    /// Consume consent and send exactly once. The trusted callback must honor
    /// the raw policy, bind dispatch to the provided executor, attach sensitive
    /// Authorization exactly once, and disable redirects/cookies/retries.
    /// An error or timeout can follow a successfully applied mutation.
    ///
    /// ```compile_fail
    /// use cloud_sdk_cratesio::publishing::*;
    /// use cloud_sdk::transport::{BoundTransport, BoundUserAgent};
    /// fn replay<T: BoundTransport + BoundUserAgent>(c: &YankClient<'_, T>, p: YankPermit<'_>, a: YankBuffers<'_>, b: YankBuffers<'_>) {
    ///     let _ = c.execute(p, a, |_, _, _, _, _| Ok::<_, ()>(()));
    ///     let _ = c.execute(p, b, |_, _, _, _, _| Ok::<_, ()>(()));
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use cloud_sdk_cratesio::publishing::*;
    /// use cloud_sdk::transport::{BoundTransport, BoundUserAgent};
    /// fn unconfirmed<T: BoundTransport + BoundUserAgent>(c: &YankClient<'_, T>, r: YankRequest<'_>, b: YankBuffers<'_>) {
    ///     let _ = c.execute(r, b, |_, _, _, _, _| Ok::<_, ()>(()));
    /// }
    /// ```
    pub fn execute<'r, E>(
        &self,
        permit: YankPermit<'r>,
        buffers: YankBuffers<'_>,
        send: impl for<'s, 'p, 'w, 'b> FnOnce(
            &T,
            ScopedCredentialMaterial<'s>,
            TransportRequest<'s>,
            RawResponsePolicy<'p>,
            &'w mut ResponseWriter<'b>,
        ) -> Result<(), E>,
    ) -> Result<YankAcknowledgement<'r>, Failure<E>> {
        let mut secret = SecretBuffer::new(buffers.credential);
        sanitize_bytes(secret.as_mut_slice());
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
