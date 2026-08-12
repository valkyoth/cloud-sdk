use cloud_sdk::operation::{
    ExecutionPermitError, PermitIdempotencyKey, PermitState, PermitTimestamp, ReconciliationToken,
    RecoveryToken,
};

use super::{RobotSubnetPermitAttempt, RobotSubnetPermitRequest, RobotSubnetPlanSubject};

macro_rules! direct_permit {
    ($name:ident, $inner:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name<'storage, 'fingerprint, 'request, R: RobotSubnetPermitRequest> {
            inner: cloud_sdk::operation::$inner<'storage, 'fingerprint>,
            binding: super::SubnetBinding<'request, R>,
        }

        impl<'storage, 'fingerprint, 'request, R: RobotSubnetPermitRequest>
            $name<'storage, 'fingerprint, 'request, R>
        {
            /// Creates authority from an exact request-bound subject.
            pub fn new(
                subject: RobotSubnetPlanSubject<'storage, 'fingerprint, 'request, R>,
                now: PermitTimestamp,
            ) -> Result<Self, ExecutionPermitError> {
                subject.binding.0.validate_authorization_evidence(now)?;
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
                RobotSubnetPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
                ExecutionPermitError,
            > {
                self.binding.0.validate_authorization_evidence(now)?;
                Ok(RobotSubnetPermitAttempt {
                    inner: self.inner.begin(now)?,
                    binding: self.binding,
                })
            }

            /// Starts only when a rechecked subject has the same fingerprint.
            pub fn begin_for(
                &mut self,
                candidate: RobotSubnetPlanSubject<'_, '_, '_, R>,
                now: PermitTimestamp,
            ) -> Result<
                RobotSubnetPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
                ExecutionPermitError,
            > {
                self.binding.0.validate_authorization_evidence(now)?;
                candidate.binding.0.validate_authorization_evidence(now)?;
                Ok(RobotSubnetPermitAttempt {
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
                candidate: RobotSubnetPlanSubject<'_, '_, '_, R>,
                idempotency: PermitIdempotencyKey<'_>,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.binding.0.validate_authorization_evidence(now)?;
                candidate.binding.0.validate_authorization_evidence(now)?;
                self.inner
                    .reconcile_not_applied(token, candidate.inner, idempotency, now)
            }
        }

        impl<R: RobotSubnetPermitRequest> core::fmt::Debug for $name<'_, '_, '_, R> {
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
    RobotSubnetMutationPermit,
    MutationPermit,
    "Direct request-bound authority for one Robot subnet mutation."
);
direct_permit!(
    RobotSubnetDestructivePermit,
    DestructivePermit,
    "Direct request-bound authority for one destructive Robot subnet mutation."
);
