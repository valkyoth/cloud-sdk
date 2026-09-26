use alloc::{format, vec::Vec};

use super::{CloudResource, parse_cloud_resource};
use crate::serde::models::ResponseModelError;
use crate::serde::strict_json::parse;

#[test]
fn complete_model_retains_unknown_fields_and_open_enum_values() {
    let value = parse(
        br#"{"id":42,"name":"group","labels":{},"type":"future-spread","created":"2026-08-08T00:00:00Z","servers":[],"future":{"enabled":true}}"#,
    );
    let Ok(value) = value else {
        unreachable!("complete model fixture failed")
    };
    let resource = parse_cloud_resource("placement_group", &value);
    let Ok(CloudResource::PlacementGroup(resource)) = resource else {
        unreachable!("placement-group model was not selected")
    };
    assert_eq!(resource.id(), 42);
    assert_eq!(resource.fields().text("type"), Some("future-spread"));
    assert!(resource.fields().get("future").is_some());
}

#[test]
fn complete_model_rejects_missing_required_and_wrong_nullable_types() {
    let missing = parse(
        br#"{"id":42,"name":"group","labels":{},"type":"spread","created":"2026-08-08T00:00:00Z"}"#,
    );
    let Ok(missing) = missing else {
        unreachable!("missing-field fixture failed")
    };
    assert_eq!(
        parse_cloud_resource("placement_group", &missing),
        Err(ResponseModelError::MissingField)
    );

    let wrong = parse(br#"{"id":42,"name":"ip","labels":{},"created":"2026-08-08T00:00:00Z","blocked":false,"location":{},"ip":"192.0.2.1","dns_ptr":[],"protection":{"delete":false},"type":"ipv4","auto_delete":false,"assignee_type":"server","assignee_id":"42"}"#);
    let Ok(wrong) = wrong else {
        unreachable!("wrong-nullability fixture failed")
    };
    assert_eq!(
        parse_cloud_resource("primary_ip", &wrong),
        Err(ResponseModelError::WrongType)
    );
}

#[test]
fn complete_model_debug_is_redacted_and_copy_is_fallible() {
    let value = parse(
        br#"{"id":42,"name":"topology-canary","labels":{},"type":"spread","created":"2026-08-08T00:00:00Z","servers":[],"future":{"address":"198.51.100.9"}}"#,
    );
    let Ok(value) = value else {
        unreachable!("redaction fixture failed")
    };
    let resource = parse_cloud_resource("placement_group", &value);
    let Ok(resource) = resource else {
        unreachable!("redaction resource failed")
    };
    let copy = resource.try_clone();
    assert_eq!(copy.as_ref(), Ok(&resource));

    let debug = format!("{resource:?} {:?}", resource.fields());
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("topology-canary"));
    assert!(!debug.contains("198.51.100.9"));
    assert!(!debug.contains("42"));

    let Some(future) = resource.fields().get("future") else {
        unreachable!("future field was not retained")
    };
    assert!(!format!("{future:?}").contains("198.51.100.9"));
}

