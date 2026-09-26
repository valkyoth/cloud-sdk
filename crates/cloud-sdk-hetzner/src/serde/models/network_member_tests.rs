use super::{CloudResource, CloudResourceKind, ResponseModelError, parse_cloud_resource};
use crate::serde::strict_json::parse;
use serde_json::{Value, json};

fn fixture() -> Value {
    json!({"id":42,"type":"server","ip":"10.0.1.2","alias_ips":["10.0.1.3"],
        "subnet":"10.0.1.0/24","status":"ok"})
}

fn decode(value: &Value) -> Result<CloudResource, ResponseModelError> {
    let bytes = serde_json::to_vec(value).unwrap_or_else(|_| unreachable!("fixture"));
    let value = parse(&bytes).unwrap_or_else(|_| unreachable!("fixture"));
    parse_cloud_resource("members", &value)
}

fn set(value: &mut Value, key: &str, replacement: Value) {
    value
        .as_object_mut()
        .unwrap_or_else(|| unreachable!("object"))
        .insert(key.into(), replacement);
}

#[test]
fn network_members_preserve_resource_kind_status_and_complete_fields() {
    for kind in ["server", "load_balancer", "future-kind"] {
        for status in [
            "ok",
            "attaching",
            "detaching",
            "updating",
            "error",
            "future-status",
        ] {
            let mut value = fixture();
            set(&mut value, "type", json!(kind));
            set(&mut value, "status", json!(status));
            let member = decode(&value).unwrap_or_else(|_| unreachable!("valid member"));
            assert_eq!(member.kind(), CloudResourceKind::NetworkMember);
            assert_eq!(member.id(), 42);
            assert_eq!(member.fields().text("type"), Some(kind));
            assert_eq!(member.fields().text("status"), Some(status));
            assert_eq!(member.fields().text("subnet"), Some("10.0.1.0/24"));
            assert_eq!(member.try_clone().as_ref(), Ok(&member));
            assert!(!alloc::format!("{member:?}").contains("10.0.1"));
        }
    }
}

#[test]
fn network_members_reject_missing_fields_invalid_ids_and_ipv4() {
    for key in ["id", "type", "ip", "alias_ips", "subnet", "status"] {
        let mut value = fixture();
        value
            .as_object_mut()
            .unwrap_or_else(|| unreachable!("object"))
            .remove(key);
        assert_eq!(decode(&value), Err(ResponseModelError::MissingField));
    }
    for id in [
        json!(0),
        json!(-1),
        json!(9007199254740992_u64),
        json!("42"),
    ] {
        let mut value = fixture();
        set(&mut value, "id", id);
        assert!(decode(&value).is_err());
    }
    for ip in [
        "::1",
        "10.0.1.256",
        "010.0.1.2",
        "10.0.1",
        "10.0.1.2/24",
        "host",
    ] {
        for key in ["ip", "alias_ips"] {
            let mut value = fixture();
            set(
                &mut value,
                key,
                if key == "ip" { json!(ip) } else { json!([ip]) },
            );
            assert_eq!(decode(&value), Err(ResponseModelError::InvalidText));
        }
    }
}
