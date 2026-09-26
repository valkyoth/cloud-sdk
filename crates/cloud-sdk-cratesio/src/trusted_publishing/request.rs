use super::{
    ExchangePolicy, Publisher, PublisherConfig, TemporaryToken, TrustedPublishingError as Error,
};
use crate::{
    credentials::{ApiToken, CredentialOrigin, OidcAssertion},
    endpoint::ApiRequestTarget,
    identifiers::NumericId,
    query::{ApiPath, FixedSegment as F, Parameter, PathSegment as P, Query, QueryError},
};
use cloud_sdk::{Method, buffer::encode_snapshot_bounded};

/// Complete source-locked operation selection; no automatic retries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustedPublishingOperation {
    /// Read configuration metadata, with explicit pagination.
    List(Publisher),
    /// Add authority to publish an existing crate.
    Create(Publisher),
    /// Destructively remove a configuration by provider-qualified ID.
    Delete(Publisher),
    /// Consume an OIDC assertion in a single exchange.
    Exchange,
    /// Revoke one owned temporary token, also clearing it locally on failure.
    Revoke,
}
impl TrustedPublishingOperation {
    /// Exact HTTP method.
    pub const fn method(self) -> Method {
        match self {
            Self::List(_) => Method::Get,
            Self::Create(_) | Self::Exchange => Method::Post,
            _ => Method::Delete,
        }
    }
    /// Neither reads nor mutations retry implicitly in this interface.
    pub const fn permits_automatic_retry(self) -> bool {
        false
    }
    #[cfg(feature = "blocking")]
    pub(super) const fn empty(self) -> bool {
        matches!(self, Self::Delete(_) | Self::Revoke)
    }
}
pub(super) enum Intent<'a> {
    List(Publisher, Query<'a>, &'a ApiToken),
    Create(PublisherConfig<'a>, &'a ApiToken),
    Delete(Publisher, NumericId, &'a ApiToken),
    Exchange(OidcAssertion, ExchangePolicy<'a>),
    Revoke(TemporaryToken),
}
/// Consumed exact authority; exchange/revoke additionally own their credential.
/// ```compile_fail
/// use cloud_sdk_cratesio::trusted_publishing::TrustedPublishingPermit;
/// fn duplicate(p: TrustedPublishingPermit<'_>) { let _ = p.clone(); }
/// ```
pub struct TrustedPublishingPermit<'a>(pub(super) Intent<'a>);
impl<'a> TrustedPublishingPermit<'a> {
    /// List exactly one crate or user filter; pages are seek-only upstream.
    pub fn list(
        publisher: Publisher,
        query: Query<'a>,
        credential: &'a ApiToken,
    ) -> Result<Self, Error> {
        if query.operation() != publisher.query()
            || query.page().is_some()
            || query
                .parameters()
                .iter()
                .filter(|p| matches!(p, Parameter::Crate(_) | Parameter::UserId(_)))
                .count()
                != 1
        {
            return Err(Error::Binding);
        }
        Ok(Self(Intent::List(publisher, query, credential)))
    }
    /// Explicit confirmation of new publication authority.
    pub const fn confirm_create(config: PublisherConfig<'a>, credential: &'a ApiToken) -> Self {
        Self(Intent::Create(config, credential))
    }
    /// Explicit destructive confirmation, bound to provider and exact positive ID.
    /// The wire route has no crate identity; the server verifies ownership.
    pub const fn confirm_delete(
        publisher: Publisher,
        id: NumericId,
        credential: &'a ApiToken,
    ) -> Self {
        Self(Intent::Delete(publisher, id, credential))
    }
    /// Consumes the assertion; preflight is performed immediately before dispatch.
    pub const fn confirm_exchange(assertion: OidcAssertion, policy: ExchangePolicy<'a>) -> Self {
        Self(Intent::Exchange(assertion, policy))
    }
    /// Consumes the token. Even ambiguous/failed revocation clears local storage.
    pub fn confirm_revoke(token: TemporaryToken) -> Self {
        Self(Intent::Revoke(token))
    }
    /// Exact operation, without disclosing configuration or credentials.
    pub const fn operation(&self) -> TrustedPublishingOperation {
        match &self.0 {
            Intent::List(p, ..) => TrustedPublishingOperation::List(*p),
            Intent::Create(c, _) => TrustedPublishingOperation::Create(c.publisher),
            Intent::Delete(p, ..) => TrustedPublishingOperation::Delete(*p),
            Intent::Exchange(..) => TrustedPublishingOperation::Exchange,
            Intent::Revoke(_) => TrustedPublishingOperation::Revoke,
        }
    }
    /// Fixed destination belonging to the credential.
    pub fn origin(&self) -> CredentialOrigin {
        match &self.0 {
            Intent::List(_, _, t) | Intent::Create(_, t) | Intent::Delete(_, _, t) => t.origin(),
            Intent::Exchange(t, _) => t.origin(),
            Intent::Revoke(t) => t.origin(),
        }
    }
    /// Atomic public route encoding. Credentials never enter the URL.
    pub fn write_target<'b>(
        &self,
        output: &'b mut [u8],
    ) -> Result<ApiRequestTarget<'b>, QueryError> {
        match &self.0 {
            Intent::List(p, q, _) => q.write_target(
                ApiPath::new(&[P::Fixed(F::TrustedPublishing), P::Fixed(p.segment())])?,
                output,
            ),
            Intent::Create(c, _) => ApiPath::new(&[
                P::Fixed(F::TrustedPublishing),
                P::Fixed(c.publisher.segment()),
            ])?
            .write(output),
            Intent::Delete(p, id, _) => ApiPath::new(&[
                P::Fixed(F::TrustedPublishing),
                P::Fixed(p.segment()),
                P::Id(*id),
            ])?
            .write(output),
            _ => {
                ApiPath::new(&[P::Fixed(F::TrustedPublishing), P::Fixed(F::Tokens)])?.write(output)
            }
        }
    }
    /// Inspect a configuration body in cleared scratch. Assertions are available
    /// only in the scoped credential adapter, never through this public method.
    pub fn with_configuration_body<R>(
        &self,
        output: &mut [u8],
        inspect: impl FnOnce(&[u8]) -> R,
    ) -> Result<R, Error> {
        let mut output = cloud_sdk_sanitization::SecretBuffer::new(output);
        cloud_sdk_sanitization::sanitize_bytes(output.as_mut_slice());
        let len = self.body(output.as_mut_slice())?;
        Ok(inspect(output.as_slice().get(..len).ok_or(Error::Limit)?))
    }
    pub(super) fn body(&self, output: &mut [u8]) -> Result<usize, Error> {
        match &self.0 {
            Intent::Create(c, _) => {
                encode_snapshot_bounded(*c, output, 4096, Error::Limit, |c, e| c.encode(e))
            }
            _ => Ok(0),
        }
    }
}
impl core::fmt::Debug for TrustedPublishingPermit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("TrustedPublishingPermit([redacted])")
    }
}
