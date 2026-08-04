//! Explicitly shareable permits backed by caller-owned atomic state.

use core::sync::atomic::{AtomicU32, Ordering};

use super::state::{AttemptPhase, PermitAttempt};
use super::{
    ExecutionPermitError, PermitDisposition, PermitIdempotencyKey, PermitScope, PermitState,
    PermitTimestamp, PlanSubject, ReconciliationToken, RecoveryToken, ReplayPolicy,
};

const STATE_MASK: u32 = 0b111;
const GENERATION_SHIFT: u32 = 3;
const GENERATION_MASK: u32 = 0x1fff;
const REMAINING_SHIFT: u32 = 16;

const READY: u32 = 0;
const IN_FLIGHT: u32 = 1;
const RECOVERABLE: u32 = 2;
const PENDING: u32 = 3;
const SPENT: u32 = 4;

/// Caller-owned atomic authority shared by all explicit permit-handle clones.
///
/// The state is initialized only through a shared permit constructor taking
/// `&mut SharedPermitState`. That exclusive borrow prevents safe code from
/// binding two independent plans to the same atomic state concurrently.
pub struct SharedPermitState {
    packed: AtomicU32,
    last_offset: AtomicU32,
}

impl SharedPermitState {
    /// Creates unbound spent state ready for one exclusive plan binding.
    #[must_use]
    pub fn new() -> Self {
        Self {
            packed: AtomicU32::new(pack(SPENT, 0, 0)),
            last_offset: AtomicU32::new(0),
        }
    }

    /// Returns the current atomic lifecycle state.
    #[must_use]
    pub fn state(&self) -> PermitState {
        unpack_state(self.packed.load(Ordering::Acquire))
    }

    fn initialize(
        &mut self,
        subject: PlanSubject<'_, '_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        let offset = subject.validity().offset(now)?;
        *self.packed.get_mut() = pack(READY, 0, subject.attempt_budget().get());
        *self.last_offset.get_mut() = offset;
        Ok(())
    }

    fn begin(
        &self,
        subject: PlanSubject<'_, '_>,
        now: PermitTimestamp,
    ) -> Result<u16, ExecutionPermitError> {
        self.observe(subject, now)?;
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (state, generation, remaining) = unpack(current);
            match state {
                READY if remaining != 0 => {}
                READY | SPENT => return Err(ExecutionPermitError::Spent),
                IN_FLIGHT => return Err(ExecutionPermitError::AttemptInFlight),
                RECOVERABLE => return Err(ExecutionPermitError::RecoveryRequired),
                PENDING => return Err(ExecutionPermitError::ReconciliationRequired),
                _ => return Err(ExecutionPermitError::Spent),
            }
            let next_remaining = remaining
                .checked_sub(1)
                .ok_or(ExecutionPermitError::Spent)?;
            let next = pack(IN_FLIGHT, generation, next_remaining);
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(generation);
            }
        }
    }

    pub(super) fn complete(
        &self,
        expected_generation: u16,
        phase: AttemptPhase,
    ) -> PermitDisposition {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (state, generation, remaining) = unpack(current);
            if state != IN_FLIGHT || generation != expected_generation {
                return PermitDisposition::Spent;
            }
            let (next_state, disposition) = match phase {
                AttemptPhase::Applied => (SPENT, PermitDisposition::Spent),
                AttemptPhase::NotSent if remaining == 0 => (SPENT, PermitDisposition::Spent),
                AttemptPhase::NotSent => (
                    RECOVERABLE,
                    PermitDisposition::Recoverable(RecoveryToken(generation)),
                ),
                AttemptPhase::Uncertain => (
                    PENDING,
                    PermitDisposition::PendingReconciliation(ReconciliationToken(generation)),
                ),
            };
            let next = pack(next_state, generation, remaining);
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return disposition;
            }
        }
    }

    fn recover_not_sent(
        &self,
        subject: PlanSubject<'_, '_>,
        token: RecoveryToken,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.observe(subject, now)?;
        if subject.replay_policy() == ReplayPolicy::SingleAttempt {
            return Err(ExecutionPermitError::ReplayForbidden);
        }
        self.rearm(RECOVERABLE, token.0)
    }

    fn reconcile_not_applied(
        &self,
        bound: PlanSubject<'_, '_>,
        candidate: PlanSubject<'_, '_>,
        token: ReconciliationToken,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.observe(bound, now)?;
        if bound.replay_policy() != ReplayPolicy::ReconcileThenRetry {
            return Err(ExecutionPermitError::ReplayForbidden);
        }
        if !bound.fingerprint().matches(candidate.fingerprint()) {
            return Err(ExecutionPermitError::FingerprintMismatch);
        }
        if !bound
            .idempotency()
            .is_some_and(|expected| expected.matches(idempotency))
        {
            return Err(ExecutionPermitError::IdempotencyMismatch);
        }
        self.rearm(PENDING, token.0)
    }

    fn rearm(
        &self,
        required_state: u32,
        expected_generation: u16,
    ) -> Result<(), ExecutionPermitError> {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (state, generation, remaining) = unpack(current);
            if state != required_state || generation != expected_generation {
                return Err(ExecutionPermitError::StaleGeneration);
            }
            if remaining == 0 {
                let _ = self.packed.compare_exchange(
                    current,
                    pack(SPENT, generation, 0),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
                return Err(ExecutionPermitError::Spent);
            }
            let Some(next_generation) = generation
                .checked_add(1)
                .filter(|value| u32::from(*value) <= GENERATION_MASK)
            else {
                if self
                    .packed
                    .compare_exchange(
                        current,
                        pack(SPENT, generation, 0),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    return Err(ExecutionPermitError::GenerationExhausted);
                }
                continue;
            };
            let next = pack(READY, next_generation, remaining);
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(());
            }
        }
    }

    fn observe(
        &self,
        subject: PlanSubject<'_, '_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        let offset = subject.validity().offset(now)?;
        self.last_offset
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |previous| {
                (offset >= previous).then_some(offset)
            })
            .map(|_| ())
            .map_err(|_| ExecutionPermitError::ClockRollback)
    }
}

