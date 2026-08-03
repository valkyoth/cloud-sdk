#![no_main]

use core::convert::Infallible;

use cloud_sdk_hetzner::serde::{
    IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonProgress,
    IncrementalJsonVisitor, VisitControl,
};
use libfuzzer_sys::fuzz_target;

#[derive(Default)]
struct Visitor {
    events: usize,
    text_bytes: usize,
    stop_after: Option<usize>,
}

impl IncrementalJsonVisitor for Visitor {
    type Error = Infallible;

    fn visit(&mut self, event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
        self.events = self.events.saturating_add(1);
        self.text_bytes = self.text_bytes.saturating_add(match event {
            IncrementalJsonEvent::Key(text)
            | IncrementalJsonEvent::StringFragment(text)
            | IncrementalJsonEvent::Number(text) => text.len(),
            _ => 0,
        });
        Ok(if self.stop_after == Some(self.events) {
            VisitControl::Stop
        } else {
            VisitControl::Continue
        })
    }
}

type Outcome = Result<IncrementalJsonProgress, IncrementalJsonError<Infallible>>;

fn decode(
    input: &[u8],
    mode: u8,
    chunk_seed: usize,
    stop_after: Option<usize>,
) -> (Outcome, usize, usize) {
    let mut decoder = IncrementalJsonDecoder::new();
    let mut visitor = Visitor {
        events: 0,
        text_bytes: 0,
        stop_after,
    };
    let mut position = 0_usize;
    let mut outcome = Ok(IncrementalJsonProgress::Pending);
    while position < input.len() {
        let remaining = input.len().saturating_sub(position);
        let width = match mode {
            0 => remaining,
            1 => 1,
            _ => chunk_seed
                .wrapping_add(position)
                .checked_rem(31)
                .unwrap_or(0)
                .saturating_add(1)
                .min(remaining),
        };
        let end = position.saturating_add(width);
        let Some(chunk) = input.get(position..end) else {
            break;
        };
        outcome = decoder.push(chunk, &mut visitor);
        if outcome != Ok(IncrementalJsonProgress::Pending) {
            break;
        }
        position = end;
    }
    if outcome == Ok(IncrementalJsonProgress::Pending) {
        outcome = decoder.finish(&mut visitor);
    }
    (outcome, visitor.events, visitor.text_bytes)
}

fuzz_target!(|data: &[u8]| {
    let Some((&seed, controlled)) = data.split_first() else {
        return;
    };
    let Some((&stop_control, payload)) = controlled.split_first() else {
        return;
    };
    let seed = usize::from(seed);
    let stop_after = (stop_control != 0).then_some(usize::from(stop_control));

    let complete = decode(payload, 0, seed, None);
    if complete.0 == Ok(IncrementalJsonProgress::Complete) {
        assert!(
            serde_json::from_slice::<serde_json::Value>(payload).is_ok(),
            "incremental decoder accepted invalid JSON"
        );
    }

    let one_shot = decode(payload, 0, seed, stop_after);
    let bytewise = decode(payload, 1, seed, stop_after);
    let variable = decode(payload, 2, seed, stop_after);

    assert_eq!(one_shot, bytewise);
    assert_eq!(one_shot, variable);
});
