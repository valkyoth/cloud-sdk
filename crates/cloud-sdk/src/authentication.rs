//! Provider-neutral authentication scope and execution contracts.

mod generation;
mod policy;
mod transport;
mod value;

pub use generation::{CredentialGeneration, CredentialGenerationError, RefreshHandoff};
pub use policy::{
    AuthenticationScope, AuthenticationScopeError, AuthenticationScopePolicy, ScopeField,
    ScopeRequirement, ScopeViolation,
};
pub use transport::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, BlockingAuthenticatedTransport,
};
pub use value::{MAX_SCOPE_VALUE_BYTES, ScopeValue, ScopeValueError};

#[cfg(test)]
mod tests;
