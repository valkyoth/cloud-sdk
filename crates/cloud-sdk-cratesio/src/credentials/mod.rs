//! Protected credentials and source-locked, origin-bound adapter contexts.
//!
//! This is a preparation boundary, not a network client. Adapter callbacks are
//! trusted: they must use the supplied endpoint, method, target and material,
//! must not copy secrets into logs, and must disable redirects. No expiry,
//! signature, issuer, audience or server-side permission claim is inferred
//! from syntactic token validation. See the crate README for these boundaries.

mod context;
mod kind;
mod material;
mod policy;
mod secret;

pub use context::{CredentialContext, CredentialOrigin};
pub use kind::{Api, CredentialKind, EmailConfirmation, Oidc, OwnerInvitation, TrustedPublishing};
pub use material::ScopedCredentialMaterial;
pub use secret::Credential;

/// Protected raw crates.io API token (not a Bearer header).
pub type ApiToken = Credential<Api>;
/// Protected temporary Bearer token, only for publishing and self-revocation.
pub type TrustedPublishingToken = Credential<TrustedPublishing>;
/// Protected compact signed OIDC assertion, only for token exchange JSON.
pub type OidcAssertion = Credential<Oidc>;
/// Protected token used only in the email confirmation path.
pub type EmailConfirmationToken = Credential<EmailConfirmation>;
/// Protected token used only in the owner invitation acceptance path.
pub type OwnerInvitationToken = Credential<OwnerInvitation>;

/// Payload-free credential validation, preparation or rotation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialError {
    /// The input is empty.
    Empty,
    /// The input exceeds the kind's documented local byte bound.
    TooLong,
    /// The input does not match the credential's lexical profile.
    InvalidSyntax,
    /// Protected storage could not be allocated.
    Allocation,
    /// Protected UTF-8 storage was unavailable.
    StorageUnavailable,
    /// The operation is not admitted for this credential type.
    OperationNotAllowed,
    /// Origin, scheme, port, or base path does not match the credential.
    DestinationMismatch,
    /// Caller storage cannot hold the complete material.
    OutputTooSmall,
}

impl core::fmt::Display for CredentialError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "crates.io credential is empty",
            Self::TooLong => "crates.io credential exceeds its byte limit",
            Self::InvalidSyntax => "crates.io credential syntax is invalid",
            Self::Allocation => "crates.io protected allocation failed",
            Self::StorageUnavailable => "crates.io protected storage is unavailable",
            Self::OperationNotAllowed => "crates.io credential operation is not allowed",
            Self::DestinationMismatch => "crates.io credential destination does not match",
            Self::OutputTooSmall => "crates.io credential output is too small",
        })
    }
}

impl core::error::Error for CredentialError {}

#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod tests;
