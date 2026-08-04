use super::{
    FingerprintBuildError, FingerprintScope, HEADERS, build_canonical_fingerprint, fixture,
    fixture_parts_endpoint, same_fingerprint,
};
use crate::Method;
use crate::transport::{EndpointIdentity, EndpointScheme};

#[test]
fn equivalent_ipv6_endpoint_spellings_have_one_fingerprint_identity() {
    let compact = EndpointIdentity::new(EndpointScheme::Https, "[2001:db8::1]", 443, "/v1");
    let expanded = EndpointIdentity::new(
        EndpointScheme::Https,
        "[2001:0db8:0000:0000:0000:0000:0000:0001]",
        443,
        "/v1",
    );
    let (Ok(compact), Ok(expanded)) = (compact, expanded) else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(compact_prepared) = fixture_parts_endpoint(
        b"{}",
        "/servers",
        Method::Post,
        "list_servers",
        "example",
        "compute",
        &HEADERS,
        compact,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(expanded_prepared) = fixture_parts_endpoint(
        b"{}",
        "/servers",
        Method::Post,
        "list_servers",
        "example",
        "compute",
        &HEADERS,
        expanded,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    assert_eq!(compact, expanded);
    assert!(same_fingerprint(
        compact_prepared,
        compact,
        FingerprintScope::Absent,
        expanded_prepared,
        expanded,
        FingerprintScope::Absent,
    ));
}

#[test]
fn fingerprint_builder_rejects_an_endpoint_outside_prepared_policy() {
    let Some(prepared) = fixture(b"{}", "/servers") else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(other) =
        EndpointIdentity::new(EndpointScheme::Https, "other.example.invalid", 443, "/v1").ok()
    else {
        unreachable!("retry security fixture construction failed");
    };
    let mut storage = [0xA5_u8; 512];
    let result =
        build_canonical_fingerprint(prepared, other, FingerprintScope::Absent, &mut storage);
    assert!(matches!(
        result,
        Err(FingerprintBuildError::EndpointNotAdmitted)
    ));
    drop(result);
    assert!(storage.iter().all(|byte| *byte == 0));
}
