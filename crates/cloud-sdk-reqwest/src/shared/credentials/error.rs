use core::fmt;

use super::super::BearerTokenError;

/// Credential-state access failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialStateError {
    /// The short-lived credential-state lock could not be recovered.
    Unavailable,
}

impl_static_error!(CredentialStateError,
    Self::Unavailable => "credential state is unavailable",
);

/// Validated token rotation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialUpdateError {
    /// The credential state could not be changed.
    StateUnavailable,
    /// The monotonic credential generation cannot advance.
    GenerationExhausted,
    /// An expiring lifecycle requires a replacement lifetime.
    LifetimeRequired,
    /// A static lifecycle cannot be changed into an expiring lifecycle.
    LifetimeForbidden,
}

impl_static_error!(CredentialUpdateError,
    Self::StateUnavailable => "credential state is unavailable",
    Self::GenerationExhausted => "credential generation is exhausted",
    Self::LifetimeRequired => "expiring credential replacement requires a lifetime",
    Self::LifetimeForbidden => "static credential replacement forbids a lifetime",
);

/// Time-qualified refresh-handoff failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RefreshHandoffError {
    /// Expiring credentials require a time-qualified refresh handoff.
    ExplicitTimeRequired,
    /// A static credential has no expiry-qualified refresh window.
    LifetimeNotConfigured,
    /// The credential has not reached its refresh window.
    RefreshNotRequired,
    /// The credential has reached its exclusive expiry.
    CredentialExpired,
    /// The supplied caller time precedes lifetime observation.
    ClockRollback,
}

impl_static_error!(RefreshHandoffError,
    Self::ExplicitTimeRequired => "expiring credential refresh requires explicit caller time",
    Self::LifetimeNotConfigured => "credential lifetime is not configured",
    Self::RefreshNotRequired => "credential has not reached its refresh window",
    Self::CredentialExpired => "credential has expired",
    Self::ClockRollback => "credential clock moved before lifetime observation",
);

/// Bearer-token validation or rotation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenRotationError {
    /// The replacement bearer token was rejected before state changed.
    TokenRejected(BearerTokenError),
    /// The credential state could not be changed.
    StateUnavailable,
    /// The monotonic credential generation cannot advance.
    GenerationExhausted,
    /// An expiring lifecycle requires a replacement lifetime.
    LifetimeRequired,
    /// A static lifecycle cannot be changed into an expiring lifecycle.
    LifetimeForbidden,
}

impl fmt::Display for TokenRotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TokenRejected(_) => "replacement bearer token was rejected",
            Self::StateUnavailable => "credential state is unavailable",
            Self::GenerationExhausted => "credential generation is exhausted",
            Self::LifetimeRequired => "expiring credential replacement requires a lifetime",
            Self::LifetimeForbidden => "static credential replacement forbids a lifetime",
        })
    }
}

impl core::error::Error for TokenRotationError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::TokenRejected(error) => Some(error),
            Self::StateUnavailable
            | Self::GenerationExhausted
            | Self::LifetimeRequired
            | Self::LifetimeForbidden => None,
        }
    }
}

/// Compare-and-swap bearer refresh failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenRefreshError {
    /// The replacement bearer token was rejected before state changed.
    TokenRejected(BearerTokenError),
    /// A newer rotation or refresh superseded this handoff.
    StaleGeneration,
    /// The refresh handoff belongs to a different credential lifecycle.
    CredentialMismatch,
    /// The credential state could not be changed.
    StateUnavailable,
    /// The monotonic credential generation cannot advance.
    GenerationExhausted,
    /// An expiring lifecycle requires a replacement lifetime.
    LifetimeRequired,
    /// A static lifecycle cannot be changed into an expiring lifecycle.
    LifetimeForbidden,
}

impl fmt::Display for TokenRefreshError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TokenRejected(_) => "refreshed bearer token was rejected",
            Self::StaleGeneration => "credential refresh generation is stale",
            Self::CredentialMismatch => "credential refresh handoff belongs to another credential",
            Self::StateUnavailable => "credential state is unavailable",
            Self::GenerationExhausted => "credential generation is exhausted",
            Self::LifetimeRequired => "expiring credential refresh requires a lifetime",
            Self::LifetimeForbidden => "static credential refresh forbids a lifetime",
        })
    }
}

impl core::error::Error for TokenRefreshError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::TokenRejected(error) => Some(error),
            Self::StaleGeneration
            | Self::CredentialMismatch
            | Self::StateUnavailable
            | Self::GenerationExhausted
            | Self::LifetimeRequired
            | Self::LifetimeForbidden => None,
        }
    }
}
