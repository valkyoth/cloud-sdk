use alloc::{format, vec, vec::Vec};

use super::{
    MAX_ROBOT_FORM_BODY_BYTES, MAX_ROBOT_FORM_FIELDS, MAX_ROBOT_FORM_NAME_BYTES,
    MAX_ROBOT_FORM_VALUE_BYTES, RobotForm, RobotFormError, RobotFormField, RobotFormSensitivity,
};

fn field<'a>(name: &'a str, value: &'a str) -> RobotFormField<'a> {
    RobotFormField::public(name, value)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"))
}

#[test]
fn source_locked_repeated_server_fields_match_robot_wire() {
    let fields = [
        field("server[]", "123.123.123.123"),
        field("server[]", "123.123.123.124"),
    ];
    let form = RobotForm::new(&fields)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let mut output = [0xa5_u8; 128];

    {
        let encoded = form
            .encode(&mut output)
            .unwrap_or_else(|_| unreachable!("source-locked Robot form was rejected"));
        assert_eq!(
            encoded.as_bytes(),
            b"server%5B%5D=123.123.123.123&server%5B%5D=123.123.123.124"
        );
        assert_eq!(encoded.len(), form.encoded_len().unwrap_or_default());
        assert_eq!(encoded.sensitivity(), RobotFormSensitivity::Public);
    }
    assert!(output.iter().all(|byte| *byte == 0));
}

#[test]
fn form_grammar_handles_spaces_separators_controls_and_utf8() {
    let fields = [field("value", "AZaz09 *-._~+&=\0\né")];
    let form = RobotForm::new(&fields)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let mut output = [0_u8; 128];
    let encoded = form
        .encode(&mut output)
        .unwrap_or_else(|_| unreachable!("valid UTF-8 form value was rejected"));

    assert_eq!(
        encoded.as_bytes(),
        b"value=AZaz09+*-._%7E%2B%26%3D%00%0A%C3%A9"
    );
}

#[test]
fn every_ascii_byte_uses_the_reviewed_form_grammar() {
    let input: Vec<u8> = (0_u8..=127).collect();
    let text = core::str::from_utf8(&input)
        .unwrap_or_else(|_| unreachable!("ASCII fixture must be UTF-8"));
    let fields = [field("all", text)];
    let form = RobotForm::new(&fields)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let mut output = [0_u8; 512];
    let encoded = form
        .encode(&mut output)
        .unwrap_or_else(|_| unreachable!("ASCII form corpus was rejected"));
    let expected = reference_form("all", text);

    assert_eq!(encoded.as_bytes(), expected);
}

#[test]
fn every_undersized_capacity_is_unchanged() {
    let fields = [field("password", "secret + value")];
    let form = RobotForm::new(&fields)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let required = form.encoded_len().unwrap_or_default();

    for capacity in 0..required {
        let mut output = [0xa5_u8; 64];
        let result = form.encode(output.get_mut(..capacity).unwrap_or_default());
        assert_eq!(result.err(), Some(RobotFormError::BufferTooSmall));
        assert_eq!(output, [0xa5; 64]);
    }
}

