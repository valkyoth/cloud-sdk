#![no_main]

use core::time::Duration;

use cloud_sdk::action_polling::{
    ActionPollStep, ActionPoller, ActionUpdate, PollContext, PollDecision, PollPolicy,
};
use libfuzzer_sys::fuzz_target;

struct Policy {
    decision: u8,
    delay: u8,
}

impl PollPolicy for Policy {
    type Error = u8;

    fn decide(&mut self, _context: PollContext) -> Result<PollDecision, Self::Error> {
        match self.decision % 4 {
            0 => Ok(PollDecision::Delay(Duration::from_millis(u64::from(
                self.delay,
            )))),
            1 => Ok(PollDecision::Cancel),
            2 => Ok(PollDecision::Timeout),
            _ => Err(self.delay),
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let mut poller = ActionPoller::new();
    for chunk in data.chunks(4).take(128) {
        let update = match chunk.first().copied().unwrap_or(0) % 3 {
            0 => ActionUpdate::Running,
            1 => ActionUpdate::Success,
            _ => ActionUpdate::Failed(7_u8),
        };
        let progress = chunk.get(1).copied().unwrap_or(0);
        let mut policy = Policy {
            decision: chunk.get(2).copied().unwrap_or(0),
            delay: chunk.get(3).copied().unwrap_or(0),
        };
        let before = poller;
        let result = poller.observe(update, progress, None, &mut policy);
        if result.is_err() {
            assert_eq!(poller, before);
        } else if result
            .as_ref()
            .is_ok_and(|step| !matches!(step, ActionPollStep::Delay(_)))
        {
            assert!(poller.is_terminal());
        }
    }
});
