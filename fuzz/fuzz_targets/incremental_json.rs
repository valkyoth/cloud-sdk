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
    let seed = data.first().copied().map_or(1, usize::from);
    let stop_after = data
        .get(1)
        .copied()
        .filter(|value| *value != 0)
        .map(usize::from);
    let one_shot = decode(data, 0, seed, stop_after);
    let bytewise = decode(data, 1, seed, stop_after);
    let variable = decode(data, 2, seed, stop_after);

    assert_eq!(one_shot, bytewise);
    assert_eq!(one_shot, variable);
});