#[test]
fn admitted_write_clears_stale_tail_and_drop_clears_everything() {
    let fields = [RobotFormField::sensitive("password", "new")
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"))];
    let form = RobotForm::new(&fields)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let mut output = [0xa5_u8; 64];

    {
        let encoded = form
            .encode(&mut output)
            .unwrap_or_else(|_| unreachable!("sensitive form was rejected"));
        assert_eq!(encoded.as_bytes(), b"password=new");
        assert!(
            encoded
                .output
                .get(encoded.len()..)
                .is_some_and(|tail| tail.iter().all(|byte| *byte == 0))
        );
        assert_eq!(encoded.sensitivity(), RobotFormSensitivity::Sensitive);
        assert!(!format!("{encoded:?}").contains("new"));
    }
    assert!(output.iter().all(|byte| *byte == 0));
}

#[test]
fn empty_form_is_valid_and_still_owns_cleanup() {
    let form = RobotForm::new(&[])
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let mut output = [0xa5_u8; 8];
    {
        let encoded = form
            .encode(&mut output)
            .unwrap_or_else(|_| unreachable!("empty Robot form was rejected"));
        assert!(form.is_empty());
        assert_eq!(form.len(), 0);
        assert!(encoded.is_empty());
        assert_eq!(encoded.as_bytes(), b"");
        assert!(encoded.output.iter().all(|byte| *byte == 0));
    }
    assert_eq!(output, [0; 8]);
}

#[test]
fn names_and_values_enforce_exact_component_bounds() {
    let maximum_name = "a".repeat(MAX_ROBOT_FORM_NAME_BYTES);
    assert!(RobotFormField::public(&maximum_name, "value").is_ok());
    let long_name = "a".repeat(MAX_ROBOT_FORM_NAME_BYTES.saturating_add(1));
    assert_eq!(
        RobotFormField::public(&long_name, "value"),
        Err(RobotFormError::NameTooLong)
    );

    let maximum_value = "v".repeat(MAX_ROBOT_FORM_VALUE_BYTES);
    assert!(RobotFormField::public("data", &maximum_value).is_ok());
    let long_value = "v".repeat(MAX_ROBOT_FORM_VALUE_BYTES.saturating_add(1));
    assert_eq!(
        RobotFormField::public("data", &long_value),
        Err(RobotFormError::ValueTooLong)
    );
}

#[test]
fn field_count_and_name_grammar_fail_closed() {
    assert_eq!(
        RobotFormField::public("", "value"),
        Err(RobotFormError::EmptyName)
    );
    for invalid in [
        "white space",
        "field=value",
        "field&value",
        "ümlaut",
        "][",
        "[]",
        "a[",
        "a]",
        "a[b]tail",
        "a[[b]",
        "a[b]]",
    ] {
        assert_eq!(
            RobotFormField::public(invalid, "value"),
            Err(RobotFormError::InvalidName)
        );
    }
    assert!(RobotFormField::public("server[]", "192.0.2.1").is_ok());
    assert!(RobotFormField::public("rules[input][4095][src_ip]", "::1").is_ok());

    let template = field("server[]", "192.0.2.1");
    let too_many = vec![template; MAX_ROBOT_FORM_FIELDS.saturating_add(1)];
    assert_eq!(
        RobotForm::new(&too_many).err(),
        Some(RobotFormError::TooManyFields)
    );
}

#[test]
fn aggregate_body_cap_accepts_exact_and_rejects_plus_one_without_writing() {
    const VALUES: usize = 8;
    const OVERHEAD: usize = VALUES * 2 + (VALUES - 1);
    const LAST_VALUE: usize = MAX_ROBOT_FORM_VALUE_BYTES - OVERHEAD;

    let full = "v".repeat(MAX_ROBOT_FORM_VALUE_BYTES);
    let last = "v".repeat(LAST_VALUE);
    let fields = [
        field("a", &full),
        field("a", &full),
        field("a", &full),
        field("a", &full),
        field("a", &full),
        field("a", &full),
        field("a", &full),
        field("a", &last),
    ];
    let exact = RobotForm::new(&fields)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    assert_eq!(exact.encoded_len(), Ok(MAX_ROBOT_FORM_BODY_BYTES));

    let over_fields = [field("a", &full); VALUES];
    let over = RobotForm::new(&over_fields)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let mut output = [0xa5_u8; 16];
    assert_eq!(over.encoded_len(), Err(RobotFormError::BodyTooLong));
    assert_eq!(
        over.encode(&mut output).err(),
        Some(RobotFormError::BodyTooLong)
    );
    assert_eq!(output, [0xa5; 16]);
}

#[test]
fn field_and_form_debug_never_include_values() {
    let secret = RobotFormField::sensitive("password", "sentinel-secret")
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let fields = [secret];
    let form = RobotForm::new(&fields)
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));

    assert!(!format!("{secret:?}").contains("sentinel-secret"));
    assert!(!format!("{form:?}").contains("sentinel-secret"));
    assert_eq!(secret.name(), "password");
    assert_eq!(secret.sensitivity(), RobotFormSensitivity::Sensitive);
    assert_eq!(form.sensitivity(), RobotFormSensitivity::Sensitive);
}

fn reference_form(name: &str, value: &str) -> Vec<u8> {
    let mut output = Vec::new();
    reference_component(name, &mut output);
    output.push(b'=');
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
                output.push(HEX.get(usize::from(byte >> 4)).copied().unwrap_or_default());
                output.push(
                    HEX.get(usize::from(byte & 0x0f))
                        .copied()
                        .unwrap_or_default(),
                );
            }
        }
    }
}
