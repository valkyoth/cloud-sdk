use cloud_sdk::operation::{
    ExecutionPermitError, PermitIdempotencyKey, PermitState, PermitTimestamp, ReconciliationToken,
    RecoveryToken, SharedPermitState,
};

use super::{RobotSshKeyPermitAttempt, RobotSshKeyPermitRequest, RobotSshKeyPlanSubject};

macro_rules! shared_permit {
    ($name:ident, $inner:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name<'state, 'storage, 'fingerprint, 'request, R: RobotSshKeyPermitRequest> {
            inner: cloud_sdk::operation::$inner<'state, 'storage, 'fingerprint>,
            binding: super::SshKeyBinding<'request, R>,
        }

        impl<'state, 'storage, 'fingerprint, 'request, R: RobotSshKeyPermitRequest>
            $name<'state, 'storage, 'fingerprint, 'request, R>
        {
            /// Binds shared state to one exact request and plan.
            pub fn new(
                state: &'state mut SharedPermitState,
                subject: RobotSshKeyPlanSubject<'storage, 'fingerprint, 'request, R>,
                now: PermitTimestamp,
            ) -> Result<Self, ExecutionPermitError> {
                Ok(Self {
                    inner: cloud_sdk::operation::$inner::new(state, subject.inner, now)?,
                    binding: subject.binding,
                })
            }

            /// Returns the shared fail-closed lifecycle state.
            #[must_use]
            pub fn state(&self) -> PermitState {
                self.inner.state()
            }

            /// Atomically starts one request-bound attempt.
            pub fn begin(
                &self,
                now: PermitTimestamp,
            ) -> Result<
                RobotSshKeyPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
                ExecutionPermitError,
            > {
                Ok(RobotSshKeyPermitAttempt {
                    inner: self.inner.begin(now)?,
                    binding: self.binding,
                })
            }

            /// Starts only when a rechecked subject has the same fingerprint.
            pub fn begin_for(
                &self,
                candidate: RobotSshKeyPlanSubject<'_, '_, '_, R>,
                now: PermitTimestamp,
            ) -> Result<
                RobotSshKeyPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
                ExecutionPermitError,
            > {
                Ok(RobotSshKeyPermitAttempt {
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
                candidate: RobotSshKeyPlanSubject<'_, '_, '_, R>,
                idempotency: PermitIdempotencyKey<'_>,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.inner
                    .reconcile_not_applied(token, candidate.inner, idempotency, now)
            }
        }

        impl<R: RobotSshKeyPermitRequest> Clone for $name<'_, '_, '_, '_, R> {
            fn clone(&self) -> Self {
                Self {
                    inner: self.inner.clone(),
                    binding: self.binding,
                }
            }
        }
    };
}

shared_permit!(
    RobotSshKeySharedMutationPermit,
    SharedMutationPermit,
    "Shared request-bound authority for one Robot ssh_keys mutation."
);
shared_permit!(
    RobotSshKeySharedDestructivePermit,
    SharedDestructivePermit,
    "Shared request-bound authority for one destructive Robot ssh_keys mutation."
);
