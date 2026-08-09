//! Provider-bound plan confirmation and execution permits.

use core::fmt;
use core::marker::PhantomData;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{
    AttemptBudget, CanonicalPlanFingerprint, ExecutionPermitError, PermitClock, PermitContext,
    PermitDisposition, PermitExecutionError, PermitIdempotencyKey, PermitState, PermitTimestamp,
    PermitValidity, PlanChange, PlanConfirmation, PlanCost, PlanFingerprintBuildError,
    PlanFingerprintDigest, PlanFingerprintScope, PlanSubject, ReconciliationToken, RecoveryToken,
    ReplayPolicy, SharedPermitState,
};
use cloud_sdk::retry::FingerprintHasher;
use cloud_sdk::transport::{BoundTransport, DeliveryClassified, DeliveryPhase, EndpointIdentity};

use super::identity::ExpectedResponseIdentity;
use super::prepared::{AssociatedCheckedResponse, Prepared};
use super::{HetznerOperation, types};

struct TypedResponseBinding<O> {
    expected: ExpectedResponseIdentity,
    operation: PhantomData<fn() -> O>,
}

impl<O> Clone for TypedResponseBinding<O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<O> Copy for TypedResponseBinding<O> {}

impl<O> TypedResponseBinding<O> {
    const fn new(expected: ExpectedResponseIdentity) -> Self {
        Self {
            expected,
            operation: PhantomData,
        }
    }
}

/// Typed Hetzner request and caller policy ready for plan fingerprinting.
pub struct AssociatedPlanConfirmation<'plan, 'request, O> {
    inner: PlanConfirmation<'plan, 'request>,
    binding: TypedResponseBinding<O>,
}

impl<'plan, 'request, O: HetznerOperation> AssociatedPlanConfirmation<'plan, 'request, O> {
    /// Binds caller policy to the same typed request that owns response identity.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: Prepared<'request, O>,
        endpoint: EndpointIdentity<'plan>,
        account: PlanFingerprintScope<'plan>,
        tenant: PlanFingerprintScope<'plan>,
        context: PermitContext<'plan>,
        validity: PermitValidity,
        replay: ReplayPolicy,
        attempts: AttemptBudget,
        change: PlanChange,
        cost: Option<PlanCost>,
        idempotency: Option<PermitIdempotencyKey<'plan>>,
    ) -> Self {
        let (prepared, expected) = prepared.into_associated_plan_parts();
        Self {
            inner: PlanConfirmation::new(
                prepared,
                endpoint,
                account,
                tenant,
                context,
                validity,
                replay,
                attempts,
                change,
                cost,
                idempotency,
            ),
            binding: TypedResponseBinding::new(expected),
        }
    }
}

impl<O: HetznerOperation> fmt::Debug for AssociatedPlanConfirmation<'_, '_, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssociatedPlanConfirmation")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("plan", &self.inner)
            .field("response_identity", &"[redacted]")
            .finish()
    }
}

/// Exact caller-buffer plan fingerprint retaining typed response identity.
pub struct AssociatedCanonicalPlanFingerprint<'output, 'plan, 'request, O> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'request>,
    binding: TypedResponseBinding<O>,
}

