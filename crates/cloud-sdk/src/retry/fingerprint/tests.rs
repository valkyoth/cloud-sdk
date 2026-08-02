use super::{
    DigestAlgorithm, FingerprintBuildError, FingerprintHasher, FingerprintKind, FingerprintRef,
    FingerprintScope, build_canonical_fingerprint, build_fingerprint_digest,
};
use crate::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use crate::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use crate::transport::{
    ContentType, EndpointIdentity, EndpointPolicy, EndpointScheme, MediaType, RawResponsePolicy,
    RequestHeader, RequestHeaders, RequestTarget, ResponseMediaPolicy, StatusCode,
    TransportRequest,
};
use crate::{Method, ProviderId, ServiceId};

mod sha256;
use sha256::{Sha256, sha256};
mod endpoint_policy_tests;

static OK: [StatusCode; 1] = [StatusCode::OK];
static JSON: [MediaType<'static>; 1] = [MediaType::JSON];
pub(super) static HEADERS: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
static OTHER_HEADERS: [RequestHeader<'static>; 1] =
    [RequestHeader::content_type(ContentType::JSON)];

#[test]
fn canonical_format_is_versioned_field_separated_and_cleared() {
    let Some(prepared) = fixture(b"{\"name\":\"one\"}", "/servers?name=one") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0xA5_u8; 512];
    {
        let result = build_canonical_fingerprint(
            prepared,
            endpoint,
            FingerprintScope::Value(b"account-a"),
            &mut storage,
        );
        assert!(result.is_ok());
        let canonical = match result {
            Ok(value) => value,
            Err(_) => return,
        };
        assert!(!canonical.is_empty());
        let FingerprintRef(FingerprintKind::Exact(bytes)) = canonical.as_ref() else {
            return;
        };
        assert!(bytes.starts_with(b"cloud-sdk/retry-fingerprint/v2\0"));
        assert!(contains_field(bytes, 1, b"example"));
        assert!(contains_field(bytes, 2, b"compute"));
        assert!(contains_field(bytes, 3, b"list_servers"));
        assert!(contains_field(bytes, 9, b"/servers"));
        assert!(contains_field(bytes, 11, b"name=one"));
        assert!(contains_field(bytes, 13, b"accept"));
        assert!(contains_field(bytes, 18, &[0]));
        assert!(contains_field(bytes, 15, b"{\"name\":\"one\"}"));
        assert!(contains_field(bytes, 16, &[1]));
        assert!(contains_field(bytes, 17, b"account-a"));
    }
    assert!(storage.iter().all(|byte| *byte == 0));
}

#[test]
fn every_request_identity_component_changes_exact_comparison() {
    let Some(endpoint) = endpoint() else { return };
    let Some(first_request) = fixture(b"one", "/servers?name=one") else {
        return;
    };
    let Some(second_request) = fixture(b"two", "/servers?name=one") else {
        return;
    };
    let mut first_storage = [0_u8; 512];
    let mut second_storage = [0_u8; 512];
    let first = build_canonical_fingerprint(
        first_request,
        endpoint,
        FingerprintScope::Absent,
        &mut first_storage,
    );
    let second = build_canonical_fingerprint(
        second_request,
        endpoint,
        FingerprintScope::Absent,
        &mut second_storage,
    );
    assert!(first.is_ok() && second.is_ok());
    if let (Ok(first), Ok(second)) = (first, second) {
        assert!(!first.as_ref().matches(second.as_ref()));
    }
}

#[test]
fn every_provider_operation_and_wire_domain_change_is_distinct() {
    let Some(base) = fixture_parts(
        b"one",
        "/servers?name=one",
        Method::Post,
        "list_servers",
        "example",
        "compute",
        &HEADERS,
    ) else {
        return;
    };
    let Some(base_endpoint) = endpoint() else {
        return;
    };
    let candidates = [
        fixture_parts(
            b"one",
            "/servers?name=one",
            Method::Post,
            "list_servers",
            "other",
            "compute",
            &HEADERS,
        ),
        fixture_parts(
            b"one",
            "/servers?name=one",
            Method::Post,
            "list_servers",
            "example",
            "network",
            &HEADERS,
        ),
        fixture_parts(
            b"one",
            "/servers?name=one",
            Method::Post,
            "get_server",
            "example",
            "compute",
            &HEADERS,
        ),
        fixture_parts(
            b"one",
            "/servers?name=one",
            Method::Get,
            "list_servers",
            "example",
            "compute",
            &HEADERS,
        ),
        fixture_parts(
            b"one",
            "/servers?name=two",
            Method::Post,
            "list_servers",
            "example",
            "compute",
            &HEADERS,
        ),
        fixture_parts(
            b"one",
            "/servers?name=one",
            Method::Post,
            "list_servers",
            "example",
            "compute",
            &OTHER_HEADERS,
        ),
        fixture_parts(
            b"two",
            "/servers?name=one",
            Method::Post,
            "list_servers",
            "example",
            "compute",
            &HEADERS,
        ),
    ];
    for candidate in candidates {
        let Some(candidate) = candidate else { return };
        assert!(!same_fingerprint(
            base,
            base_endpoint,
            FingerprintScope::Absent,
            candidate,
            base_endpoint,
            FingerprintScope::Absent,
        ));
    }
    let Some(other_endpoint) =
        EndpointIdentity::new(EndpointScheme::Https, "other.example.invalid", 443, "/v1").ok()
    else {
        return;
    };
    let Some(other_prepared) = fixture_parts_endpoint(
        b"one",
        "/servers?name=one",
        Method::Post,
        "list_servers",
        "example",
        "compute",
        &HEADERS,
        other_endpoint,
    ) else {
        return;
    };
    assert!(!same_fingerprint(
        base,
        base_endpoint,
        FingerprintScope::Absent,
        other_prepared,
        other_endpoint,
        FingerprintScope::Absent,
    ));
    assert!(!same_fingerprint(
        base,
        base_endpoint,
        FingerprintScope::Absent,
        base,
        base_endpoint,
        FingerprintScope::Value(b"account"),
    ));
    assert!(!same_fingerprint(
        base,
        base_endpoint,
        FingerprintScope::Absent,
        base,
        base_endpoint,
        FingerprintScope::Value(b""),
    ));
}

#[test]
fn digest_boundary_accepts_sha256_vector_and_rejects_wrong_length() {
    assert_eq!(
        sha256(b"abc"),
        Some([
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ])
    );
    let Some(prepared) = fixture(b"{}", "/servers") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut scratch = [0xA5_u8; 512];
    let mut digest_output = [0xA5_u8; 64];
    {
        let digest = build_fingerprint_digest(
            prepared,
            endpoint,
            FingerprintScope::Absent,
            &mut scratch,
            &mut digest_output,
            &Sha256,
        );
        assert!(digest.is_ok());
        let Ok(digest) = digest else { return };
        assert_eq!(digest.algorithm(), DigestAlgorithm::Sha256);
        let FingerprintRef(FingerprintKind::Digest { bytes, .. }) = digest.as_ref() else {
            return;
        };
        assert_eq!(bytes.len(), 32);
        assert!(bytes.iter().any(|byte| *byte != 0));
    }
    assert!(scratch.iter().all(|byte| *byte == 0));
    assert!(digest_output.iter().all(|byte| *byte == 0));

    digest_output.fill(0xA5);
    let error = build_fingerprint_digest(
        prepared,
        endpoint,
        FingerprintScope::Absent,
        &mut scratch,
        &mut digest_output,
        &WrongLength,
    );
    assert!(matches!(
        error,
        Err(FingerprintBuildError::InvalidDigestLength)
    ));
    drop(error);
    assert!(scratch.iter().all(|byte| *byte == 0));
    assert!(digest_output.iter().all(|byte| *byte == 0));

    let mut tiny_output = [0xA5_u8; 31];
    let error = build_fingerprint_digest(
        prepared,
        endpoint,
        FingerprintScope::Absent,
        &mut scratch,
        &mut tiny_output,
        &Sha256,
    );
    assert!(matches!(error, Err(FingerprintBuildError::OutputTooSmall)));
    drop(error);
    assert!(tiny_output.iter().all(|byte| *byte == 0));
}

#[test]
fn insufficient_storage_fails_with_cleared_output() {
    let Some(prepared) = fixture(b"{}", "/servers") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0xA5_u8; 8];
    let result =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage);
    assert!(matches!(result, Err(FingerprintBuildError::OutputTooSmall)));
    drop(result);
    assert!(storage.iter().all(|byte| *byte == 0));
}

