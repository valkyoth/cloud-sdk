use crate::{
    credentials::ApiToken,
    endpoint::ApiRequestTarget,
    identifiers::{CrateName, Version},
    query::{ApiPath, FixedSegment as F, PathSegment as P, QueryError},
};
use cloud_sdk::Method;

/// Cargo-compatible visibility mutation, not archive deletion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum YankOperation {
    /// Prevent new dependency selections; existing lockfiles remain usable.
    Yank,
    /// Restore eligibility for new dependency selections.
    Unyank,
}
impl YankOperation {
    /// Official operation ID.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Yank => "yank_version",
            Self::Unyank => "unyank_version",
        }
    }
    /// Cargo verb; both requests have no body.
    pub const fn method(self) -> Method {
        match self {
            Self::Yank => Method::Delete,
            Self::Unyank => Method::Put,
        }
    }
    /// Intended state, not evidence that it was applied or reached the index.
    pub const fn requested_yanked(self) -> bool {
        matches!(self, Self::Yank)
    }
    /// Repeating an intent converges only without intervening writers.
    pub const fn is_state_convergent(self) -> bool {
        true
    }
    /// Fresh consent is required even after timeout or acknowledgement failure.
    pub const fn permits_automatic_retry(self) -> bool {
        false
    }
}
/// Immutable crate/version-bound intent; does not dispatch or authorize a write.
#[derive(Clone, Copy)]
pub struct YankRequest<'a> {
    pub(super) name: CrateName<'a>,
    pub(super) version: Version<'a>,
    pub(super) operation: YankOperation,
}
impl<'a> YankRequest<'a> {
    /// Explicitly request yanking this exact version, including its build suffix.
    /// The Cargo endpoint also clears any existing yank message.
    pub const fn yank(name: CrateName<'a>, version: Version<'a>) -> Self {
        Self {
            name,
            version,
            operation: YankOperation::Yank,
        }
    }
    /// Explicitly request unyanking this exact version, clearing its yank message.
    pub const fn unyank(name: CrateName<'a>, version: Version<'a>) -> Self {
        Self {
            name,
            version,
            operation: YankOperation::Unyank,
        }
    }
    /// Immutable mutation classification.
    pub const fn operation(self) -> YankOperation {
        self.operation
    }
    /// Atomic target encoding; insufficient output remains unchanged.
    pub fn write_target(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError> {
        ApiPath::new(&[
            P::Fixed(F::Crates),
            P::Crate(self.name),
            P::Version(self.version),
            P::Fixed(match self.operation {
                YankOperation::Yank => F::Yank,
                YankOperation::Unyank => F::Unyank,
            }),
        ])?
        .write(output)
    }
    /// Authorize one attempt with this credential. This is local consent, not
    /// proof of provider ownership, token scope, or compare-and-swap protection.
    pub fn confirm(self, credential: &'a ApiToken) -> YankPermit<'a> {
        YankPermit {
            request: self,
            credential,
        }
    }
}
impl core::fmt::Debug for YankRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("YankRequest([redacted])")
    }
}
/// Consumed, non-cloneable consent for one exact intent and credential.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::publishing::YankPermit;
/// fn duplicate(p: YankPermit<'_>) { let _ = p.clone(); }
/// ```
///
/// ```compile_fail
/// use cloud_sdk_cratesio::{publishing::YankRequest, credentials::ApiToken, identifiers::{CrateName, Version}};
/// fn clear(t: &mut ApiToken, n: CrateName<'_>, v: Version<'_>) {
///     let p = YankRequest::yank(n, v).confirm(t);
///     t.clear();
///     let _ = p.operation();
/// }
/// ```
pub struct YankPermit<'a> {
    pub(super) request: YankRequest<'a>,
    pub(super) credential: &'a ApiToken,
}
impl YankPermit<'_> {
    /// Exact operation without exposing credential or target.
    pub const fn operation(&self) -> YankOperation {
        self.request.operation
    }
}
impl core::fmt::Debug for YankPermit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("YankPermit([redacted])")
    }
}
