use cloud_sdk::operation::{
    ExecutionPermitError, PermitIdempotencyKey, PermitState, PermitTimestamp, ReconciliationToken,
    RecoveryToken,
};

use super::{RobotClientMutationOperation, RobotMutationPermitAttempt, RobotMutationPlanSubject};

macro_rules! direct_permit {
    ($name:ident, $inner:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name<'storage, 'fingerprint, 'request, R>
        where
            R: RobotClientMutationOperation,
        {
            inner: cloud_sdk::operation::$inner<'storage, 'fingerprint>,
            binding: super::MutationBinding<'request, R>,
        }

        impl<'storage, 'fingerprint, 'request, R> $name<'storage, 'fingerprint, 'request, R>
        where
            R: RobotClientMutationOperation,
        {
            /// Creates authority from one exact request-bound subject.
            pub fn new(
                subject: RobotMutationPlanSubject<'storage, 'fingerprint, 'request, R>,
                now: PermitTimestamp,
            ) -> Result<Self, ExecutionPermitError> {
                Ok(Self {
                    inner: cloud_sdk::operation::$inner::new(subject.inner, now)?,
                    binding: subject.binding,
                })
            }

            /// Returns the fail-closed lifecycle state.
            #[must_use]
            pub const fn state(&self) -> PermitState {
                self.inner.state()
            }

            /// Starts one attempt for the exact bound request.
            pub fn begin(
                &mut self,
                now: PermitTimestamp,
            ) -> Result<
                RobotMutationPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
                ExecutionPermitError,
            > {
                Ok(RobotMutationPermitAttempt {
                    inner: self.inner.begin(now)?,
                    binding: self.binding,
                })
            }

            /// Starts only if the supplied subject is exactly unchanged.
            pub fn begin_for(
                &mut self,
                candidate: RobotMutationPlanSubject<'_, '_, '_, R>,
                now: PermitTimestamp,
            ) -> Result<
                RobotMutationPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
                ExecutionPermitError,
            > {
                Ok(RobotMutationPermitAttempt {
                    inner: self.inner.begin_for(candidate.inner, now)?,
                    binding: self.binding,
                })
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
                candidate: RobotMutationPlanSubject<'_, '_, '_, R>,
                idempotency: PermitIdempotencyKey<'_>,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.inner
                    .reconcile_not_applied(token, candidate.inner, idempotency, now)
            }
        }

        impl<R: RobotClientMutationOperation> core::fmt::Debug for $name<'_, '_, '_, R> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("state", &self.inner.state())
                    .field("request", &"[bound]")
                    .finish()
            }
        }
    };
}

direct_permit!(
    RobotMutationPermit,
    MutationPermit,
    "Single-owner authority for one exact generic Robot mutation."
);
direct_permit!(
    RobotMutationDestructivePermit,
    DestructivePermit,
    "Single-owner authority for one exact destructive Robot mutation."
);