pub(super) fn fixture(
    body: &'static [u8],
    target: &'static str,
) -> Option<PreparedRequest<'static>> {
    fixture_parts(
        body,
        target,
        Method::Post,
        "list_servers",
        "example",
        "compute",
        &HEADERS,
    )
}

fn fixture_parts(
    body: &'static [u8],
    target: &'static str,
    method: Method,
    operation: &'static str,
    provider: &'static str,
    service: &'static str,
    headers: &'static [RequestHeader<'static>],
) -> Option<PreparedRequest<'static>> {
    let endpoint = endpoint()?;
    fixture_parts_endpoint(
        body, target, method, operation, provider, service, headers, endpoint,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn fixture_parts_endpoint(
    body: &'static [u8],
    target: &'static str,
    method: Method,
    operation: &'static str,
    provider: &'static str,
    service: &'static str,
    headers: &'static [RequestHeader<'static>],
    endpoint: EndpointIdentity<'static>,
) -> Option<PreparedRequest<'static>> {
    let request = TransportRequest::new(method, RequestTarget::new(target).ok()?)
        .with_headers(RequestHeaders::new(headers).ok()?)
        .with_body(body);
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .ok()?;
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        64,
    )
    .ok()?;
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let raw = RawResponsePolicy::new(
        64,
        64,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        0,
    )
    .ok()?;
    let prepared = PreparedRequest::new(
        request,
        ProviderService::new(
            ProviderId::new(provider).ok()?,
            ServiceId::new(service).ok()?,
            EndpointPolicy::fixed(endpoint),
        ),
        metadata,
        response,
        authentication,
        raw,
    )
    .ok()?;
    let operation = OperationId::new(operation).ok()?;
    Some(prepared.with_operation_id(operation).with_replayable_body())
}