impl<'output, 'plan, 'request, O> AssociatedCanonicalPlanFingerprint<'output, 'plan, 'request, O> {
    /// Borrows the request, fingerprint, and opaque provider response binding.
    #[must_use]
    pub fn subject(&self) -> AssociatedPlanSubject<'_, '_, O> {
        AssociatedPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining typed response identity.
pub struct AssociatedPlanFingerprintDigest<'output, 'plan, 'request, O> {
    inner: PlanFingerprintDigest<'output, 'plan, 'request>,
    binding: TypedResponseBinding<O>,
}

impl<'output, 'plan, 'request, O> AssociatedPlanFingerprintDigest<'output, 'plan, 'request, O> {
    /// Borrows the request, fingerprint, and opaque provider response binding.
    #[must_use]
    pub fn subject(&self) -> AssociatedPlanSubject<'_, '_, O> {
        AssociatedPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact typed plan fingerprint in caller-owned storage.
pub fn build_associated_canonical_plan<'output, 'plan, 'request, O>(
    plan: AssociatedPlanConfirmation<'plan, 'request, O>,
    output: &'output mut [u8],
) -> Result<
    AssociatedCanonicalPlanFingerprint<'output, 'plan, 'request, O>,
    PlanFingerprintBuildError<core::convert::Infallible>,
> {
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(AssociatedCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a typed collision-resistant plan digest and clears scratch storage.
pub fn build_associated_plan_digest<'output, 'plan, 'request, O, H: FingerprintHasher>(
    plan: AssociatedPlanConfirmation<'plan, 'request, O>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    AssociatedPlanFingerprintDigest<'output, 'plan, 'request, O>,
    PlanFingerprintBuildError<H::Error>,
> {
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(AssociatedPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque typed subject used to create or recheck associated permits.
#[derive(Clone, Copy)]
pub struct AssociatedPlanSubject<'request, 'fingerprint, O> {
    inner: PlanSubject<'request, 'fingerprint>,
    binding: TypedResponseBinding<O>,
}

impl<O: HetznerOperation> fmt::Debug for AssociatedPlanSubject<'_, '_, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssociatedPlanSubject")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("subject", &self.inner)
            .field("response_identity", &"[redacted]")
            .finish()
    }
}

/// One in-flight permit attempt retaining its typed response binding.
#[must_use]
pub struct AssociatedPermitAttempt<'permit, 'request, 'fingerprint, O> {
    inner: cloud_sdk::operation::PermitAttempt<'permit, 'request, 'fingerprint>,
    binding: TypedResponseBinding<O>,
}

impl<'permit, 'request, 'fingerprint, O: HetznerOperation>
    AssociatedPermitAttempt<'permit, 'request, 'fingerprint, O>
{
    /// Completes a manually driven attempt with conservative delivery state.
    pub fn complete(self, phase: DeliveryPhase) -> PermitDisposition {
        self.inner.complete(phase)
    }

    /// Executes through a delivery-classified blocking transport.
    pub fn execute_blocking<'buffer, T, C>(
        self,
        clock: &C,
        transport: &T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> Result<AssociatedCheckedResponse<'buffer, O>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| AssociatedCheckedResponse::new(inner, binding.expected))
    }

    /// Executes through a delivery-classified Send-async transport.
    pub async fn execute_async<'transport, 'buffer, T, C>(
        self,
        clock: &'transport C,
        transport: &'transport T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> Result<AssociatedCheckedResponse<'buffer, O>, PermitExecutionError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + Sync + ?Sized,
        'request: 'transport,
        'permit: 'transport,
    {
        let binding = self.binding;
        self.inner
            .execute_async(clock, transport, body, headers)
            .await
            .map(|inner| AssociatedCheckedResponse::new(inner, binding.expected))
    }

    /// Executes through a delivery-classified local-async transport.
    pub async fn execute_local_async<'transport, 'buffer, T, C>(
        self,
        clock: &'transport C,
        transport: &'transport T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> Result<AssociatedCheckedResponse<'buffer, O>, PermitExecutionError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
        'request: 'transport,
        'permit: 'transport,
    {
        let binding = self.binding;
        self.inner
            .execute_local_async(clock, transport, body, headers)
            .await
            .map(|inner| AssociatedCheckedResponse::new(inner, binding.expected))
    }
}

impl<O: HetznerOperation> fmt::Debug for AssociatedPermitAttempt<'_, '_, '_, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssociatedPermitAttempt")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("attempt", &"[redacted]")
            .finish()
    }
}

macro_rules! associated_direct_permit {
    ($name:ident, $inner:ident, $marker:ty) => {
        #[doc = concat!("Provider-bound wrapper around `cloud_sdk::operation::", stringify!($inner), "`.")]
        pub struct $name<'request, 'fingerprint, O> {
            inner: cloud_sdk::operation::$inner<'request, 'fingerprint>,
            binding: TypedResponseBinding<O>,
        }

        impl<'request, 'fingerprint, O> $name<'request, 'fingerprint, O>
        where
            O: HetznerOperation<Permit = $marker>,
        {
            /// Creates authority only from a provider-bound typed subject.
            pub fn new(
                subject: AssociatedPlanSubject<'request, 'fingerprint, O>,
                now: PermitTimestamp,
            ) -> Result<Self, ExecutionPermitError> {
                Ok(Self {
                    inner: cloud_sdk::operation::$inner::new(subject.inner, now)?,
                    binding: subject.binding,
                })
            }

            /// Returns the current fail-closed lifecycle state.
            #[must_use]
            pub const fn state(&self) -> PermitState {
                self.inner.state()
            }

            /// Starts one attempt for the bound typed request.
            pub fn begin(
                &mut self,
                now: PermitTimestamp,
            ) -> Result<AssociatedPermitAttempt<'_, 'request, 'fingerprint, O>, ExecutionPermitError> {
                let binding = self.binding;
                let inner = self.inner.begin(now)?;
                Ok(AssociatedPermitAttempt { inner, binding })
            }

            /// Starts only when another typed subject has the same fingerprint.
            pub fn begin_for(
                &mut self,
                candidate: AssociatedPlanSubject<'_, '_, O>,
                now: PermitTimestamp,
            ) -> Result<AssociatedPermitAttempt<'_, 'request, 'fingerprint, O>, ExecutionPermitError> {
                let binding = self.binding;
                let inner = self.inner.begin_for(candidate.inner, now)?;
                Ok(AssociatedPermitAttempt { inner, binding })
            }

            /// Rearms after a generation-matched proven-not-sent result.
            pub fn recover_not_sent(
                &mut self,
                token: RecoveryToken,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.inner.recover_not_sent(token, now)
            }

            /// Rearms after operation-specific reconciliation.
            pub fn reconcile_not_applied(
                &mut self,
                token: ReconciliationToken,
                candidate: AssociatedPlanSubject<'_, '_, O>,
                idempotency: PermitIdempotencyKey<'_>,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.inner
                    .reconcile_not_applied(token, candidate.inner, idempotency, now)
            }
        }
    };
}

