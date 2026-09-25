use super::*;
use crate::{
    credentials::{ApiToken, CredentialOrigin},
    discovery::tests::Fixture as _,
    identifiers::{CrateName, Owner, UserLogin},
};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
#[cfg(feature = "blocking")]
mod execution;
fn name() -> CrateName<'static> {
    CrateName::new("example").fixture("crate")
}
fn selector(s: &str) -> OwnerSelector<'_> {
    OwnerSelector::new(s).fixture("selector")
}
fn token(origin: CredentialOrigin) -> ApiToken {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let mut bytes = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 32];
    ApiToken::from_mut_bytes(origin, &mut bytes).fixture("token")
}
#[test]
fn selectors_batches_and_cargo_wire_are_exact() {
    let owners = [
        selector("alice"),
        selector("crates.io:bob"),
        selector("github:charlie"),
        selector("github:org:team"),
    ];
    for remove in [false, true] {
        let request = if remove {
            OwnerChangeRequest::remove(name(), &owners)
        } else {
            OwnerChangeRequest::add(name(), &owners)
        }
        .fixture("request");
        let mut target = [0; 128];
        assert_eq!(
            request.write_target(&mut target).fixture("path").as_str(),
            "/api/v1/crates/example/owners"
        );
        let mut tiny = [0xa5; 2];
        assert!(request.write_target(&mut tiny).is_err());
        assert_eq!(tiny, [0xa5; 2]);
        let mut body = [0xa5; 512];
        request
            .with_json_body(&mut body, |b| {
                assert_eq!(
                    b,
                    br#"{"users":["alice","crates.io:bob","github:charlie","github:org:team"]}"#
                )
            })
            .fixture("JSON");
        assert!(body.iter().all(|b| *b == 0));
        assert!(
            request
                .with_json_body(&mut tiny, |_| unreachable!("undersized buffer accepted"))
                .is_err()
        );
        assert_eq!(tiny, [0; 2]);
        assert_eq!(request.operation().is_destructive(), remove);
        assert_eq!(
            request.operation().method(),
            if remove {
                cloud_sdk::Method::Delete
            } else {
                cloud_sdk::Method::Put
            }
        );
        assert!(!request.operation().permits_automatic_retry());
        assert_eq!(format!("{request:?}"), "OwnerChangeRequest([redacted])");
    }
    for s in [
        "",
        "crates.io:",
        "github:",
        "github:org:",
        "github::team",
        "github:a:b:c",
        "gitlab:a",
        "a\"",
        "a\n",
        "crates.io:a:b",
    ] {
        assert!(OwnerSelector::new(s).is_err());
    }
    assert!(OwnerChangeRequest::add(name(), &[]).is_err());
    assert!(OwnerChangeRequest::remove(name(), &[selector("a"); 11]).is_err());
    for pair in [
        [selector("Alice"), selector("crates.io:alice")],
        [selector("crates.io:a-b"), selector("github:A_B")],
        [selector("github:org:TEAM"), selector("github:ORG:team")],
    ] {
        assert!(OwnerChangeRequest::add(name(), &pair).is_err());
        assert!(OwnerChangeRequest::remove(name(), &pair).is_err());
    }
}
#[test]
fn batch_limit_and_permission_kind_are_checked() {
    let owners = [
        selector("a"),
        selector("b"),
        selector("c"),
        selector("d"),
        selector("e"),
        selector("f"),
        selector("g"),
        selector("h"),
        selector("i"),
        selector("j"),
    ];
    let request = OwnerChangeRequest::add(name(), &owners).fixture("ten owners");
    let mut bytes = [0; MAX_OWNER_CHANGE_BODY_BYTES];
    request
        .with_json_body(&mut bytes, |wire| {
            let value: serde_json::Value = serde_json::from_slice(wire).fixture("JSON");
            assert_eq!(
                value
                    .get("users")
                    .fixture("users")
                    .as_array()
                    .fixture("array")
                    .len(),
                10
            );
            assert!(value.get("owners").is_none());
        })
        .fixture("write");
    let token = token(CredentialOrigin::Production);
    assert!(request.confirm_removal(&token).is_err());
    assert!(
        OwnerChangeRequest::remove(name(), &owners)
            .fixture("request")
            .confirm_add(&token)
            .is_err()
    );
}
#[test]
fn preflight_denies_self_last_owner_unknown_namespaces_and_wrong_crate() {
    let token = token(CredentialOrigin::Production);
    let actor = UserLogin::new("alice").fixture("actor");
    let active = [
        Owner::new("alice").fixture("owner"),
        Owner::new("bob").fixture("owner"),
        Owner::new("github:org:team").fixture("team"),
    ];
    let snapshot = RemovalSnapshot::from_complete_active_list(name(), &active).fixture("snapshot");
    for (names, mode, ok) in [
        (&[selector("crates.io:bob")][..], SelfRemoval::Deny, true),
        (&[selector("crates.io:alice")][..], SelfRemoval::Deny, false),
        (&[selector("crates.io:alice")][..], SelfRemoval::Allow, true),
        (
            &[selector("crates.io:alice"), selector("crates.io:bob")][..],
            SelfRemoval::Allow,
            false,
        ),
        (&[selector("github:bob")][..], SelfRemoval::Deny, false),
        (&[selector("bob")][..], SelfRemoval::Deny, false),
        (
            &[selector("crates.io:unknown")][..],
            SelfRemoval::Deny,
            false,
        ),
        (&[selector("github:org:team")][..], SelfRemoval::Deny, true),
    ] {
        let request = OwnerChangeRequest::remove(name(), names).fixture("request");
        assert_eq!(
            request
                .confirm_removal_after_preflight(&token, &snapshot, actor, mode)
                .is_ok(),
            ok
        );
    }
    let names = [selector("crates.io:bob")];
    assert!(
        OwnerChangeRequest::remove(CrateName::new("other").fixture("other"), &names)
            .fixture("request")
            .confirm_removal_after_preflight(&token, &snapshot, actor, SelfRemoval::Deny)
            .is_err()
    );
    let duplicate = [
        Owner::new("alice").fixture("owner"),
        Owner::new("ALICE").fixture("owner"),
    ];
    assert!(RemovalSnapshot::from_complete_active_list(name(), &duplicate).is_err());
    assert!(RemovalSnapshot::from_complete_active_list(name(), &[]).is_err());
    assert!(
        OwnerChangeRequest::remove(name(), &names)
            .fixture("request")
            .confirm_removal_after_preflight(
                &token,
                &snapshot,
                UserLogin::new("absent").fixture("actor"),
                SelfRemoval::Allow
            )
            .is_err()
    );
}
fn decode(
    permit: &OwnerChangePermit<'_>,
    wire: &[u8],
) -> Result<OwnerChangeResponse, OwnerChangeError> {
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
        Err(_) => Err(OwnerChangeError::Json),
    };
    assert!(bytes.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    result
}
#[test]
fn acknowledgements_do_not_infer_acceptance_from_prose() {
    let token = token(CredentialOrigin::Production);
    let owners = [selector("alice")];
    for permit in [
        OwnerChangeRequest::add(name(), &owners)
            .fixture("add")
            .confirm_add(&token)
            .fixture("permit"),
        OwnerChangeRequest::remove(name(), &owners)
            .fixture("remove")
            .confirm_removal(&token)
            .fixture("permit"),
    ] {
        for message in [
            "user already has a pending invitation",
            "invited",
            "owners successfully removed",
            "unrecognized future prose",
            "",
        ] {
            let wire =
                serde_json::to_vec(&serde_json::json!({"ok":true,"msg":message})).fixture("JSON");
            let response = decode(&permit, &wire).fixture("decode");
            assert_eq!(
                response.outcome(),
                if permit.operation().is_destructive() {
                    OwnershipOutcome::RemovalAcknowledged
                } else {
                    OwnershipOutcome::AdditionAcknowledged
                }
            );
            response
                .with_message(|m| assert_eq!(m, message))
                .fixture("message");
            assert!(!format!("{response:?}").contains("pending invitation"));
        }
        for wire in [
            br#"{"ok":false,"msg":"failed"}"#.as_slice(),
            br#"{"ok":true}"#,
            br#"{"ok":true,"msg":null}"#,
            br#"{"ok":true,"ok":true,"msg":"x"}"#,
            br#"{"ok":true,"msg":"partial","errors":[{"detail":"failure"}]}"#,
        ] {
            assert!(decode(&permit, wire).is_err());
        }
        let wire = serde_json::to_vec(&serde_json::json!({"ok":true,"msg":"x".repeat(8193)}))
            .fixture("JSON");
        assert!(decode(&permit, &wire).is_err());
    }
}
