use super::*;
use crate::{discovery::tests::Fixture as _, wire::JsonResponsePolicy};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
use serde_json::{Value, json};

fn decode(wire: &[u8]) -> Result<CargoOwners, Error> {
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
        .fixture("media");
    attempt
        .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = match JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(0))
    {
        Ok(success) => CargoOwners::decode(success),
        Err(_) => Err(Error::Schema),
    };
    assert!(bytes.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    result
}
fn value(value: Value) -> Result<CargoOwners, Error> {
    decode(&serde_json::to_vec(&value).fixture("json"))
}

#[test]
fn minimal_cargo_profile_preserves_optional_names_and_unsigned_ids() {
    let list = decode(br#"{"users":[{"id":70,"login":"github:rust-lang:core","name":"Core"},{"id":0,"login":"zero"},{"id":4294967295,"login":"max","name":null}]}"#).fixture("owners");
    assert_eq!(list.items().len(), 3);
    let first = list.items().first().fixture("first");
    assert_eq!(first.id(), 70);
    assert!(
        first
            .with_login(|s| s == "github:rust-lang:core")
            .fixture("login")
    );
    assert!(
        first
            .fields()
            .get("name")
            .fixture("name")
            .fixture("present")
            .with_text(|s| s == "Core")
            .fixture("text")
    );
    assert_eq!(list.items().get(1).fixture("zero").id(), 0);
    assert!(
        list.items()
            .get(1)
            .fixture("zero")
            .fields()
            .get("name")
            .fixture("name")
            .is_none()
    );
    let last = list.items().last().fixture("max");
    assert_eq!(last.id(), u32::MAX);
    assert!(
        last.fields()
            .get("name")
            .fixture("name")
            .fixture("null")
            .is_null()
    );
    assert!(!format!("{list:?}").contains("rust-lang"));
    assert!(
        value(json!({"users":[]}))
            .fixture("empty")
            .items()
            .is_empty()
    );
    // IDs overlap between concrete service user/team namespaces; no kind is guessed.
    value(json!({"users":[{"id":1,"login":"a"},{"id":1,"login":"github:a:b"}]}))
        .fixture("namespaces");
}

#[test]
fn malformed_cargo_owner_fields_and_duplicate_logins_fail_closed() {
    for id in [
        json!(-1),
        json!(4294967296_u64),
        json!(1.0),
        json!("1"),
        Value::Null,
    ] {
        assert!(value(json!({"users":[{"id":id,"login":"a"}]})).is_err());
    }
    for login in [
        json!(""),
        json!("a\nb"),
        json!("a\u{7f}"),
        json!("a\u{85}"),
        json!("a".repeat(257)),
        json!(1),
        Value::Null,
    ] {
        assert!(value(json!({"users":[{"id":1,"login":login}]})).is_err());
    }
    for name in [json!("a\nb"), json!("a".repeat(257)), json!(1), json!({})] {
        assert!(value(json!({"users":[{"id":1,"login":"a","name":name}]})).is_err());
    }
    for wire in [
        br#"{}"#.as_slice(),
        br#"{"users":null}"#,
        br#"{"users":[{}]}"#,
        br#"{"users":[{"id":1}]}"#,
        br#"{"users":[{"login":"a"}]}"#,
        br#"{"users":[{"id":1,"id":2,"login":"a"}]}"#,
        br#"{"users":[],"users":[]}"#,
        br#"{"users":[{"id":1,"login":"a"},{"id":2,"login":"a"}]}"#,
        br#"{"errors":[{"detail":"no"}],"users":[]}"#,
    ] {
        assert!(decode(wire).is_err());
    }
}

#[test]
fn cargo_owner_limits_are_exact_and_extensions_are_inert() {
    let mut users = vec![];
    for n in 0..MAX_OWNERS {
        users.push(json!({"id":n,"login":format!("u{n}")}));
    }
    assert_eq!(
        value(json!({"users":users}))
            .fixture("maximum")
            .items()
            .len(),
        MAX_OWNERS
    );
    users.push(json!({"id":999,"login":"extra"}));
    assert!(matches!(value(json!({"users":users})), Err(Error::Limit)));
    let text = "x".repeat(256);
    let list = value(json!({"users":[{"id":1,"login":text,"name":text,"url":"https://untrusted.invalid/","kind":"future"}]})).fixture("bounded extensions");
    assert!(
        list.items()
            .first()
            .fixture("owner")
            .fields()
            .get("url")
            .fixture("url")
            .is_some()
    );
}
