use alloc::format;
use alloc::vec::Vec;

use super::super::{
    IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonProgress,
    IncrementalJsonVisitor, VisitControl,
};
use super::support::{Collector, decode, decode_at_splits};

fn valid_events(
    result: Result<
        Vec<super::support::OwnedEvent>,
        IncrementalJsonError<core::convert::Infallible>,
    >,
) -> Vec<super::support::OwnedEvent> {
    assert!(result.is_ok());
    result.unwrap_or_default()
}

#[test]
fn every_general_boundary_matches_one_shot_events() {
    let input =
        br#" {"id":42,"name":"line\nquote:\"","ok":true,"none":null,"items":[-0,1.5e+2,{}]} "#;
    let expected = valid_events(decode(input));
    for split in 0..=input.len() {
        assert_eq!(
            decode_at_splits(input, &[split]),
            Ok(expected.clone()),
            "split {split}"
        );
    }
    let every_byte: Vec<_> = (0..input.len()).collect();
    assert_eq!(decode_at_splits(input, &every_byte), Ok(expected));
}

#[test]
fn every_utf8_and_escape_boundary_is_validated() {
    let input = "{\"raw\":\"aé☃𝄞z\",\"escaped\":\"\\u00e9\\u2603\\uD834\\uDD1E\"}";
    let expected = valid_events(decode(input.as_bytes()));
    for split in 0..=input.len() {
        assert_eq!(
            decode_at_splits(input.as_bytes(), &[split]),
            Ok(expected.clone()),
            "UTF-8 split {split}"
        );
    }
}

#[test]
fn duplicate_keys_are_rejected_across_every_boundary() {
    let input = br#"{"se\u0063ret":1,"secret":2}"#;
    for split in 0..=input.len() {
        assert!(matches!(
            decode_at_splits(input, &[split]),
            Err(IncrementalJsonError::DuplicateKey)
        ));
    }
}

#[test]
fn every_truncated_prefix_fails_closed() {
    let input = "{\"items\":[true,false,null,12.5e-2,\"é\\n\\u2603\"],\"tail\":{}}".as_bytes();
    assert!(decode(input).is_ok());
    for end in 0..input.len() {
        let mut decoder = IncrementalJsonDecoder::new();
        let mut visitor = Collector::default();
        assert_eq!(
            decoder.push(input.get(..end).unwrap_or_default(), &mut visitor),
            Ok(IncrementalJsonProgress::Pending)
        );
        assert!(decoder.finish(&mut visitor).is_err(), "prefix {end}");
    }
}

struct StopAfter {
    remaining: usize,
}

impl IncrementalJsonVisitor for StopAfter {
    type Error = core::convert::Infallible;

    fn visit(&mut self, _event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
        self.remaining = self.remaining.saturating_sub(1);
        Ok(if self.remaining == 0 {
            VisitControl::Stop
        } else {
            VisitControl::Continue
        })
    }
}

#[test]
fn early_stop_never_claims_complete_document_validation() {
    let mut decoder = IncrementalJsonDecoder::new();
    let mut visitor = StopAfter { remaining: 2 };
    let input = br#"{"secret":"not parsed after stop" BROKEN"#;
    assert_eq!(
        decoder.push(input, &mut visitor),
        Ok(IncrementalJsonProgress::Stopped)
    );
    assert_eq!(
        decoder.finish(&mut visitor),
        Ok(IncrementalJsonProgress::Stopped)
    );
    assert_eq!(
        decoder.push(b"null", &mut visitor),
        Ok(IncrementalJsonProgress::Stopped)
    );
}

#[test]
fn visitor_errors_are_recoverable_but_redacted_from_diagnostics() {
    struct Failing;
    impl IncrementalJsonVisitor for Failing {
        type Error = &'static str;

        fn visit(&mut self, _event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
            Err("sensitive visitor payload")
        }
    }

    let mut decoder = IncrementalJsonDecoder::new();
    let result = decoder.push(b"null", &mut Failing);
    assert!(result.is_err(), "visitor failure must propagate");
    let error = match result {
        Err(error) => error,
        Ok(_) => return,
    };
    assert!(!format!("{error:?}").contains("sensitive visitor payload"));
    assert_eq!(
        error.into_visitor_error(),
        Some("sensitive visitor payload")
    );
}

#[test]
fn payload_bearing_events_have_redacted_debug_output() {
    for event in [
        IncrementalJsonEvent::Key("sensitive-key"),
        IncrementalJsonEvent::StringFragment("sensitive-value"),
        IncrementalJsonEvent::Number("123456789"),
    ] {
        let debug = format!("{event:?}");
        assert!(debug.contains("redacted"));
        assert!(!debug.contains("sensitive"));
        assert!(!debug.contains("123456789"));
    }
}
