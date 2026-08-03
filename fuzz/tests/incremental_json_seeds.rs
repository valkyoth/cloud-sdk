use core::convert::Infallible;

use cloud_sdk_hetzner::serde::{
    IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonProgress,
    IncrementalJsonVisitor, VisitControl,
};

const VALID_SEED: &[u8] = include_bytes!("../seeds/incremental_json/valid.seed");
const DUPLICATE_SEED: &[u8] = include_bytes!("../seeds/incremental_json/duplicate.seed");
const CONTROL_BYTES: [u8; 2] = [b'A', b'!'];

struct Continue;

impl IncrementalJsonVisitor for Continue {
    type Error = Infallible;

    fn visit(&mut self, _event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
        Ok(VisitControl::Continue)
    }
}

fn split_seed(seed: &[u8]) -> (&[u8], &[u8]) {
    let (controls, payload) = seed.split_at(2);
    assert_eq!(controls, CONTROL_BYTES);
    assert!(!payload.is_empty());
    (controls, payload)
}

#[test]
fn valid_seed_reaches_complete_incremental_and_independent_parsers() {
    let (_, payload) = split_seed(VALID_SEED);
    assert!(serde_json::from_slice::<serde_json::Value>(payload).is_ok());

    let mut decoder = IncrementalJsonDecoder::new();
    assert_eq!(
        decoder.push(payload, &mut Continue),
        Ok(IncrementalJsonProgress::Pending)
    );
    assert_eq!(
        decoder.finish(&mut Continue),
        Ok(IncrementalJsonProgress::Complete)
    );
}

#[test]
fn duplicate_seed_reaches_duplicate_key_rejection() {
    let (_, payload) = split_seed(DUPLICATE_SEED);
    assert!(serde_json::from_slice::<serde_json::Value>(payload).is_ok());

    let mut decoder = IncrementalJsonDecoder::new();
    assert!(matches!(
        decoder.push(payload, &mut Continue),
        Err(IncrementalJsonError::DuplicateKey)
    ));
}
