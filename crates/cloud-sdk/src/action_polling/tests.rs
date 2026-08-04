use super::{
    ActionObserveError, ActionPollError, ActionPollLimits, ActionPollLimitsError, ActionPollStep,
    ActionPoller, ActionUpdate, ExponentialBackoff, ExponentialBackoffError, PollBackoff,
    PollContext, PollControl, PollRequestStep, ProgressObservation, ProgressPolicy,
    ProviderTimeError, ProviderTimeObservation,
};
use crate::rate_limit::WallClockTimestamp;
use crate::retry::{MonotonicDuration, MonotonicInstant};
use core::fmt::{self, Write};

fn duration(value: u64) -> MonotonicDuration {
    MonotonicDuration::new(value)
}

fn instant(value: u64) -> MonotonicInstant {
    MonotonicInstant::new(value)
}

fn limits(observations: u32) -> ActionPollLimits {
    ActionPollLimits::new(observations, duration(8), duration(20), duration(40))
        .unwrap_or_else(|_| unreachable!())
}

fn make_backoff() -> ExponentialBackoff {
    ExponentialBackoff::new(duration(2), duration(8), 2).unwrap_or_else(|_| unreachable!())
}

struct DebugBuffer {
    bytes: [u8; 64],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 64],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[test]
fn request_delay_and_terminal_steps_are_explicit() {
    let mut poller = ActionPoller::new(limits(4), ProgressPolicy::Nondecreasing, instant(10));
    let mut backoff = make_backoff();
    assert_eq!(
        poller.next_request(PollControl::Continue, instant(10)),
        Ok(PollRequestStep::Request)
    );
    assert_eq!(
        poller.next_request(PollControl::Continue, instant(10)),
        Err(ActionPollError::ResponsePending)
    );
    assert_eq!(
        poller.observe(
            ActionUpdate::<()>::Running,
            ProgressObservation::Percent(10),
            None,
            ProviderTimeObservation::default(),
            instant(11),
            &mut backoff,
        ),
        Ok(ActionPollStep::Delay(duration(2)))
    );
    assert_eq!(
        poller.next_request(PollControl::Continue, instant(12)),
        Ok(PollRequestStep::Delay(duration(1)))
    );
    assert_eq!(
        poller.next_request(PollControl::Continue, instant(13)),
        Ok(PollRequestStep::Request)
    );
    assert_eq!(
        poller.observe(
            ActionUpdate::<()>::Success,
            ProgressObservation::Percent(255),
            None,
            ProviderTimeObservation::default(),
            instant(14),
            &mut backoff,
        ),
        Ok(ActionPollStep::Complete)
    );
    assert!(poller.is_terminal());
}

#[test]
fn observation_limit_is_unconditional_and_terminal() {
    let mut poller = ActionPoller::new(limits(2), ProgressPolicy::Nondecreasing, instant(0));
    let mut backoff = make_backoff();
    assert!(
        poller
            .next_request(PollControl::Continue, instant(0))
            .is_ok()
    );
    assert!(
        poller
            .observe(
                ActionUpdate::<()>::Running,
                ProgressObservation::Percent(0),
                None,
                ProviderTimeObservation::default(),
                instant(1),
                &mut backoff,
            )
            .is_ok()
    );
    assert!(
        poller
            .next_request(PollControl::Continue, instant(3))
            .is_ok()
    );
    assert_eq!(
        poller.observe(
            ActionUpdate::<()>::Running,
            ProgressObservation::Percent(1),
            None,
            ProviderTimeObservation::default(),
            instant(4),
            &mut backoff,
        ),
        Err(ActionObserveError::Driver(
            ActionPollError::ObservationLimitExceeded
        ))
    );
    assert_eq!(poller.observations(), 2);
    assert!(poller.is_terminal());
}

#[test]
fn regressions_require_explicit_bounded_resets() {
    let mut strict = ActionPoller::new(limits(4), ProgressPolicy::Nondecreasing, instant(0));
    let mut backoff = make_backoff();
    assert!(
        strict
            .next_request(PollControl::Continue, instant(0))
            .is_ok()
    );
    assert!(
        strict
            .observe(
                ActionUpdate::<()>::Running,
                ProgressObservation::Percent(80),
                None,
                ProviderTimeObservation::default(),
                instant(1),
                &mut backoff,
            )
            .is_ok()
    );
    assert!(
        strict
            .next_request(PollControl::Continue, instant(3))
            .is_ok()
    );
    assert_eq!(
        strict.observe(
            ActionUpdate::<()>::Running,
            ProgressObservation::Percent(20),
            None,
            ProviderTimeObservation::default(),
            instant(4),
            &mut backoff,
        ),
        Err(ActionObserveError::Driver(
            ActionPollError::ProgressRegressed
        ))
    );

    let policy = ProgressPolicy::ExplicitResets { max_resets: 1 };
    let mut resettable = ActionPoller::new(limits(4), policy, instant(0));
    let mut reset_backoff = make_backoff();
    assert!(
        resettable
            .next_request(PollControl::Continue, instant(0))
            .is_ok()
    );
    assert!(
        resettable
            .observe(
                ActionUpdate::<()>::Running,
                ProgressObservation::Percent(80),
                None,
                ProviderTimeObservation::default(),
                instant(1),
                &mut reset_backoff,
            )
            .is_ok()
    );
    assert!(
        resettable
            .next_request(PollControl::Continue, instant(3))
            .is_ok()
    );
    assert_eq!(
        resettable.observe(
            ActionUpdate::<()>::Running,
            ProgressObservation::Reset(10),
            None,
            ProviderTimeObservation::default(),
            instant(4),
            &mut reset_backoff,
        ),
        Ok(ActionPollStep::Delay(duration(2)))
    );
}

