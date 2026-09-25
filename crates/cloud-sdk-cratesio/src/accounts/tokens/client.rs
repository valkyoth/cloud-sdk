use super::{TokenOperation, TokenPermit, TokenResponse};
use crate::{
    credentials::{CredentialContext, CredentialOrigin, ScopedCredentialMaterial},
    discovery::{DiscoveryClient, DiscoveryError as Error, DiscoveryExecutionError as Failure},
    query::MAX_TARGET_BYTES,
    wire::{CratesIoWireError, IdentifyingUserAgent},
};
use cloud_sdk::{
    rate_limit::{RetryAfter, WallClockTimestamp},
    transport::{
        BoundTransport, BoundUserAgent, HeaderName, MediaType, RawResponsePolicy, RequestHeaders,
        ResponseBuffer, ResponseMediaPolicy, ResponseWriter, TransportRequest,
    },
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

/// Caller-owned storage, cleared on every exit including rejected admission.
pub struct TokenBuffers<'a> {
    /// Authorization scratch, never a borrowed credential input.
    pub credential: &'a mut [u8],
    /// Bounded response storage.
    pub response: &'a mut [u8],
    /// Response header storage.
    pub headers: &'a mut [u8],
}
/// Single-attempt token execution through an explicitly trusted blocking adapter.
pub struct TokenClient<'a, T: ?Sized>(DiscoveryClient<'a, T>);
impl<T: ?Sized> core::fmt::Debug for TokenClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("TokenClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> TokenClient<'a, T> {
    /// Fixed production origin and explicit identifying user agent.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Fixed staging origin; production credentials are refused before sending.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, Error> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
    /// Consumes exact caller authority and executes at most once. The trusted
    /// adapter must send the supplied request with sensitive Authorization to the
    /// bound executor, enforce the raw policy, and disable cookies, redirects and
    /// retries. It must not retain/log credentials, private IDs or response bodies.
    /// An error may follow a successful upstream revocation: reconcile out of band.
    ///
    /// ```compile_fail
    /// use cloud_sdk_cratesio::{accounts::tokens::*, credentials::ApiToken};
    /// use cloud_sdk::transport::{BoundTransport, BoundUserAgent};
    /// fn without_confirmation<T: BoundTransport + BoundUserAgent>(
    ///     c: &TokenClient<'_, T>, token: &ApiToken, b: TokenBuffers<'_>) {
    ///     c.execute(token, b, |_, _, _, _, _| Ok::<_, ()>(()));
    /// }
    /// ```
    /// ```compile_fail
    /// use cloud_sdk_cratesio::accounts::tokens::*;
    /// use cloud_sdk::transport::{BoundTransport, BoundUserAgent};
    /// fn replay<T: BoundTransport + BoundUserAgent>(c: &TokenClient<'_, T>,
    ///     p: TokenPermit<'_>, first: TokenBuffers<'_>, second: TokenBuffers<'_>) {
    ///     let _ = c.execute(p, first, |_, _, _, _, _| Ok::<_, ()>(()));
    ///     let _ = c.execute(p, second, |_, _, _, _, _| Ok::<_, ()>(()));
    /// }
    /// ```
    pub fn execute<E>(
        &self,
        permit: TokenPermit<'_>,
        buffers: TokenBuffers<'_>,
        send: impl for<'r, 'p, 'w, 'b> FnOnce(
            &T,
            ScopedCredentialMaterial<'r>,
            TransportRequest<'r>,
            RawResponsePolicy<'p>,
            &'w mut ResponseWriter<'b>,
        ) -> Result<(), E>,
    ) -> Result<TokenResponse, Failure<E>> {
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
        if permit.origin() != origin {
            return Err(Failure::Model(Error::Binding));
        }
        let mut target = [0; MAX_TARGET_BYTES];
        let context = CredentialContext::api(
            origin,
            permit.operation.method(),
            permit
                .write_target(&mut target)
                .map_err(|_| Failure::Model(Error::Binding))?,
        )
        .map_err(|_| Failure::Model(Error::Binding))?;
        let headers = self.0.headers().map_err(Failure::Model)?;
        let headers = RequestHeaders::new(&headers).map_err(|_| Failure::Model(Error::Value))?;
        let empty = permit.operation == TokenOperation::RevokeCurrent;
        let policy = if empty {
            RawResponsePolicy::new(
                0,
                self.0.maximum,
                ResponseMediaPolicy::Forbidden,
                ResponseMediaPolicy::Required(&[MediaType::JSON]),
                &[HeaderName::new("retry-after").map_err(|_| Failure::Model(Error::Value))?],
                8,
            )
            .map_err(|_| Failure::Model(Error::Value))?
        } else {
            self.0.policy().map_err(Failure::Model)?
        };
        let mut attempt = self.0.gate.begin().map_err(Failure::Schedule)?;
        permit
            .credential
            .with_material_for_adapter(
                &context,
                self.0.executor,
                secret.as_mut_slice(),
                |executor, material| {
                    let wire = TransportRequest::new(material.method(), material.target())
                        .with_headers(headers);
                    send(executor, material, wire, policy, response.writer())
                },
            )
            .map_err(|_| Failure::Model(Error::Binding))?
            .map_err(Failure::Transport)?;
        let is_empty_success = response
            .with_response(|r| r.status().get() == 204)
            .map_err(|_| Failure::Staging)?;
        if empty && is_empty_success {
            let now = self.0.response_time_seconds().map_err(Failure::Model)?;
            let delay = response
                .with_response(|r| {
                    if !r.body().is_empty() {
                        return Err(CratesIoWireError::UnexpectedStatus);
                    }
                    if r.headers().get("content-type").is_some()
                        || r.headers().get("content-encoding").is_some()
                    {
                        return Err(CratesIoWireError::ContentType);
                    }
                    r.headers()
                        .get("retry-after")
                        .map(|h| {
                            RetryAfter::parse(h.value(), WallClockTimestamp::new(now))
                                .map_err(|_| CratesIoWireError::RetryAfter)
                        })
                        .transpose()
                })
                .map_err(|_| Failure::Staging)?
                .map_err(Failure::Wire)?;
            self.0
                .defer_response_delay(&mut attempt, delay, now)
                .map_err(Failure::Schedule)?;
            Ok(TokenResponse::CurrentRevoked)
        } else {
            permit
                .decode_response(self.0.admit(response, &mut attempt)?)
                .map_err(Failure::Model)
        }
    }
}
