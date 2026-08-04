use crate::operation::{RequestIdPolicy, RetryEligibility};

/// Finite payload-free client failure category.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticErrorCategory {
    /// Provider request preparation failed before transport access.
    Preparation,
    /// Execution authority was absent or invalid.
    Authorization,
    /// Endpoint identity or provider endpoint policy rejected dispatch.
    Endpoint,
    /// The concrete transport failed without exposing its error payload.
    Transport,
    /// Response staging or commit failed.
    ResponseTransaction,
    /// Provider-neutral response policy rejected the response.
    ResponsePolicy,
    /// Provider-owned checked decoding failed.
    Decode,
}

/// Retry classification copied from validated operation metadata.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticRetryCategory {
    /// Operation metadata forbids retry.
    Ineligible,
    /// Retry requires an explicit caller-owned policy.
    ExplicitPolicy,
}

impl From<RetryEligibility> for DiagnosticRetryCategory {
    fn from(value: RetryEligibility) -> Self {
        match value {
            RetryEligibility::Never => Self::Ineligible,
            RetryEligibility::ExplicitPolicy => Self::ExplicitPolicy,
        }
    }
}

/// Request-identifier observation without identifier bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticRequestId {
    /// Operation policy discards request identifiers without exposing presence.
    Discarded,
    /// An admitted protected or retainable identifier was absent.
    Absent,
    /// An identifier is closure-scoped to checked response handling.
    Protected,
    /// An identifier may move only into cleanup-owning retained metadata.
    Retainable,
}

impl DiagnosticRequestId {
    pub(crate) const fn classify(policy: RequestIdPolicy, present: bool) -> Self {
        match (policy, present) {
            (RequestIdPolicy::Discard, _) => Self::Discarded,
            (RequestIdPolicy::Protected | RequestIdPolicy::Retain, false) => Self::Absent,
            (RequestIdPolicy::Protected, true) => Self::Protected,
            (RequestIdPolicy::Retain, true) => Self::Retainable,
        }
    }
}
