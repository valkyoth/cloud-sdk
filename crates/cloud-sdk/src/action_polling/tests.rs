use core::time::Duration;

use super::{
    ActionPollError, ActionPollStep, ActionPoller, ActionUpdate, PollContext, PollDecision,
    PollPolicy,
};
use crate::rate_limit::RateLimit;

struct Policy {
    decision: PollDecision,
    context: Option<PollContext>,
}

impl PollPolicy for Policy {
    type Error = ();

    fn decide(&mut self, context: PollContext) -> Result<PollDecision, Self::Error> {
        self.context = Some(context);
        Ok(self.decision)
    }
}

struct FailingPolicy;

impl PollPolicy for FailingPolicy {
    type Error = u8;

    fn decide(&mut self, _context: PollContext) -> Result<PollDecision, Self::Error> {
        Err(7)
    }
}

#[test]
fn caller_policy_receives_progress_and_rate_limit() {
    let rate_limit = RateLimit::new(3600, 3, 42).ok();
    let mut policy = Policy {
        decision: PollDecision::Delay(Duration::from_secs(2)),
        context: None,
    };
    let mut poller = ActionPoller::new();
    assert_eq!(
        poller.observe(ActionUpdate::<()>::Running, 25, rate_limit, &mut policy),
        Ok(ActionPollStep::Delay(Duration::from_secs(2)))
    );
    assert_eq!(
        policy.context,
        Some(PollContext {
            observation: 1,
            progress: 25,
            rate_limit,
        })
    );
    assert!(!poller.is_terminal());
}

#[test]
fn terminal_success_and_failure_bypass_delay_policy() {
    let mut policy = Policy {
        decision: PollDecision::Delay(Duration::from_secs(1)),
        context: None,
    };
    let mut succeeded = ActionPoller::new();
    assert_eq!(
        succeeded.observe(ActionUpdate::<u8>::Success, 100, None, &mut policy),
        Ok(ActionPollStep::Complete)
    );
    assert!(succeeded.is_terminal());
    assert_eq!(policy.context, None);

    let mut failed = ActionPoller::new();
    assert_eq!(
        failed.observe(ActionUpdate::Failed(7_u8), 60, None, &mut policy),
        Ok(ActionPollStep::Failed(7))
    );
    assert!(failed.is_terminal());
}

#[test]
fn caller_policy_controls_cancel_and_timeout() {
    for (decision, expected) in [
        (PollDecision::Cancel, ActionPollStep::Cancelled),
        (PollDecision::Timeout, ActionPollStep::TimedOut),
    ] {
        let mut policy = Policy {
            decision,
            context: None,
        };
        let mut poller = ActionPoller::new();
        assert_eq!(
            poller.observe(ActionUpdate::<()>::Running, 0, None, &mut policy),
            Ok(expected)
        );
        assert!(poller.is_terminal());
    }
}

#[test]
fn policy_errors_are_preserved_without_advancing_the_poller() {
    let mut poller = ActionPoller::new();
    assert_eq!(
        poller.observe(ActionUpdate::<()>::Running, 10, None, &mut FailingPolicy,),
        Err(ActionPollError::Policy(7))
    );
    assert_eq!(poller.observations(), 0);
    assert!(!poller.is_terminal());
}

#[test]
fn rejects_busy_loops_progress_regression_and_post_terminal_polling() {
    let mut zero = Policy {
        decision: PollDecision::Delay(Duration::ZERO),
        context: None,
    };
    let mut poller = ActionPoller::new();
    assert_eq!(
        poller.observe(ActionUpdate::<()>::Running, 10, None, &mut zero),
        Err(ActionPollError::ZeroDelay)
    );
    assert_eq!(poller.observations(), 0);

    zero.decision = PollDecision::Delay(Duration::from_secs(1));
    assert!(
        poller
            .observe(ActionUpdate::<()>::Running, 10, None, &mut zero)
            .is_ok()
    );
    assert_eq!(
        poller.observe(ActionUpdate::<()>::Running, 9, None, &mut zero),
        Err(ActionPollError::ProgressRegressed)
    );
    assert_eq!(poller.observations(), 1);

    assert!(
        poller
            .observe(ActionUpdate::<()>::Success, 100, None, &mut zero)
            .is_ok()
    );
    assert_eq!(
        poller.observe(ActionUpdate::<()>::Success, 100, None, &mut zero),
        Err(ActionPollError::Terminal)
    );
}
