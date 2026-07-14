#![no_main]

use cloud_sdk_hetzner::{
    dns::{
        rrsets::{RecordComment, RecordValue, RrsetEndpoint, RrsetName, RrsetReference, RrsetType},
        zones::{ZoneEndpoint, ZoneName, ZoneReference},
    },
    labels::{LabelKey, LabelSelector, LabelValue},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };
    let mut fields = text.splitn(6, '\n');
    let zone_text = fields.next().unwrap_or_default();
    let rrset_text = fields.next().unwrap_or_default();
    let record_text = fields.next().unwrap_or_default();
    let selector_text = fields.next().unwrap_or_default();
    let comment_text = fields.next().unwrap_or_default();

    let _ = LabelKey::new(selector_text);
    let _ = LabelValue::new(selector_text);
    let _ = LabelSelector::new(selector_text);

    if let Ok(zone) = ZoneName::new(zone_text) {
        let reference = ZoneReference::Name(zone);
        check_path(ZoneEndpoint::Get(reference).write_path(&mut [0_u8; 1_024]));

        if let Ok(name) = RrsetName::new(rrset_text) {
            let rrset = RrsetReference::new(reference, name, RrsetType::Txt);
            check_path(RrsetEndpoint::Get(rrset).write_path(&mut [0_u8; 1_024]));
        }
    }

    if let Ok(value) = RecordValue::new(record_text) {
        check_json_write(record_text, |output| value.write_json_string(output));
    }
    if let Ok(comment) = RecordComment::new(comment_text) {
        check_json_write(comment_text, |output| comment.write_json_string(output));
    }
});

fn check_path<E: core::fmt::Debug>(result: Result<usize, E>) {
    if let Ok(len) = result {
        assert!(len <= 1_024);
    }
}

fn check_json_write<E, F>(expected: &str, write: F)
where
    E: core::fmt::Debug,
    F: FnOnce(&mut [u8]) -> Result<usize, E>,
{
    let capacity = expected.len().saturating_mul(2).min(4_096);
    let mut output = vec![0xa5_u8; capacity];
    let before = output.clone();
    match write(&mut output) {
        Ok(len) => {
            let parsed = serde_json::from_slice::<String>(&output[..len]);
            assert!(parsed.as_deref().is_ok_and(|value| value == expected));
        }
        Err(_) => assert_eq!(output, before),
    }
}