pub(super) fn same_fingerprint(
    first: PreparedRequest<'_>,
    first_endpoint: EndpointIdentity<'_>,
    first_scope: FingerprintScope<'_>,
    second: PreparedRequest<'_>,
    second_endpoint: EndpointIdentity<'_>,
    second_scope: FingerprintScope<'_>,
) -> bool {
    let mut first_storage = [0_u8; 512];
    let mut second_storage = [0_u8; 512];
    let first = build_canonical_fingerprint(first, first_endpoint, first_scope, &mut first_storage);
    let second =
        build_canonical_fingerprint(second, second_endpoint, second_scope, &mut second_storage);
    match (first, second) {
        (Ok(first), Ok(second)) => first.as_ref().matches(second.as_ref()),
        _ => true,
    }
}

fn endpoint() -> Option<EndpointIdentity<'static>> {
    EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1").ok()
}

fn contains_field(mut input: &[u8], tag: u8, expected: &[u8]) -> bool {
    input = input
        .strip_prefix(b"cloud-sdk/retry-fingerprint/v2\0")
        .unwrap_or_default();
    while input.len() >= 9 {
        let Some(current_tag) = input.first().copied() else {
            return false;
        };
        let mut length = [0_u8; 8];
        let Some(encoded_length) = input.get(1..9) else {
            return false;
        };
        length.copy_from_slice(encoded_length);
        let Ok(length) = usize::try_from(u64::from_be_bytes(length)) else {
            return false;
        };
        let Some(end) = 9_usize.checked_add(length) else {
            return false;
        };
        let Some(value) = input.get(9..end) else {
            return false;
        };
        if current_tag == tag && value == expected {
            return true;
        }
        input = input.get(end..).unwrap_or_default();
    }
    false
}

struct WrongLength;

impl FingerprintHasher for WrongLength {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, _input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0xA5);
        Ok(31)
    }
}
