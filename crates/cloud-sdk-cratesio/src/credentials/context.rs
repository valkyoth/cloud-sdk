use super::{
    Api, CredentialError, CredentialKind, EmailConfirmation, Oidc, OwnerInvitation,
    TrustedPublishing,
};
use crate::endpoint::{ApiRequestTarget, OfficialCratesIoEndpoint};
use cloud_sdk::Method;
use core::{fmt, marker::PhantomData};

/// Closed credential destination, selected when ingesting the secret.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialOrigin {
    /// The production crates.io API, never its static-download host.
    Production,
    /// The isolated staging registry API.
    Staging,
}

impl CredentialOrigin {
    /// Returns the fixed official destination.
    #[must_use]
    pub const fn endpoint(self) -> OfficialCratesIoEndpoint {
        match self {
            Self::Production => OfficialCratesIoEndpoint::production_api(),
            Self::Staging => OfficialCratesIoEndpoint::staging_api(),
        }
    }
}

/// Sealed method/target context for exactly one credential kind.
///
/// Contexts carry no credential bytes. Path credentials supply their own
/// target suffix during protected preparation.
pub struct CredentialContext<'a, K: CredentialKind> {
    pub(super) origin: CredentialOrigin,
    pub(super) method: Method,
    pub(super) target: &'a str,
    kind: PhantomData<K>,
}

impl<'a, K: CredentialKind> CredentialContext<'a, K> {
    const fn new(origin: CredentialOrigin, method: Method, target: &'a str) -> Self {
        Self {
            origin,
            method,
            target,
            kind: PhantomData,
        }
    }
}

impl<'a> CredentialContext<'a, Api> {
    /// Admits only method/path pairs whose source lock accepts API tokens.
    ///
    /// Dynamic segments use a conservative unescaped ASCII profile. Domain
    /// request builders will apply their additional field and query policies.
    pub fn api(
        origin: CredentialOrigin,
        method: Method,
        target: ApiRequestTarget<'a>,
    ) -> Result<Self, CredentialError> {
        if !super::policy::api_allowed(method, target) {
            return Err(CredentialError::OperationNotAllowed);
        }
        Ok(Self::new(origin, method, target.as_str()))
    }
}

impl CredentialContext<'static, TrustedPublishing> {
    /// Admits temporary tokens to the package publication operation only.
    #[must_use]
    pub const fn publish(origin: CredentialOrigin) -> Self {
        Self::new(origin, Method::Put, "/api/v1/crates/new")
    }
    /// Admits a temporary token to its own revocation operation.
    #[must_use]
    pub const fn revoke(origin: CredentialOrigin) -> Self {
        Self::new(origin, Method::Delete, "/api/v1/trusted_publishing/tokens")
    }
}

impl CredentialContext<'static, Oidc> {
    /// Prepares an assertion for the crates.io JSON exchange endpoint only.
    #[must_use]
    pub const fn exchange(origin: CredentialOrigin) -> Self {
        Self::new(origin, Method::Post, "/api/v1/trusted_publishing/tokens")
    }
}

impl CredentialContext<'static, EmailConfirmation> {
    /// Prepares an email token in its fixed confirmation path only.
    #[must_use]
    pub const fn confirm_email(origin: CredentialOrigin) -> Self {
        Self::new(origin, Method::Put, "/api/v1/confirm/")
    }
}

impl CredentialContext<'static, OwnerInvitation> {
    /// Prepares an invitation token in its fixed acceptance path only.
    #[must_use]
    pub const fn accept_invitation(origin: CredentialOrigin) -> Self {
        Self::new(
            origin,
            Method::Put,
            "/api/v1/me/crate_owner_invitations/accept/",
        )
    }
}

impl<K: CredentialKind> fmt::Debug for CredentialContext<'_, K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CredentialContext([redacted])")
    }
}