associated_direct_permit!(
    AssociatedMutationPermit,
    MutationPermit,
    types::MutationPermit
);
associated_direct_permit!(
    AssociatedDestructivePermit,
    DestructivePermit,
    types::DestructivePermit
);
associated_direct_permit!(AssociatedCostPermit, CostPermit, types::CostPermit);

macro_rules! associated_shared_permit {
    ($name:ident, $inner:ident, $marker:ty) => {
        #[doc = concat!("Provider-bound wrapper around `cloud_sdk::operation::", stringify!($inner), "`.")]
        pub struct $name<'state, 'request, 'fingerprint, O> {
            inner: cloud_sdk::operation::$inner<'state, 'request, 'fingerprint>,
            binding: TypedResponseBinding<O>,
        }

        impl<'state, 'request, 'fingerprint, O> $name<'state, 'request, 'fingerprint, O>
        where
            O: HetznerOperation<Permit = $marker>,
        {
            /// Exclusively binds shared state to one provider-bound subject.
            pub fn new(
                state: &'state mut SharedPermitState,
                subject: AssociatedPlanSubject<'request, 'fingerprint, O>,
                now: PermitTimestamp,
            ) -> Result<Self, ExecutionPermitError> {
                Ok(Self {
                    inner: cloud_sdk::operation::$inner::new(state, subject.inner, now)?,
                    binding: subject.binding,
                })
            }

            /// Returns the shared lifecycle state.
            #[must_use]
            pub fn state(&self) -> PermitState {
                self.inner.state()
            }

            /// Atomically starts one typed attempt.
            pub fn begin(
                &self,
                now: PermitTimestamp,
            ) -> Result<AssociatedPermitAttempt<'_, 'request, 'fingerprint, O>, ExecutionPermitError> {
                Ok(AssociatedPermitAttempt {
                    inner: self.inner.begin(now)?,
                    binding: self.binding,
                })
            }

            /// Starts only when another typed subject has the same fingerprint.
            pub fn begin_for(
                &self,
                candidate: AssociatedPlanSubject<'_, '_, O>,
                now: PermitTimestamp,
            ) -> Result<AssociatedPermitAttempt<'_, 'request, 'fingerprint, O>, ExecutionPermitError> {
                Ok(AssociatedPermitAttempt {
                    inner: self.inner.begin_for(candidate.inner, now)?,
                    binding: self.binding,
                })
            }

            /// Atomically recovers a generation-matched not-sent attempt.
            pub fn recover_not_sent(
                &self,
                token: RecoveryToken,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.inner.recover_not_sent(token, now)
            }

            /// Rearms after operation-specific reconciliation.
            pub fn reconcile_not_applied(
                &self,
                token: ReconciliationToken,
                candidate: AssociatedPlanSubject<'_, '_, O>,
                idempotency: PermitIdempotencyKey<'_>,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.inner
                    .reconcile_not_applied(token, candidate.inner, idempotency, now)
            }
        }

        impl<O> Clone for $name<'_, '_, '_, O> {
            fn clone(&self) -> Self {
                Self {
                    inner: self.inner.clone(),
                    binding: self.binding,
                }
            }
        }
    };
}

associated_shared_permit!(
    AssociatedSharedMutationPermit,
    SharedMutationPermit,
    types::MutationPermit
);
associated_shared_permit!(
    AssociatedSharedDestructivePermit,
    SharedDestructivePermit,
    types::DestructivePermit
);
associated_shared_permit!(
    AssociatedSharedCostPermit,
    SharedCostPermit,
    types::CostPermit
);
