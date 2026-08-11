#![no_main]

use cloud_sdk_hetzner::robot::{RobotForm, RobotFormError, RobotFormField};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(value) = core::str::from_utf8(data) else {
        return;
    };
    let fields = [
        RobotFormField::public("server[]", value).unwrap_or_else(|_| unreachable!()),
        RobotFormField::sensitive("rules[input][0][name]", value)
            .unwrap_or_else(|_| unreachable!()),
    ];
    let form = RobotForm::new(&fields).unwrap_or_else(|_| unreachable!());
    let expected = reference_form(value);
    assert_eq!(form.encoded_len(), Ok(expected.len()));

    let capacity = usize::from(data.first().copied().unwrap_or(0))
        .saturating_mul(64)
        .min(16_384);
    let mut output = vec![0xa5_u8; capacity];
    let before = output.clone();
    let buffer_too_small = match form.encode(&mut output) {
        Ok(encoded) => {
            assert_eq!(encoded.as_bytes(), expected);
            false
        }
        Err(RobotFormError::BufferTooSmall) => true,
        Err(error) => panic!("bounded valid form failed unexpectedly: {error}"),
    };
    if buffer_too_small {
        assert_eq!(output, before);
    } else {
        assert!(output.iter().all(|byte| *byte == 0));
    }
});

fn reference_form(value: &str) -> Vec<u8> {
    let mut output = b"server%5B%5D=".to_vec();
    reference_component(value, &mut output);
    output.extend_from_slice(b"&rules%5Binput%5D%5B0%5D%5Bname%5D=");
    reference_component(value, &mut output);
    output
}

fn reference_component(value: &str, output: &mut Vec<u8>) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for byte in value.bytes() {
        match byte {
            b' ' => output.push(b'+'),
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                output.push(byte);
            }
            _ => {
                output.push(b'%');
                output.push(HEX[usize::from(byte >> 4)]);
                output.push(HEX[usize::from(byte & 0x0f)]);
            }
        }
    }
}
