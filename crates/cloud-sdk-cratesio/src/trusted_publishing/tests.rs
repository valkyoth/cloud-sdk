use super::*;
use crate::{
    credentials::{ApiToken, CredentialOrigin, OidcAssertion},
    discovery::tests::Fixture as _,
    identifiers::{CrateName, NumericId},
    query::{Parameter, Query},
};
use alloc::{format, string::String, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
use serde_json::{Value, json};
mod assertions;
mod configurations;
#[cfg(feature = "blocking")]
mod execution;
mod tokens;
#[cfg(feature = "blocking")]
mod workflow;
fn config(p: Publisher) -> PublisherConfig<'static> {
    PublisherConfig::new(
        p,
        CrateName::new("regex").fixture("crate"),
        "rust-lang",
        "regex",
        if p == Publisher::GitHub {
            "ci.yml"
        } else {
            ".gitlab-ci.yml"
        },
        None,
    )
    .fixture("config")
}
fn id(n: u64) -> NumericId {
    NumericId::new(n).fixture("id")
}
fn api(origin: CredentialOrigin) -> ApiToken {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let mut bytes = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 32];
    ApiToken::from_mut_bytes(origin, &mut bytes).fixture("api token")
}
fn fixture(name: &str) -> Value {
    let mut all: Value = serde_json::from_str(include_str!("fixtures.json")).fixture("fixtures");
    all.as_object_mut()
        .fixture("object")
        .remove(name)
        .fixture("named fixture")
}
fn decode(
    permit: TrustedPublishingPermit<'_>,
    wire: &Value,
    now: u64,
) -> Result<TrustedPublishingResponse, TrustedPublishingError> {
    let wire = serde_json::to_vec(wire).fixture("serialize");
    let mut bytes = vec![0xa5; wire.len()];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(&wire);
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
    let success = crate::wire::JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(now))
        .fixture("wire");
    let result = permit.decode_response(success, now);
    assert!(bytes.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    result
}
fn encode(bytes: &[u8]) -> String {
    let mut output = vec![0; bytes.len().saturating_mul(2).saturating_add(8)];
    let n = base64_ng::URL_SAFE_NO_PAD
        .encode_slice(bytes, &mut output)
        .fixture("base64");
    String::from_utf8(output.get(..n).fixture("range").to_vec()).fixture("utf8")
}
fn jwt(header: &Value, claims: &Value) -> String {
    format!(
        "{}.{}.{}",
        encode(&serde_json::to_vec(header).fixture("header")),
        encode(&serde_json::to_vec(claims).fixture("claims")),
        encode(b"not-a-signature")
    )
}
fn header() -> Value {
    json!({"alg":"RS256","kid":"test-key-id"})
}
fn claims(p: Publisher, now: u64) -> Value {
    let mut c = json!({"iss":p.issuer(), "aud":"crates.io", "iat":now, "exp":now.saturating_add(60), "jti":"synthetic-run", "sha":"public-commit-id"});
    let fields = match p {
        Publisher::GitHub => {
            json!({"repository_owner_id":"123", "repository":"rust-lang/regex", "workflow_ref":"rust-lang/regex/.github/workflows/ci.yml@refs/heads/main", "event_name":"push", "run_id":"42"})
        }
        Publisher::GitLab => {
            json!({"namespace_id":"123", "project_path":"rust-lang/regex", "ci_config_ref_uri":"gitlab.com/rust-lang/regex//.gitlab-ci.yml@refs/heads/main", "job_id":"42"})
        }
    };
    c.as_object_mut()
        .fixture("object")
        .extend(fields.as_object().fixture("object").clone());
    c
}
fn assertion(p: Publisher, now: u64, origin: CredentialOrigin) -> OidcAssertion {
    let mut bytes = jwt(&header(), &claims(p, now)).into_bytes();
    let token = OidcAssertion::from_mut_bytes(origin, &mut bytes).fixture("assertion");
    assert!(bytes.iter().all(|b| *b == 0));
    token
}
fn policy(p: Publisher, now: u64) -> ExchangePolicy<'static> {
    ExchangePolicy::new(config(p), "crates.io", now, 1800).fixture("policy")
}
fn token_response() -> Value {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let raw = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 31];
    let xor = raw.iter().fold(0u8, |a, b| a ^ b);
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let check = *alphabet.get(usize::from(xor) % 62).fixture("checksum");
    let mut text = String::from("cio_tp_");
    text.push_str(core::str::from_utf8(&raw).fixture("ascii"));
    text.push(char::from(check));
    json!({"token":text})
}
fn temporary(now: u64) -> TemporaryToken {
    let p = Publisher::GitHub;
    let permit = TrustedPublishingPermit::confirm_exchange(
        assertion(p, now, CredentialOrigin::Production),
        policy(p, now),
    );
    let TrustedPublishingResponse::Exchanged(token) =
        decode(permit, &token_response(), now).fixture("exchange")
    else {
        unreachable!("wrong response")
    };
    token
}
