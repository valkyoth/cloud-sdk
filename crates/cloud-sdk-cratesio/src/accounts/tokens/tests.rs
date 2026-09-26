use super::*;
use crate::{
    credentials::{ApiToken, CredentialOrigin},
    discovery::{DiscoveryError as Error, tests::Fixture as _},
    identifiers::NumericId,
};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
#[cfg(feature = "blocking")]
mod execution;
#[cfg(all(feature = "blocking", feature = "async"))]
mod unified;
fn id(n: u64) -> NumericId {
    NumericId::new(n).fixture("id")
}
fn token(origin: CredentialOrigin) -> ApiToken {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let mut bytes = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 32];
    ApiToken::from_mut_bytes(origin, &mut bytes).fixture("credential")
}
const RECORD: &str = r#"{"api_token":{"id":42,"name":"private-name","created_at":"2026-01-01T12:00:00Z","last_used_at":null,"expired_at":"2027-01-01T12:00:00Z","crate_scopes":["private-*"],"endpoint_scopes":["publish-update","yank"]}}"#;
fn decode(permit: &TokenPermit<'_>, wire: &[u8]) -> Result<TokenResponse, Error> {
    let mut bytes = vec![0xa5; wire.len()];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(wire);
    attempt
        .headers_mut()
        .fixture("headers")
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .fixture("type");
    attempt
        .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = match crate::wire::JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(0))
    {
        Ok(success) => permit.decode_response(success),
        Err(_) => Err(Error::Json),
    };
    assert!(bytes.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
    result
}
#[test]
fn routes_methods_and_atomic_encoding() {
    let token = token(CredentialOrigin::Production);
    for (permit, path, method) in [
        (
            TokenPermit::inspect(id(42), &token),
            "/api/v1/me/tokens/42",
            cloud_sdk::Method::Get,
        ),
        (
            TokenPermit::confirm_revoke(id(42), &token),
            "/api/v1/me/tokens/42",
            cloud_sdk::Method::Delete,
        ),
        (
            TokenPermit::confirm_revoke_current(&token),
            "/api/v1/tokens/current",
            cloud_sdk::Method::Delete,
        ),
    ] {
        let mut output = [0; 128];
        assert_eq!(
            permit
                .write_target(&mut output)
                .fixture("path")
                .as_request_target()
                .path()
                .as_str(),
            path
        );
        assert_eq!(permit.operation().method(), method);
        assert_eq!(
            permit.operation().is_destructive(),
            method == cloud_sdk::Method::Delete
        );
        let mut tiny = [0xa5; 3];
        assert!(permit.write_target(&mut tiny).is_err());
        assert_eq!(tiny, [0xa5; 3]);
        assert!(!format!("{permit:?}").contains("42"));
    }
}
#[test]
fn metadata_is_bound_protected_and_scopes_are_explicit() {
    let token = token(CredentialOrigin::Production);
    let permit = TokenPermit::inspect(id(42), &token);
    let TokenResponse::Metadata(value) = decode(&permit, RECORD.as_bytes()).fixture("record")
    else {
        unreachable!("wrong variant")
    };
    assert_eq!(value.id(), id(42));
    value
        .with_name(|name| assert_eq!(name, "private-name"))
        .fixture("name");
    assert_eq!(
        value.endpoint_scopes(),
        Some([EndpointScope::PublishUpdate, EndpointScope::Yank].as_slice())
    );
    assert_eq!(
        value.crate_scopes().fixture("scopes").fixture("some").len(),
        1
    );
    value
        .with_expiry(|expiry| assert_eq!(expiry, Some("2027-01-01T12:00:00Z")))
        .fixture("expiry");
    assert_eq!(format!("{value:?}"), "TokenMetadata([redacted])");
    for empty in ["null", "[]"] {
        let wire = RECORD
            .replace("[\"private-*\"]", empty)
            .replace("[\"publish-update\",\"yank\"]", empty)
            .replace("\"2027-01-01T12:00:00Z\"", "null");
        let TokenResponse::Metadata(value) = decode(&permit, wire.as_bytes()).fixture("record")
        else {
            unreachable!("variant")
        };
        assert_eq!(
            value.crate_scopes().fixture("scopes").is_none(),
            empty == "null"
        );
        assert_eq!(value.endpoint_scopes().is_none(), empty == "null");
        value
            .with_expiry(|v| assert!(v.is_none()))
            .fixture("expiry");
    }
}
#[test]
fn malformed_identity_scope_timestamp_and_acknowledgements_fail() {
    let token = token(CredentialOrigin::Production);
    let permit = TokenPermit::inspect(id(42), &token);
    for wire in [
        RECORD.replace("42", "43"),
        RECORD.replace("42", "0"),
        RECORD.replace("\"crate_scopes\":[\"private-*\"],", ""),
        RECORD.replace("publish-update", "unreviewed"),
        RECORD.replace("2027-01-01", "2027-02-30"),
        RECORD.replace("2026-01-01T12:00:00Z", "bad"),
        RECORD.replace("null", "true"),
        RECORD.replace("\"id\":42", "\"id\":42,\"id\":42"),
        RECORD.replace("private-name", &"a".repeat(1025)),
        RECORD.replace("private-*", &"a".repeat(257)),
        RECORD.replace(
            "[\"private-*\"]",
            &format!("[{}]", vec!["\"a\""; MAX_TOKEN_SCOPES + 1].join(",")),
        ),
    ] {
        assert!(decode(&permit, wire.as_bytes()).is_err());
    }
    let revoke = TokenPermit::confirm_revoke(id(42), &token);
    assert!(matches!(decode(&revoke, b"{}"), Ok(TokenResponse::Revoked)));
    for wire in [b"[]".as_slice(), b"{\"ok\":false}", b"{\"errors\":[]}"] {
        assert!(decode(&revoke, wire).is_err());
    }
    assert!(decode(&TokenPermit::confirm_revoke_current(&token), b"{}").is_err());
}
