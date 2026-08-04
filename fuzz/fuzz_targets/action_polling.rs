#![no_main]

use cloud_sdk::action_polling::{
    ActionPollLimits, ActionPoller, ActionUpdate, PollBackoff, PollContext, PollControl,
    PollRequestStep, ProgressObservation, ProgressPolicy, ProviderTimeObservation,
};
use cloud_sdk::retry::{MonotonicDuration, MonotonicInstant};
use libfuzzer_sys::fuzz_target;

struct InputBackoff(u8);

impl PollBackoff for InputBackoff {
    type Error = ();

    fn delay(&mut self, _context: PollContext) -> Result<MonotonicDuration, Self::Error> {
        Ok(MonotonicDuration::new(u64::from(self.0)))
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(limits) = ActionPollLimits::new(
        32,
        MonotonicDuration::new(32),
        MonotonicDuration::new(256),
        MonotonicDuration::new(1_024),
    ) else {
        return;
    };
    let mut poller = ActionPoller::new(
        limits,
        ProgressPolicy::ExplicitResets { max_resets: 4 },
        MonotonicInstant::new(0),
    );
    let mut now = 0_u64;
    for chunk in data.chunks(5).take(128) {
        let control = if chunk.first().copied().unwrap_or(0) & 1 == 0 {
            PollControl::Continue
        } else {
            PollControl::Cancel
        };
        match poller.next_request(control, MonotonicInstant::new(now)) {
            Ok(PollRequestStep::Delay(delay)) => {
                now = now.saturating_add(delay.get());
                continue;
            }
            Ok(PollRequestStep::Request) => {}
            _ => break,
        }
        let update = match chunk.get(1).copied().unwrap_or(0) % 3 {
            0 => ActionUpdate::Running,
            1 => ActionUpdate::Success,
            _ => ActionUpdate::Failed(7_u8),
        };
        let value = chunk.get(2).copied().unwrap_or(0);
        let progress = match chunk.get(3).copied().unwrap_or(0) % 3 {
            0 => ProgressObservation::Unavailable,
            1 => ProgressObservation::Percent(value),
            _ => ProgressObservation::Reset(value),
        };
        now = now.saturating_add(1);
        let mut backoff = InputBackoff(chunk.get(4).copied().unwrap_or(0));
        let _ = poller.observe(
            update,
            progress,
            None,
            ProviderTimeObservation::default(),
            MonotonicInstant::new(now),
            &mut backoff,
        );
        if poller.is_terminal() {
            break;
        }
    }
});
