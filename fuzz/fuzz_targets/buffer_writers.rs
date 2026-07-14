#![no_main]

use cloud_sdk::buffer::{write_json_string, write_percent_encoded, write_u64};
use libfuzzer_sys::fuzz_target;

#[derive(Clone, Copy)]
struct TooSmall;

fuzz_target!(|data: &[u8]| {
    let number = read_u64(data);
    let mut decimal = [0_u8; 20];
    let mut decimal_len = 0;
    let result = write_u64(&mut decimal, &mut decimal_len, number, TooSmall);
    assert!(result.is_ok());
    assert_eq!(
        core::str::from_utf8(&decimal[..decimal_len]).ok(),
        Some(number.to_string().as_str())
    );

    let Ok(text) = core::str::from_utf8(data) else {
        return;
    };

    let capacity = usize::from(data.first().copied().unwrap_or(0))
        .saturating_mul(32)
        .min(8_192);
    let mut json = vec![0xa5_u8; capacity];
    let before = json.clone();
    let mut json_len = 0;
    match write_json_string(&mut json, &mut json_len, text, TooSmall) {
        Ok(()) => {
            let decoded = serde_json::from_slice::<String>(&json[..json_len]);
            assert!(decoded.as_deref().is_ok_and(|value| value == text));
        }
        Err(_) => {
            assert_eq!(json_len, 0);
            assert_eq!(json, before);
        }
    }

    let encoded_capacity = text.len().saturating_mul(3);
    let mut encoded = vec![0_u8; encoded_capacity];
    let mut encoded_len = 0;
    let result = write_percent_encoded(&mut encoded, &mut encoded_len, text, TooSmall);
    assert!(result.is_ok());
    assert!(encoded_len <= encoded.len());
    assert!(encoded[..encoded_len].iter().all(u8::is_ascii));
});

fn read_u64(data: &[u8]) -> u64 {
    let mut bytes = [0_u8; 8];
    for (target, source) in bytes.iter_mut().zip(data.iter().copied()) {
        *target = source;
    }
    u64::from_le_bytes(bytes)
}
