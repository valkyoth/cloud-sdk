use crate::{
    discovery::{DiscoveryError as Error, DiscoveryValue},
    identifiers::NumericId,
};
use alloc::vec::Vec;

/// Reviewed endpoint scope names, not local mutation authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointScope {
    /// Publish a new crate.
    PublishNew,
    /// Publish an existing crate version.
    PublishUpdate,
    /// Manage trusted-publishing configuration.
    TrustedPublishing,
    /// Yank or unyank versions.
    Yank,
    /// Change crate owners.
    ChangeOwners,
}
impl EndpointScope {
    pub(super) fn parse(text: &str) -> Result<Self, Error> {
        match text {
            "publish-new" => Ok(Self::PublishNew),
            "publish-update" => Ok(Self::PublishUpdate),
            "trusted-publishing" => Ok(Self::TrustedPublishing),
            "yank" => Ok(Self::Yank),
            "change-owners" => Ok(Self::ChangeOwners),
            _ => Err(Error::Value),
        }
    }
}
/// Protected token metadata, never a token credential or authorization grant.
/// Lookup does not report revocation state or prove current usability.
pub struct TokenMetadata {
    pub(super) id: NumericId,
    pub(super) fields: DiscoveryValue,
    pub(super) endpoints: Option<Vec<EndpointScope>>,
}
impl core::fmt::Debug for TokenMetadata {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("TokenMetadata([redacted])")
    }
}
impl TokenMetadata {
    /// Exact ID matched to the requested token.
    pub const fn id(&self) -> NumericId {
        self.id
    }
    /// Scoped protected token-name access.
    pub fn with_name<R>(&self, inspect: impl FnOnce(&str) -> R) -> Result<R, Error> {
        self.fields.required("name")?.with_text(inspect)
    }
    /// None means upstream's legacy scope, not an empty permission set.
    pub fn endpoint_scopes(&self) -> Option<&[EndpointScope]> {
        self.endpoints.as_deref()
    }
    /// None means unrestricted crate scopes; Some(empty) remains distinct.
    /// Patterns are inert metadata, not interpreted as local authorization.
    pub fn crate_scopes(&self) -> Result<Option<&[DiscoveryValue]>, Error> {
        let value = self.fields.required("crate_scopes")?;
        if value.is_null() {
            Ok(None)
        } else {
            value.array().map(Some)
        }
    }
    /// Validated nullable RFC3339 expiry. No clock or expiry decision is inferred.
    pub fn with_expiry<R>(&self, inspect: impl FnOnce(Option<&str>) -> R) -> Result<R, Error> {
        let value = self.fields.required("expired_at")?;
        if value.is_null() {
            Ok(inspect(None))
        } else {
            value.with_text(|text| inspect(Some(text)))
        }
    }
    /// Protected validated fields including creation/last-use timestamps.
    /// Unknown metadata remains inert and redacted.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.fields
    }
}
/// Operation-specific result. Revocation acknowledges only the exact exchange.
#[derive(Debug)]
pub enum TokenResponse {
    /// Requested metadata matched its ID.
    Metadata(TokenMetadata),
    /// Provider acknowledged a revoke-by-ID update, possibly affecting zero rows.
    Revoked,
    /// Provider acknowledged self-revocation with an empty 204 response.
    CurrentRevoked,
}
