#[test]
fn security_sensitive_retry_tests_cannot_return_early() {
    for source in [
        include_str!("../tests.rs"),
        include_str!("permit_tests.rs"),
        include_str!("policy_identity_tests.rs"),
        include_str!("../fingerprint/tests.rs"),
        include_str!("../fingerprint/tests/endpoint_policy_tests.rs"),
    ] {
        for line in source.lines() {
            assert!(!contains_ignoring_ascii_whitespace(line, b"return;"));
            assert!(!contains_ignoring_ascii_whitespace(line, b"return}"));
            assert!(!contains_ignoring_ascii_whitespace(line, b"=>return,"));
        }
    }
}

fn contains_ignoring_ascii_whitespace(input: &str, needle: &[u8]) -> bool {
    let mut pattern = needle.iter().copied();
    let Some(first) = pattern.next() else {
        return false;
    };
    let mut expected = Some(first);
    for byte in input.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if Some(byte) == expected {
            expected = pattern.next();
        } else {
            pattern = needle.iter().copied();
            expected = pattern.next();
            if Some(byte) == expected {
                expected = pattern.next();
            }
        }
        if expected.is_none() {
            return true;
        }
    }
    false
}
