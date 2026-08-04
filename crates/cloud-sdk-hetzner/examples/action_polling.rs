//! Decodes one action response and advances bounded caller-owned polling.

use cloud_sdk::action_polling::{
    ActionPollLimits, ActionPollStep, ActionPoller, ExponentialBackoff, PollControl,
    PollRequestStep, ProgressObservation, ProgressPolicy, ProviderTimeObservation,
};
use cloud_sdk::retry::{MonotonicDuration, MonotonicInstant};
use cloud_sdk_hetzner::serde::ActionEnvelope;

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
    let Ok(limits) = ActionPollLimits::new(
        60,
        MonotonicDuration::new(8_000),
        MonotonicDuration::new(120_000),
        MonotonicDuration::new(300_000),
    ) else {
        return;
    };
    let Ok(mut backoff) = ExponentialBackoff::new(
        MonotonicDuration::new(2_000),
        MonotonicDuration::new(8_000),
        2,
    ) else {
        return;
    };
    let mut poller = ActionPoller::new(
        limits,
        ProgressPolicy::Nondecreasing,
        MonotonicInstant::new(0),
    );
    assert_eq!(
        poller.next_request(PollControl::Continue, MonotonicInstant::new(0)),
        Ok(PollRequestStep::Request)
    );
    let step = poller.observe(
        envelope.action().polling_update(),
        ProgressObservation::Percent(envelope.action().progress()),
        None,
        ProviderTimeObservation::default(),
        MonotonicInstant::new(10),
        &mut backoff,
    );

    assert_eq!(
        step,
        Ok(ActionPollStep::Delay(MonotonicDuration::new(2_000)))
    );
}
