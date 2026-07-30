//! Provider-neutral authentication scope and execution contracts.

mod generation;
mod policy;
mod signing;
mod transport;
mod value;

pub use generation::{CredentialGeneration, CredentialGenerationError, RefreshHandoff};
pub use policy::{
    AuthenticationScope, AuthenticationScopeError, AuthenticationScopePolicy, ScopeField,
    ScopeRequirement, ScopeViolation,
};
pub use signing::{
    CanonicalSigningInput, MAX_CANONICAL_SIGNING_INPUT_BYTES, MAX_SIGNING_BODY_DIGEST_BYTES,
    MAX_SIGNING_HEADERS, MAX_SIGNING_NONCE_BYTES, RequestBodyHasher, RequestSigner,
    SigningBodyDigest, SigningHeaders, SigningInputError, SigningNonce, SigningValueError,
    UnixTime,
};
pub use transport::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, BlockingAuthenticatedTransport,
};
pub use value::{MAX_SCOPE_VALUE_BYTES, ScopeValue, ScopeValueError};

#[cfg(test)]
mod tests;
