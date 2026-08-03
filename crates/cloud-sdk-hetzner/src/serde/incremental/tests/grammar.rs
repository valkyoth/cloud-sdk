use alloc::format;
use alloc::string::String;

use super::super::{IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonProgress};
use super::support::{Collector, decode};

#[test]
fn agrees_with_serde_json_on_valid_and_invalid_fixtures() {
    let valid: &[&[u8]] = &[
        b"null",
        b" true \r\n",
        br#"{"empty":{},"array":[],"number":-12.50E-3}"#,
        br#"["", "\\/\b\f\n\r\t", "\u0000", "\uD834\uDD1E"]"#,
        "{\"unicode\":\"é☃𝄞\"}".as_bytes(),
    ];
    for input in valid {
        assert!(serde_json::from_slice::<serde_json::Value>(input).is_ok());
        assert!(decode(input).is_ok(), "{}", String::from_utf8_lossy(input));
    }

    let invalid: &[&[u8]] = &[
        b"",
        b"nul",
        b"true false",
        b"[1,]",
        b"[,1]",
        br#"{"a":1,}"#,
        br#"{"a" 1}"#,
        br#""\uD800""#,
        br#""\uDC00""#,
        br#""\uD800\u0041""#,
        b"01",
        b"1.",
        b"1e",
        b"1e+",
        b"+1",
        &[b'"', 0xc0, 0x80, b'"'],
    ];
    for input in invalid {
        assert!(serde_json::from_slice::<serde_json::Value>(input).is_err());
        assert!(decode(input).is_err(), "{:?}", input);
    }
}

#[test]
fn rejects_duplicate_decoded_keys_at_each_nesting_level() {
    for input in [
        br#"{"a":1,"a":2}"#.as_slice(),
        br#"{"outer":{"a":1,"\u0061":2}}"#.as_slice(),
    ] {
        assert!(matches!(
            decode(input),
            Err(IncrementalJsonError::DuplicateKey)
        ));
    }
    assert!(decode(br#"[{"a":1},{"a":2}]"#).is_ok());
}

#[test]
fn long_keys_can_grow_protected_storage_and_remain_duplicate_checked() {
    let key = "k".repeat(96);
    let unique = format!(r#"{{"{key}":1}}"#);
    let duplicate = format!(r#"{{"{key}":1,"{key}":2}}"#);
    assert!(decode(unique.as_bytes()).is_ok());
    assert!(matches!(
        decode(duplicate.as_bytes()),
        Err(IncrementalJsonError::DuplicateKey)
    ));
}

#[test]
fn decoder_is_terminal_after_failure_and_completion() {
    let mut failed = IncrementalJsonDecoder::new();
    let mut collector = Collector::default();
    assert!(failed.push(b"[}", &mut collector).is_err());
    assert!(matches!(
        failed.push(b"null", &mut collector),
        Err(IncrementalJsonError::TerminalState)
    ));

    let mut complete = IncrementalJsonDecoder::new();
    assert_eq!(
        complete.push(b"null", &mut collector),
        Ok(IncrementalJsonProgress::Pending)
    );
    assert_eq!(
        complete.finish(&mut collector),
        Ok(IncrementalJsonProgress::Complete)
    );
    assert!(matches!(
        complete.finish(&mut collector),
        Err(IncrementalJsonError::TerminalState)
    ));
}

#[test]
fn debug_output_never_contains_staged_payloads() {
    let mut decoder = IncrementalJsonDecoder::new();
    let mut collector = Collector::default();
    assert_eq!(
        decoder.push(br#"{"password":"partial-secret"#.as_slice(), &mut collector),
        Ok(IncrementalJsonProgress::Pending)
    );
    let debug = format!("{decoder:?}");
    assert!(!debug.contains("password"));
    assert!(!debug.contains("partial-secret"));
    assert!(debug.contains("redacted"));
}

#[test]
fn completed_utf8_scratch_is_cleared_before_more_input() {
    use super::super::state::Lexical;

    let mut decoder = IncrementalJsonDecoder::new();
    let mut collector = Collector::default();
    assert_eq!(
        decoder.push("\"é".as_bytes(), &mut collector),
        Ok(IncrementalJsonProgress::Pending)
    );
    assert!(matches!(decoder.lexical, Some(Lexical::String(_))));
    let Some(Lexical::String(string)) = decoder.lexical.as_ref() else {
        return;
    };
    assert_eq!(string.utf8, [0; 4]);
    assert_eq!(string.utf8_len, 0);
}

#[test]
fn failure_clears_owned_staging_state() {
    let mut decoder = IncrementalJsonDecoder::new();
    let mut collector = Collector::default();
    assert!(decoder.push(br#"{"secret":12x"#, &mut collector).is_err());
    assert!(decoder.lexical.is_none());
    assert!(decoder.frames.is_empty());
}
