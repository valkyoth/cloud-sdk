use super::{CloudResource, ResponseModelError, parse_cloud_resource};
use crate::serde::strict_json::parse;
use serde_json::{Value, json};

fn fixture(model: &str) -> Value {
    let all: Value = serde_json::from_str(include_str!("../cloud_model_fixtures.json"))
        .unwrap_or_else(|_| unreachable!("source fixtures must parse"));
    all.get(model)
        .unwrap_or_else(|| unreachable!("source model must exist"))
        .clone()
}

fn decode(model: &str, value: &Value) -> Result<CloudResource, ResponseModelError> {
    let bytes = serde_json::to_vec(value).unwrap_or_else(|_| unreachable!("fixture serialization"));
    parse_cloud_resource(
        model,
        &parse(&bytes).unwrap_or_else(|_| unreachable!("fixture JSON")),
    )
}

#[test]
fn image_deprecation_is_required_nullable_and_calendar_checked() {
    for model in ["image", "server"] {
        let mut value = fixture(model);
        let image = if model == "server" {
            value
                .get_mut("image")
                .unwrap_or_else(|| unreachable!("image"))
        } else {
            &mut value
        };
        image
            .as_object_mut()
            .unwrap_or_else(|| unreachable!("image object"))
            .insert("deprecation".into(), Value::Null);
        assert!(decode(model, &value).is_ok());
        let image = if model == "server" {
            value
                .get_mut("image")
                .unwrap_or_else(|| unreachable!("image"))
        } else {
            &mut value
        };
        image
            .as_object_mut()
            .unwrap_or_else(|| unreachable!("image object"))
            .insert(
                "deprecation".into(),
                json!({
                    "announced": "2026-09-01T00:00:00Z",
                    "unavailable_after": "2027-01-01T00:00:00Z"
                }),
            );
        let decoded = decode(model, &value).unwrap_or_else(|_| unreachable!("valid deprecation"));
        let fields = if model == "server" {
            decoded
                .fields()
                .get("image")
                .and_then(|v| v.as_object())
                .unwrap_or_else(|| unreachable!("image"))
        } else {
            decoded.fields()
        };
        assert_eq!(
            fields
                .get("deprecation")
                .and_then(|v| v.as_object())
                .and_then(|v| v.text("announced")),
            Some("2026-09-01T00:00:00Z")
        );
        for malformed in [
            json!({}),
            json!(false),
            json!({
                "announced": "2026-02-30T00:00:00Z", "unavailable_after": "2027-01-01T00:00:00Z"
            }),
        ] {
            let image = if model == "server" {
                value
                    .get_mut("image")
                    .unwrap_or_else(|| unreachable!("image"))
            } else {
                &mut value
            };
            image
                .as_object_mut()
                .unwrap_or_else(|| unreachable!("image object"))
                .insert("deprecation".into(), malformed);
            assert!(decode(model, &value).is_err());
        }
        let image = if model == "server" {
            value
                .get_mut("image")
                .unwrap_or_else(|| unreachable!("image"))
        } else {
            &mut value
        };
        image
            .as_object_mut()
            .unwrap_or_else(|| unreachable!("image object"))
            .remove("deprecation");
        assert!(matches!(
            decode(model, &value),
            Err(ResponseModelError::MissingField)
        ));
    }
}

#[test]
fn ip_names_follow_new_source_length_bounds() {
    for model in ["floating_ip", "primary_ip"] {
        let mut value = fixture(model);
        for length in [1, 255] {
            value
                .as_object_mut()
                .unwrap_or_else(|| unreachable!("object"))
                .insert("name".into(), json!("n".repeat(length)));
            assert!(decode(model, &value).is_ok());
        }
        for length in [0, 256] {
            value
                .as_object_mut()
                .unwrap_or_else(|| unreachable!("object"))
                .insert("name".into(), json!("n".repeat(length)));
            assert!(decode(model, &value).is_err());
        }
    }
}

#[test]
fn every_target_health_branch_checks_new_fields() {
    for kind in ["server", "ip", "label_selector"] {
        let mut value = fixture("load_balancer");
        let health = json!({"listen_port":443,"status":"unhealthy",
            "detail":"unexpected_http_status","http_status_code":503});
        let server = json!({"type":"server","server":{"id":42,"ip":"203.0.113.1"},
            "use_private_ip":false,"health_status":[health.clone()]});
        value
            .as_object_mut()
            .unwrap_or_else(|| unreachable!("object"))
            .insert(
                "targets".into(),
                json!([match kind {
                    "server" => server,
                    "ip" => json!({"type":"ip","ip":{"ip":"203.0.113.1"},"health_status":[health]}),
                    _ => json!({"type":"label_selector","label_selector":{"selector":"env=prod"},
                "use_private_ip":false,"targets":[server]}),
                }]),
            );
        assert!(decode("load_balancer", &value).is_ok());
        let target = value
            .pointer_mut("/targets/0")
            .unwrap_or_else(|| unreachable!("target"));
        let target = if kind == "label_selector" {
            target
                .pointer_mut("/targets/0")
                .unwrap_or_else(|| unreachable!("nested target"))
        } else {
            target
        };
        *target
            .pointer_mut("/health_status/0/http_status_code")
            .unwrap_or_else(|| unreachable!("health code")) = json!("503");
        assert!(decode("load_balancer", &value).is_err());
    }
}

#[test]
fn announced_legacy_deprecated_removal_does_not_relax_present_types() {
    for (model, pointer, required) in [
        ("image", "", "deprecation"),
        ("server_type", "", "locations"),
        ("load_balancer_type", "", "deprecation"),
        ("server", "/image", "deprecation"),
        ("server", "/server_type", "locations"),
        ("load_balancer", "/load_balancer_type", "deprecation"),
    ] {
        let mut value = fixture(model);
        let object = value
            .pointer_mut(pointer)
            .and_then(Value::as_object_mut)
            .unwrap_or_else(|| unreachable!("legacy owner"));
        assert!(object.remove("deprecated").is_some());
        let result = decode(model, &value);
        assert!(result.is_ok(), "{model} {pointer}: {:?}", result.err());
        let mut missing = value.clone();
        assert!(
            missing
                .pointer_mut(pointer)
                .and_then(Value::as_object_mut)
                .unwrap_or_else(|| unreachable!("required owner"))
                .remove(required)
                .is_some()
        );
        assert!(decode(model, &missing).is_err());
        value
            .pointer_mut(pointer)
            .and_then(Value::as_object_mut)
            .unwrap_or_else(|| unreachable!("legacy owner"))
            .insert("deprecated".into(), json!({}));
        assert!(decode(model, &value).is_err());
    }
}
