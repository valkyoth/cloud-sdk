//! Decodes one action response and applies caller-owned polling policy.

use core::time::Duration;

use cloud_sdk::action_polling::{
    ActionPollStep, ActionPoller, PollContext, PollDecision, PollPolicy,
};
use cloud_sdk_hetzner::serde::ActionEnvelope;

struct FixedDelay;

impl PollPolicy for FixedDelay {
    type Error = ();

    fn decide(&mut self, _context: PollContext) -> Result<PollDecision, Self::Error> {
        Ok(PollDecision::Delay(Duration::from_secs(2)))
    }
}

fn main() {
    let body = br#"{"action":{
        "id":42,
        "command":"create_server",
        "status":"running",
        "progress":25,
        "started":"2026-07-13T12:00:00Z",
        "finished":null,
        "resources":[],
        "error":null
    }}"#;
    let Ok(envelope) = serde_json::from_slice::<ActionEnvelope<'_>>(body) else {
        return;
    };
    let mut poller = ActionPoller::new();
    let mut policy = FixedDelay;
    let step = poller.observe(
        envelope.action().polling_update(),
        envelope.action().progress(),
        None,
        &mut policy,
    );

    assert_eq!(step, Ok(ActionPollStep::Delay(Duration::from_secs(2))));
}