impl Default for SharedPermitState {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for SharedPermitState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SharedPermitState")
            .field("state", &self.state())
            .field("authority", &"[redacted]")
            .finish()
    }
}

macro_rules! shared_permit {
    ($name:ident, $scope:expr, $description:literal) => {
        #[doc = $description]
        ///
        /// Every clone references the same caller-owned atomic state. Cloning
        /// never creates new budget or an independent recovery generation.
        pub struct $name<'state, 'request, 'fingerprint> {
            state: &'state SharedPermitState,
            subject: PlanSubject<'request, 'fingerprint>,
        }

        impl<'state, 'request, 'fingerprint> $name<'state, 'request, 'fingerprint> {
            /// Exclusively binds fresh shared state to one confirmed plan.
            pub fn new(
                state: &'state mut SharedPermitState,
                subject: PlanSubject<'request, 'fingerprint>,
                now: PermitTimestamp,
            ) -> Result<Self, ExecutionPermitError> {
                if subject.scope() != $scope {
                    return Err(ExecutionPermitError::ScopeMismatch);
                }
                state.initialize(subject, now)?;
                Ok(Self { state, subject })
            }

            /// Returns the shared lifecycle state.
            #[must_use]
            pub fn state(&self) -> PermitState {
                self.state.state()
            }

            /// Atomically starts one attempt for the bound plan.
            pub fn begin(
                &self,
                now: PermitTimestamp,
            ) -> Result<PermitAttempt<'_, 'request, 'fingerprint>, ExecutionPermitError> {
                let generation = self.state.begin(self.subject, now)?;
                Ok(PermitAttempt::shared(self.state, self.subject, generation))
            }

            /// Starts only if the candidate fingerprint matches the bound plan.
            pub fn begin_for(
                &self,
                candidate: PlanSubject<'_, '_>,
                now: PermitTimestamp,
            ) -> Result<PermitAttempt<'_, 'request, 'fingerprint>, ExecutionPermitError> {
                if !self.subject.fingerprint().matches(candidate.fingerprint()) {
                    return Err(ExecutionPermitError::FingerprintMismatch);
                }
                self.begin(now)
            }

            /// Atomically recovers a generation-matched `NotSent` attempt.
            pub fn recover_not_sent(
                &self,
                token: RecoveryToken,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.state.recover_not_sent(self.subject, token, now)
            }

            /// Rearms after caller-performed operation-specific reconciliation.
            pub fn reconcile_not_applied(
                &self,
                token: ReconciliationToken,
                candidate: PlanSubject<'_, '_>,
                idempotency: PermitIdempotencyKey<'_>,
                now: PermitTimestamp,
            ) -> Result<(), ExecutionPermitError> {
                self.state
                    .reconcile_not_applied(self.subject, candidate, token, idempotency, now)
            }
        }

        impl Clone for $name<'_, '_, '_> {
            fn clone(&self) -> Self {
                Self {
                    state: self.state,
                    subject: self.subject,
                }
            }
        }

        impl core::fmt::Debug for $name<'_, '_, '_> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("state", &self.state())
                    .field("plan", &"[redacted]")
                    .finish()
            }
        }
    };
}

shared_permit!(
    SharedMutationPermit,
    PermitScope::Mutation,
    "Shared atomic mutation authority."
);
shared_permit!(
    SharedDestructivePermit,
    PermitScope::Destructive,
    "Shared atomic destructive authority."
);
shared_permit!(
    SharedCostPermit,
    PermitScope::Cost,
    "Shared atomic price-bounded authority."
);

fn pack(state: u32, generation: u16, remaining: u16) -> u32 {
    state | (u32::from(generation) << GENERATION_SHIFT) | (u32::from(remaining) << REMAINING_SHIFT)
}

fn unpack(value: u32) -> (u32, u16, u16) {
    let generation = u16::try_from((value >> GENERATION_SHIFT) & GENERATION_MASK).unwrap_or(0);
    let remaining = u16::try_from(value >> REMAINING_SHIFT).unwrap_or(0);
    (value & STATE_MASK, generation, remaining)
}

fn unpack_state(value: u32) -> PermitState {
    match value & STATE_MASK {
        READY => PermitState::Ready,
        IN_FLIGHT => PermitState::InFlight,
        RECOVERABLE => PermitState::Recoverable,
        PENDING => PermitState::PendingReconciliation,
        _ => PermitState::Spent,
    }
}
