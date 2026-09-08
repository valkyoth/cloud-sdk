use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::Infallible;

use super::super::{
    IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonLimits,
    IncrementalJsonProgress, IncrementalJsonVisitor, VisitControl,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum OwnedEvent {
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    Key(String),
    StringStart,
    StringFragment(String),
    StringEnd,
    Number(String),
    Bool(bool),
    Null,
}

#[derive(Default)]
pub(super) struct Collector {
    pub(super) events: Vec<OwnedEvent>,
}

impl IncrementalJsonVisitor for Collector {
    type Error = Infallible;

    fn visit(&mut self, event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
        self.events.push(match event {
            IncrementalJsonEvent::StartObject => OwnedEvent::StartObject,
            IncrementalJsonEvent::EndObject => OwnedEvent::EndObject,
            IncrementalJsonEvent::StartArray => OwnedEvent::StartArray,
            IncrementalJsonEvent::EndArray => OwnedEvent::EndArray,
            IncrementalJsonEvent::Key(value) => OwnedEvent::Key(value.to_string()),
            IncrementalJsonEvent::StringStart => OwnedEvent::StringStart,
            IncrementalJsonEvent::StringFragment(value) => {
                OwnedEvent::StringFragment(value.to_string())
            }
            IncrementalJsonEvent::StringEnd => OwnedEvent::StringEnd,
            IncrementalJsonEvent::Number(value) => OwnedEvent::Number(value.to_string()),
            IncrementalJsonEvent::Bool(value) => OwnedEvent::Bool(value),
            IncrementalJsonEvent::Null => OwnedEvent::Null,
        });
        Ok(VisitControl::Continue)
    }
}

pub(super) fn decode(input: &[u8]) -> Result<Vec<OwnedEvent>, IncrementalJsonError<Infallible>> {
    decode_with_limits(input, IncrementalJsonLimits::DEFAULT)
}

pub(super) fn decode_with_limits(
    input: &[u8],
    limits: IncrementalJsonLimits,
) -> Result<Vec<OwnedEvent>, IncrementalJsonError<Infallible>> {
    let mut decoder = IncrementalJsonDecoder::with_limits(limits);
    let mut collector = Collector::default();
    assert_eq!(
        decoder.push(input, &mut collector)?,
        IncrementalJsonProgress::Pending
    );
    assert_eq!(
        decoder.finish(&mut collector)?,
        IncrementalJsonProgress::Complete
    );
    Ok(collector.events)
}

pub(super) fn decode_at_splits(
    input: &[u8],
    splits: &[usize],
) -> Result<Vec<OwnedEvent>, IncrementalJsonError<Infallible>> {
    let mut decoder = IncrementalJsonDecoder::new();
    let mut collector = Collector::default();
    let mut start = 0;
    for end in splits.iter().copied().chain(core::iter::once(input.len())) {
        let chunk = input
            .get(start..end)
            .ok_or(IncrementalJsonError::InvalidSyntax)?;
        decoder.push(chunk, &mut collector)?;
        start = end;
    }
    assert_eq!(
        decoder.finish(&mut collector)?,
        IncrementalJsonProgress::Complete
    );
    Ok(collector.events)
}