#[test]
fn changelog_additions_are_retained_without_weakening_known_validation() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../cloud_model_fixtures.json"))
            .unwrap_or_else(|_| unreachable!("generated fixture JSON"));

    let mut load_balancer = fixture_resource(&fixture, "load_balancer");
    let health = health_fixture_mut(&mut load_balancer);
    health.insert("status".into(), serde_json::json!("unhealthy"));
    health.insert("detail".into(), serde_json::json!("unexpected_http_status"));
    health.insert("http_status_code".into(), serde_json::json!(503));
    let load_balancer = parse_resource_fixture("load_balancer", &load_balancer);
    let CloudResource::LoadBalancer(load_balancer) = load_balancer else {
        unreachable!("load-balancer fixture kind")
    };
    let health = load_balancer
        .fields()
        .get("targets")
        .and_then(|value| value.as_array())
        .and_then(|targets| targets.first())
        .and_then(|value| value.as_object())
        .and_then(|target| target.get("health_status"))
        .and_then(|value| value.as_array())
        .and_then(|statuses| statuses.first())
        .and_then(|value| value.as_object())
        .unwrap_or_else(|| unreachable!("retained health status"));
    assert_eq!(health.text("detail"), Some("unexpected_http_status"));
    assert_eq!(health.u64("http_status_code"), Some(503));

    let mut primary_ip = fixture_resource(&fixture, "primary_ip");
    set_fixture_field(
        &mut primary_ip,
        "assignee_type",
        serde_json::json!("unassigned"),
    );
    set_fixture_field(&mut primary_ip, "assignee_id", serde_json::Value::Null);
    let primary_ip = parse_resource_fixture("primary_ip", &primary_ip);
    let CloudResource::PrimaryIp(primary_ip) = primary_ip else {
        unreachable!("primary-IP fixture kind")
    };
    assert_eq!(
        primary_ip.fields().text("assignee_type"),
        Some("unassigned")
    );
    assert!(
        primary_ip
            .fields()
            .get("assignee_id")
            .is_some_and(super::super::CloudValue::is_null)
    );

    let mut mismatch = fixture_resource(&fixture, "primary_ip");
    set_fixture_field(
        &mut mismatch,
        "assignee_type",
        serde_json::json!("unassigned"),
    );
    set_fixture_field(&mut mismatch, "assignee_id", serde_json::json!(42));
    assert!(matches!(
        try_parse_resource_fixture("primary_ip", &mismatch),
        Err(ResponseModelError::EnvelopeMismatch)
    ));

    let mut mismatch = fixture_resource(&fixture, "load_balancer");
    let health = health_fixture_mut(&mut mismatch);
    health.insert("status".into(), serde_json::json!("unhealthy"));
    health.insert("detail".into(), serde_json::json!("unexpected_http_status"));
    assert!(health.remove("http_status_code").is_some());
    assert!(matches!(
        try_parse_resource_fixture("load_balancer", &mismatch),
        Err(ResponseModelError::EnvelopeMismatch)
    ));

    let mut mismatch = fixture_resource(&fixture, "load_balancer");
    let health = health_fixture_mut(&mut mismatch);
    health.insert("status".into(), serde_json::json!("unhealthy"));
    health.insert("detail".into(), serde_json::json!("future_reason"));
    assert!(matches!(
        try_parse_resource_fixture("load_balancer", &mismatch),
        Err(ResponseModelError::UnknownEnumValue)
    ));
}

fn fixture_resource(fixture: &serde_json::Value, name: &str) -> serde_json::Value {
    fixture
        .get(name)
        .cloned()
        .unwrap_or_else(|| unreachable!("generated resource fixture"))
}

fn health_fixture_mut(
    fixture: &mut serde_json::Value,
) -> &mut serde_json::Map<alloc::string::String, serde_json::Value> {
    fixture
        .get_mut("targets")
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|targets| targets.first_mut())
        .and_then(|target| target.get_mut("health_status"))
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|statuses| statuses.first_mut())
        .and_then(serde_json::Value::as_object_mut)
        .unwrap_or_else(|| unreachable!("health fixture object"))
}

fn set_fixture_field(fixture: &mut serde_json::Value, name: &str, value: serde_json::Value) {
    fixture
        .as_object_mut()
        .unwrap_or_else(|| unreachable!("resource fixture object"))
        .insert(name.into(), value);
}

fn parse_resource_fixture(model: &str, fixture: &serde_json::Value) -> CloudResource {
    try_parse_resource_fixture(model, fixture)
        .unwrap_or_else(|_| unreachable!("source-complete fixture parsing"))
}

fn try_parse_resource_fixture(
    model: &str,
    fixture: &serde_json::Value,
) -> Result<CloudResource, ResponseModelError> {
    let bytes: Vec<u8> =
        serde_json::to_vec(fixture).unwrap_or_else(|_| unreachable!("fixture serialization"));
    let value = parse(&bytes).unwrap_or_else(|_| unreachable!("strict fixture parsing"));
    parse_cloud_resource(model, &value)
}
