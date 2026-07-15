//! Explicit operation safety and retry classification.

/// Provider operation impact.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OperationImpact {
    /// The operation cannot change provider state.
    ReadOnly,
    /// The operation changes provider state without being inherently destructive.
    Mutation,
    /// The operation deletes, disables, resets, detaches, or otherwise destroys state.
    Destructive,
}

/// HTTP request semantics independent of provider impact.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RequestSemantics {
    /// Repeating the request is both read-only and idempotent.
    Safe,
    /// Repeating the request has the same intended effect, but it changes state.
    Idempotent,
    /// Repeating the request can create an additional or different effect.
    NonIdempotent,
}

/// Whether caller-owned retry policy may retry the operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RetryEligibility {
    /// Retrying is not admitted by operation metadata.
    Never,
    /// An explicit caller policy may retry eligible transient failures.
    ExplicitPolicy,
}

/// Whether executing the operation may directly incur provider charges.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CostIntent {
    /// The operation has no known direct resource cost.
    NoKnownCost,
    /// The operation may create or enlarge a billed resource.
    MayIncurCost,
}

/// Incoherent operation metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationMetadataError {
    /// Read-only operations must use safe request semantics.
    ReadOnlyMustBeSafe,
    /// Mutating and destructive operations cannot use safe semantics.
    StateChangeCannotBeSafe,
    /// Non-idempotent operations cannot be retry eligible.
    NonIdempotentRetry,
}

impl_static_error!(OperationMetadataError,
    Self::ReadOnlyMustBeSafe => "read-only operation must use safe semantics",
    Self::StateChangeCannotBeSafe => "state-changing operation cannot use safe semantics",
    Self::NonIdempotentRetry => "non-idempotent operation cannot be retry eligible",
);

/// Complete operation safety metadata without permissive defaults.
///
/// ```compile_fail
/// use cloud_sdk::operation::OperationMetadata;
///
/// let _: OperationMetadata = Default::default();
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationMetadata {
    impact: OperationImpact,
    semantics: RequestSemantics,
    retry: RetryEligibility,
    cost: CostIntent,
}

impl OperationMetadata {
    /// Creates complete metadata after checking safety invariants.
    pub const fn new(
        impact: OperationImpact,
        semantics: RequestSemantics,
        retry: RetryEligibility,
        cost: CostIntent,
    ) -> Result<Self, OperationMetadataError> {
        match (impact, semantics, retry) {
            (OperationImpact::ReadOnly, semantics, _)
                if !matches!(semantics, RequestSemantics::Safe) =>
            {
                return Err(OperationMetadataError::ReadOnlyMustBeSafe);
            }
            (
                OperationImpact::Mutation | OperationImpact::Destructive,
                RequestSemantics::Safe,
                _,
            ) => {
                return Err(OperationMetadataError::StateChangeCannotBeSafe);
            }
            (_, RequestSemantics::NonIdempotent, RetryEligibility::ExplicitPolicy) => {
                return Err(OperationMetadataError::NonIdempotentRetry);
            }
            _ => {}
        }
        Ok(Self {
            impact,
            semantics,
            retry,
            cost,
        })
    }

    /// Returns provider-state impact.
    #[must_use]
    pub const fn impact(self) -> OperationImpact {
        self.impact
    }

    /// Returns HTTP request semantics.
    #[must_use]
    pub const fn semantics(self) -> RequestSemantics {
        self.semantics
    }

    /// Returns explicit retry eligibility.
    #[must_use]
    pub const fn retry_eligibility(self) -> RetryEligibility {
        self.retry
    }

    /// Returns direct cost intent.
    #[must_use]
    pub const fn cost_intent(self) -> CostIntent {
        self.cost
    }
}