#[test]
fn cancellation_timeout_and_monotonic_rollback_stop_requests() {
    let mut cancelled = ActionPoller::new(limits(4), ProgressPolicy::Unordered, instant(0));
    assert_eq!(
        cancelled.next_request(PollControl::Cancel, instant(0)),
        Ok(PollRequestStep::Cancelled)
    );
    assert!(cancelled.is_terminal());

    let mut timed_out = ActionPoller::new(limits(4), ProgressPolicy::Unordered, instant(0));
    assert_eq!(
        timed_out.next_request(PollControl::Continue, instant(40)),
        Ok(PollRequestStep::TimedOut)
    );

    let mut rollback = ActionPoller::new(limits(4), ProgressPolicy::Unordered, instant(10));
    assert_eq!(
        rollback.next_request(PollControl::Continue, instant(9)),
        Err(ActionPollError::MonotonicRollback)
    );
    assert!(rollback.is_terminal());
}

#[test]
fn provider_wall_clock_rollback_never_extends_monotonic_budgets() {
    let mut poller = ActionPoller::new(limits(3), ProgressPolicy::Unordered, instant(0));
    let mut backoff = make_backoff();
    assert!(
        poller
            .next_request(PollControl::Continue, instant(0))
            .is_ok()
    );
    let later = ProviderTimeObservation::new(Some(WallClockTimestamp::new(500)), None)
        .unwrap_or_else(|_| unreachable!());
    assert!(
        poller
            .observe(
                ActionUpdate::<()>::Running,
                ProgressObservation::Unavailable,
                None,
                later,
                instant(1),
                &mut backoff,
            )
            .is_ok()
    );
    assert!(
        poller
            .next_request(PollControl::Continue, instant(3))
            .is_ok()
    );
    let earlier = ProviderTimeObservation::new(Some(WallClockTimestamp::new(100)), None)
        .unwrap_or_else(|_| unreachable!());
    assert!(
        poller
            .observe(
                ActionUpdate::<()>::Success,
                ProgressObservation::Unavailable,
                None,
                earlier,
                instant(4),
                &mut backoff,
            )
            .is_ok()
    );
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct SecretPolicyError(&'static str);

struct FailingBackoff;

impl PollBackoff for FailingBackoff {
    type Error = SecretPolicyError;

    fn delay(&mut self, _context: PollContext) -> Result<MonotonicDuration, Self::Error> {
        Err(SecretPolicyError("customer-payload"))
    }
}

#[test]
fn policy_errors_are_redacted_and_terminal() {
    let mut poller = ActionPoller::new(limits(3), ProgressPolicy::Unordered, instant(0));
    assert!(
        poller
            .next_request(PollControl::Continue, instant(0))
            .is_ok()
    );
    let error = poller.observe(
        ActionUpdate::<()>::Running,
        ProgressObservation::Unavailable,
        None,
        ProviderTimeObservation::default(),
        instant(1),
        &mut FailingBackoff,
    );
    let mut debug = DebugBuffer::new();
    assert!(write!(&mut debug, "{error:?}").is_ok());
    assert_eq!(debug.as_str(), "Err(Backoff([redacted]))");
    assert!(!debug.as_str().contains("customer-payload"));
    assert!(poller.is_terminal());
}

struct FixedBackoff(MonotonicDuration);

impl PollBackoff for FixedBackoff {
    type Error = core::convert::Infallible;

    fn delay(&mut self, _context: PollContext) -> Result<MonotonicDuration, Self::Error> {
        Ok(self.0)
    }
}

#[test]
fn busy_loop_delay_and_cumulative_budgets_fail_closed() {
    for (delay, expected) in [
        (duration(0), ActionPollError::ZeroDelay),
        (duration(9), ActionPollError::DelayLimitExceeded),
    ] {
        let mut poller = ActionPoller::new(limits(3), ProgressPolicy::Unordered, instant(0));
        assert!(
            poller
                .next_request(PollControl::Continue, instant(0))
                .is_ok()
        );
        assert_eq!(
            poller.observe(
                ActionUpdate::<()>::Running,
                ProgressObservation::Unavailable,
                None,
                ProviderTimeObservation::default(),
                instant(1),
                &mut FixedBackoff(delay),
            ),
            Err(ActionObserveError::Driver(expected))
        );
        assert!(poller.is_terminal());
    }

    let tight = ActionPollLimits::new(4, duration(2), duration(3), duration(20))
        .unwrap_or_else(|_| unreachable!());
    let mut poller = ActionPoller::new(tight, ProgressPolicy::Unordered, instant(0));
    let mut fixed = FixedBackoff(duration(2));
    assert!(
        poller
            .next_request(PollControl::Continue, instant(0))
            .is_ok()
    );
    assert!(
        poller
            .observe(
                ActionUpdate::<()>::Running,
                ProgressObservation::Unavailable,
                None,
                ProviderTimeObservation::default(),
                instant(1),
                &mut fixed,
            )
            .is_ok()
    );
    assert!(
        poller
            .next_request(PollControl::Continue, instant(3))
            .is_ok()
    );
    assert_eq!(
        poller.observe(
            ActionUpdate::<()>::Running,
            ProgressObservation::Unavailable,
            None,
            ProviderTimeObservation::default(),
            instant(4),
            &mut fixed,
        ),
        Err(ActionObserveError::Driver(
            ActionPollError::CumulativeDelayExceeded
        ))
    );
}

#[test]
fn delay_cannot_reach_elapsed_deadline() {
    let short = ActionPollLimits::new(3, duration(8), duration(20), duration(10))
        .unwrap_or_else(|_| unreachable!());
    let mut poller = ActionPoller::new(short, ProgressPolicy::Unordered, instant(0));
    assert!(
        poller
            .next_request(PollControl::Continue, instant(0))
            .is_ok()
    );
    assert_eq!(
        poller.observe(
            ActionUpdate::<()>::Running,
            ProgressObservation::Unavailable,
            None,
            ProviderTimeObservation::default(),
            instant(3),
            &mut FixedBackoff(duration(7)),
        ),
        Err(ActionObserveError::Driver(
            ActionPollError::ElapsedBudgetExceeded
        ))
    );
    assert!(poller.is_terminal());
}

#[test]
fn constructors_reject_incoherent_limits_and_provider_time() {
    assert_eq!(
        ActionPollLimits::new(0, duration(1), duration(1), duration(2)),
        Err(ActionPollLimitsError::Zero)
    );
    assert_eq!(
        ActionPollLimits::new(1, duration(3), duration(2), duration(4)),
        Err(ActionPollLimitsError::DelayExceedsCumulative)
    );
    assert_eq!(
        ActionPollLimits::new(1, duration(3), duration(3), duration(3)),
        Err(ActionPollLimitsError::DelayExceedsElapsed)
    );
    assert_eq!(
        ExponentialBackoff::new(duration(0), duration(1), 2),
        Err(ExponentialBackoffError::ZeroDelay)
    );
    assert_eq!(
        ExponentialBackoff::new(duration(2), duration(1), 2),
        Err(ExponentialBackoffError::InitialExceedsMaximum)
    );
    assert_eq!(
        ExponentialBackoff::new(duration(1), duration(2), 0),
        Err(ExponentialBackoffError::InvalidMultiplier)
    );
    assert_eq!(
        ProviderTimeObservation::new(
            Some(WallClockTimestamp::new(2)),
            Some(WallClockTimestamp::new(1)),
        ),
        Err(ProviderTimeError::ExpiryBeforeObservation)
    );
}

#[test]
fn exponential_backoff_caps_and_resets_after_progress() {
    let wide = ActionPollLimits::new(5, duration(8), duration(30), duration(50))
        .unwrap_or_else(|_| unreachable!());
    let mut poller = ActionPoller::new(wide, ProgressPolicy::Nondecreasing, instant(0));
    let mut backoff = make_backoff();

    for (request_at, observed_at, progress, expected_delay) in
        [(0, 1, 10, 2), (3, 4, 10, 4), (8, 9, 10, 8), (17, 18, 20, 2)]
    {
        assert_eq!(
            poller.next_request(PollControl::Continue, instant(request_at)),
            Ok(PollRequestStep::Request)
        );
        assert_eq!(
            poller.observe(
                ActionUpdate::<()>::Running,
                ProgressObservation::Percent(progress),
                None,
                ProviderTimeObservation::default(),
                instant(observed_at),
                &mut backoff,
            ),
            Ok(ActionPollStep::Delay(duration(expected_delay)))
        );
    }
}
