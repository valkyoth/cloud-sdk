//! Single-owner execution permits.

use super::state::{DirectState, PermitAttempt};
use super::{
    ExecutionPermitError, PermitIdempotencyKey, PermitScope, PermitState, PermitTimestamp,
    PlanSubject, ReconciliationToken, RecoveryToken,
};

macro_rules! direct_permit {
    ($name:ident, $scope:expr, $description:literal) => {
        #[doc = $description]
        ///
        /// This authority is intentionally neither `Copy` nor `Clone`.
        #[doc = concat!(
                            "```compile_fail\nuse cloud_sdk::operation::", stringify!($name),
                            ";\nfn clone_authority(value: ", stringify!($name),
                            "<'_, '_>) { let _ = value.clone(); }\n```"
                        )]
        #[doc = concat!(
                            "```compile_fail\nuse cloud_sdk::operation::", stringify!($name),
                            ";\nfn copy_authority(value: ", stringify!($name),
                            "<'_, '_>) { let _first = value; let _second = value; }\n```"
                        )]
        pub struct $name<'request, 'fingerprint> {
            inner: DirectState<'request, 'fingerprint>,
        }

        impl<'request, 'fingerprint> $name<'request, 'fingerprint> {
            /// Creates one direct authority for an exact confirmed plan.
            pub fn new(
                subject: PlanSubject<'request, 'fingerprint>,
                now: PermitTimestamp,
            ) -> Result<Self, ExecutionPermitError> {
                Ok(Self {
                    inner: DirectState::new(subject, $scope, now)?,
                })
            }

            /// Returns the current fail-closed lifecycle state.
            #[must_use]
            pub const fn state(&self) -> PermitState {
                self.inner.state()
            }

            /// Starts one attempt for the originally confirmed request.
            pub fn begin(
                &mut self,
                now: PermitTimestamp,
            ) -> Result<PermitAttempt<'_, 'request, 'fingerprint>, ExecutionPermitError> {
                self.inner.begin(now)
            }

            /// Starts one attempt only if a supplied plan still matches exactly.
            pub fn begin_for(
                &mut self,
                subject: PlanSubject<'_, '_>,
                now: PermitTimestamp,
            ) -> Result<PermitAttempt<'_, 'request, 'fingerprint>, ExecutionPermitError> {
                self.inner.begin_for(subject, now)
            }

            /// Rearms authority after a generation-matched `NotSent` result.
            pub fn recover_not_sent(
                &mut self,
                token: RecoveryToken,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.inner.recover_not_sent(token, now)
            }

            /// Rearms after caller-performed operation-specific reconciliation.
            ///
            /// Callers must invoke this only after proving that the provider did
            /// not apply the uncertain attempt. The exact plan and idempotency
            /// identity are rechecked before authority becomes ready.
            pub fn reconcile_not_applied(
                &mut self,
                token: ReconciliationToken,
                subject: PlanSubject<'_, '_>,
                idempotency: PermitIdempotencyKey<'_>,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.inner
                    .reconcile_not_applied(token, subject, idempotency, now)
            }
        }

        impl core::fmt::Debug for $name<'_, '_> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("state", &self.inner.state())
                    .field("plan", &"[redacted]")
                    .finish()
            }
        }
    };
}

direct_permit!(
    MutationPermit,
    PermitScope::Mutation,
    "Single-owner authority for one non-destructive mutation plan."
);
direct_permit!(
    DestructivePermit,
    PermitScope::Destructive,
    "Single-owner authority for one destructive mutation plan."
);
direct_permit!(
    CostPermit,
    PermitScope::Cost,
    "Single-owner authority for one exact price-bounded cost plan."
);
