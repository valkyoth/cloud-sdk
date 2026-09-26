use super::{
    TrustedPublishingError as Error, TrustedPublishingPermit, TrustedPublishingResponse,
    request::Intent,
};
use crate::{
    credentials::{CredentialContext, ScopedCredentialMaterial},
    discovery::{DiscoveryClient, DiscoveryExecutionError as Failure},
    query::MAX_TARGET_BYTES,
    wire::IdentifyingUserAgent,
};
use cloud_sdk::transport::{
    BoundTransport, BoundUserAgent, MediaType, RawResponsePolicy, RequestHeader, RequestHeaders,
    ResponseBuffer, ResponseWriter, TransportRequest,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

/// Caller storage erased on every exit, including rejected preflight.
pub struct TrustedPublishingBuffers<'a> {
    /// Sensitive authorization or assertion JSON scratch.
    pub credential: &'a mut [u8],
    /// Configuration JSON scratch.
    pub body: &'a mut [u8],
    /// Bounded response bytes.
    pub response: &'a mut [u8],
    /// Retained response header bytes.
    pub headers: &'a mut [u8],
}
/// One-shot execution with fixed origin and shared process admission.
pub struct TrustedPublishingClient<'a, T: ?Sized>(DiscoveryClient<'a, T>);
impl<T: ?Sized> core::fmt::Debug for TrustedPublishingClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("TrustedPublishingClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> TrustedPublishingClient<'a, T> {
    /// Bind to production and an explicit identifying user agent.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Bind to staging. Production credentials are rejected before sending.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
    /// Consume authority and call the trusted adapter at most once. The adapter
    /// must enforce the supplied raw policy, verified TLS, deadlines and bound
    /// origin, with no cookies, redirects or retries. Send sensitive Authorization
    /// only when material supplies it; exchange assertions go only in the JSON
    /// body. Never retain/log either. Failure can follow a committed mutation;
    /// reconcile out of band instead of automatically replaying the request.
    pub fn execute<E>(
        &self,
        permit: TrustedPublishingPermit<'_>,
        buffers: TrustedPublishingBuffers<'_>,
        send: impl for<'r, 'p, 'w, 'b> FnOnce(
            &T,
            ScopedCredentialMaterial<'r>,
            TransportRequest<'r>,
            RawResponsePolicy<'p>,
            &'w mut ResponseWriter<'b>,
        ) -> Result<(), E>,
    ) -> Result<TrustedPublishingResponse, Failure<E>> {
        let mut secret = SecretBuffer::new(buffers.credential);
        let mut body = SecretBuffer::new(buffers.body);
        sanitize_bytes(secret.as_mut_slice());
        sanitize_bytes(body.as_mut_slice());
        let mut response = ResponseBuffer::new(buffers.response, self.0.maximum, buffers.headers);
        self.0.verify().map_err(Failure::Model)?;
        let origin = permit.origin();
        if origin.endpoint() != self.0.endpoint {
            return Err(Failure::Model(Error::Binding));
        }
        let mut target = [0; MAX_TARGET_BYTES];
        let target = permit
            .write_target(&mut target)
            .map_err(|_| Failure::Model(Error::Binding))?;
        let length = permit.body(body.as_mut_slice()).map_err(Failure::Model)?;
        let has_body = matches!(permit.0, Intent::Create(..) | Intent::Exchange(..));
        let headers = [
            RequestHeader::accept(MediaType::JSON),
            RequestHeader::new("accept-encoding", "identity")
                .map_err(|_| Failure::Model(Error::Value))?,
            RequestHeader::new("content-type", "application/json")
                .map_err(|_| Failure::Model(Error::Value))?,
        ];
        let headers = RequestHeaders::new(if has_body { &headers } else { &headers[..2] })
            .map_err(|_| Failure::Model(Error::Value))?;
        let empty = permit.operation().empty();
        let policy = if empty {
            super::empty::policy(self.0.maximum)
        } else {
            self.0.policy()
        }
        .map_err(Failure::Model)?;
        let mut attempt = self.0.gate.begin().map_err(Failure::Schedule)?;
        let apply = |executor: &T, material: ScopedCredentialMaterial<'_>| {
            if let Intent::Exchange(_, policy) = &permit.0 {
                policy
                    .preflight(
                        material.json_body().ok_or(Failure::Model(Error::Binding))?,
                        self.0.response_time_seconds().map_err(Failure::Model)?,
                    )
                    .map_err(Failure::Model)?;
            }
            let bytes = material.json_body().unwrap_or(
                body.as_slice()
                    .get(..length)
                    .ok_or(Failure::Model(Error::Limit))?,
            );
            let request = TransportRequest::new(material.method(), material.target())
                .with_headers(headers)
                .with_body(bytes);
            send(executor, material, request, policy, response.writer()).map_err(Failure::Transport)
        };
        match &permit.0 {
            Intent::List(_, _, token) | Intent::Create(_, token) | Intent::Delete(_, _, token) => {
                token.with_material_for_adapter(
                    &CredentialContext::api(origin, permit.operation().method(), target)
                        .map_err(|_| Failure::Model(Error::Binding))?,
                    self.0.executor,
                    secret.as_mut_slice(),
                    apply,
                )
            }
            Intent::Exchange(token, _) => token.with_material_for_adapter(
                &CredentialContext::exchange(origin),
                self.0.executor,
                secret.as_mut_slice(),
                apply,
            ),
            Intent::Revoke(token) => token.credential.with_material_for_adapter(
                &CredentialContext::revoke(origin),
                self.0.executor,
                secret.as_mut_slice(),
                apply,
            ),
        }
        .map_err(|_| Failure::Model(Error::Binding))??;
        let now = self.0.response_time_seconds().map_err(Failure::Model)?;
        if empty
            && response
                .with_response(|r| r.status().get() == 204)
                .map_err(|_| Failure::Staging)?
        {
            let delay = super::empty::admit(&response, now)?;
            self.0
                .defer_response_delay(&mut attempt, delay, now)
                .map_err(Failure::Schedule)?;
            return Ok(if matches!(permit.0, Intent::Revoke(_)) {
                TrustedPublishingResponse::Revoked
            } else {
                TrustedPublishingResponse::Deleted
            });
        }
        permit
            .decode_response(self.0.admit(response, &mut attempt)?, now)
            .map_err(Failure::Model)
    }
}
