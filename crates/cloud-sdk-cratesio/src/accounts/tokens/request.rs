use crate::{
    credentials::{ApiToken, CredentialOrigin},
    endpoint::ApiRequestTarget,
    identifiers::NumericId,
    query::{ApiPath, FixedSegment as F, PathSegment as P, QueryError},
};
use cloud_sdk::Method;

/// Exact source-locked public token operation; no automatic retries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenOperation {
    /// Inspect metadata for a known token ID.
    Inspect,
    /// Revoke a known token ID belonging to the authenticated user.
    Revoke,
    /// Revoke the exact credential used in this exchange.
    RevokeCurrent,
}
impl TokenOperation {
    /// Public OpenAPI operation identity.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Inspect => "find_api_token",
            Self::Revoke => "revoke_api_token",
            Self::RevokeCurrent => "revoke_current_api_token",
        }
    }
    /// Exact HTTP method.
    pub const fn method(self) -> Method {
        if matches!(self, Self::Inspect) {
            Method::Get
        } else {
            Method::Delete
        }
    }
    /// Revocation is destructive even though repeated provider updates converge.
    pub const fn is_destructive(self) -> bool {
        !matches!(self, Self::Inspect)
    }
}
/// Non-cloneable, consumed authority bound to one action, ID and credential.
/// The server verifies ownership; this value expresses caller intent only.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::accounts::tokens::TokenPermit;
/// fn replay(p: TokenPermit<'_>) { let _ = p.clone(); }
/// ```
/// ```compile_fail
/// use cloud_sdk_cratesio::{accounts::tokens::TokenPermit, credentials::ApiToken};
/// fn rotate(t: &mut ApiToken) {
///     let permit = TokenPermit::confirm_revoke_current(t);
///     t.clear();
///     let _ = permit.operation();
/// }
/// ```
pub struct TokenPermit<'a> {
    pub(super) operation: TokenOperation,
    pub(super) id: Option<NumericId>,
    pub(super) credential: &'a ApiToken,
}
impl<'a> TokenPermit<'a> {
    /// Authorizes one read of this token's metadata, not a mutation.
    pub const fn inspect(id: NumericId, credential: &'a ApiToken) -> Self {
        Self {
            operation: TokenOperation::Inspect,
            id: Some(id),
            credential,
        }
    }
    /// Explicit destructive confirmation for this exact ID and credential.
    /// A successful acknowledgement does not prove the ID existed upstream.
    pub const fn confirm_revoke(id: NumericId, credential: &'a ApiToken) -> Self {
        Self {
            operation: TokenOperation::Revoke,
            id: Some(id),
            credential,
        }
    }
    /// Explicitly revoke the credential used to authenticate this one exchange.
    /// An error may follow successful revocation. Never automatically retry.
    pub const fn confirm_revoke_current(credential: &'a ApiToken) -> Self {
        Self {
            operation: TokenOperation::RevokeCurrent,
            id: None,
            credential,
        }
    }
    /// Exact public operation, without private identity or credential data.
    pub const fn operation(&self) -> TokenOperation {
        self.operation
    }
    /// Official origin bound to the borrowed credential.
    pub fn origin(&self) -> CredentialOrigin {
        self.credential.origin()
    }
    /// Atomic target encoding. Credential bytes never appear in this target.
    pub fn write_target<'b>(
        &self,
        output: &'b mut [u8],
    ) -> Result<ApiRequestTarget<'b>, QueryError> {
        match self.id {
            Some(id) => {
                ApiPath::new(&[P::Fixed(F::Me), P::Fixed(F::Tokens), P::Id(id)])?.write(output)
            }
            None => ApiPath::new(&[P::Fixed(F::Tokens), P::Fixed(F::Current)])?.write(output),
        }
    }
}
impl core::fmt::Debug for TokenPermit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("TokenPermit([redacted])")
    }
}
