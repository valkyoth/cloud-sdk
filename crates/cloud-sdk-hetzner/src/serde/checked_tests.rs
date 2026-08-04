use alloc::format;
use alloc::string::String;

use cloud_sdk::transport::StatusCode;

use super::checked_test_support::{
    decode_response, decode_response_with_headers, decode_response_with_headers_at, empty_response,
    prepared, response,
};
use super::{HetznerDecodeError, HetznerSuccess, ResourceKind};
use crate::identity::{CLOUD_SERVICE_ID, STORAGE_SERVICE_ID};

fn action() -> &'static str {
    r#"{"id":42,"command":"poweron_server","status":"running","progress":10,"started":"2026-07-16T00:00:00Z","finished":null,"resources":[{"id":7,"type":"server"}],"error":null}"#
}

fn pagination() -> &'static str {
    r#"{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}"#
}

#[test]
fn decodes_action_list_resource_and_paginated_resource_families() {
    let single = format!(r#"{{"action":{}}}"#, action());
    let decoded = decode_response(
        prepared("get_action", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, single.as_bytes()),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::Action(_))
    ));

    let actions = format!(r#"{{"actions":[{}]}}"#, action());
    let decoded = decode_response(
        prepared("get_actions", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, actions.as_bytes()),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::Actions {
            pagination: None,
            ..
        })
    ));

    let paged_actions = format!(r#"{{"actions":[{}],"meta":{}}}"#, action(), pagination());
    assert!(
        decode_response(
            prepared("list_servers_actions", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, paged_actions.as_bytes()),
        )
        .is_ok()
    );

    let server = br#"{"server":{"id":42,"name":"web-1","status":"running","future":true}}"#;
    let decoded = decode_response(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, server),
    );
    let Ok(decoded) = decoded else {
        unreachable!("security fixture construction failed")
    };
    let HetznerSuccess::Resource(resource) = decoded.success() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(resource.kind(), ResourceKind::Server);
    assert_eq!(resource.name(), Some("web-1"));

    let servers = format!(
        r#"{{"servers":[{{"id":42,"name":"web-1","status":"running"}}],"meta":{}}}"#,
        pagination()
    );
    let decoded = decode_response(
        prepared("list_servers", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, servers.as_bytes()),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::Resources {
            pagination: Some(_),
            ..
        })
    ));
}

#[test]
fn decodes_composite_special_empty_and_storage_families() {
    let create = format!(
        r#"{{"server":{{"id":42,"name":"web-1","status":"running"}},"action":{},"next_actions":[],"root_password":"dont-log-this"}}"#,
        action()
    );
    let decoded = decode_response(
        prepared("create_server", CLOUD_SERVICE_ID, StatusCode::CREATED),
        response(StatusCode::CREATED, create.as_bytes()),
    );
    let Ok(decoded) = decoded else {
        unreachable!("security fixture construction failed")
    };
    let HetznerSuccess::Composite(composite) = decoded.success() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(composite.secrets().len(), 1);
    let Some(secret) = composite.secrets().first() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        secret
            .value()
            .try_with_secret(|value| value == "dont-log-this"),
        Ok(true)
    );
    assert!(!format!("{composite:?}").contains("dont-log-this"));

    let metrics = br#"{"metrics":{"start":"2026-01-01T00:00:00Z","end":"2026-01-01T01:00:00Z","step":60.0,"time_series":{"cpu":{"values":[[1.5,"42"]]}}}}"#;
    assert!(
        decode_response(
            prepared("get_server_metrics", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, metrics),
        )
        .is_ok()
    );
    let zonefile = br#"{"zonefile":"example.com. 60 IN A 192.0.2.1"}"#;
    let decoded = decode_response(
        prepared("get_zone_zonefile", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, zonefile),
    );
    let Ok(decoded) = decoded else {
        unreachable!("security fixture construction failed")
    };
    let HetznerSuccess::ZoneFile(zonefile) = decoded.success() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        zonefile.try_with_zonefile(|value| value == "example.com. 60 IN A 192.0.2.1"),
        Ok(true)
    );
    let pricing = br#"{"pricing":{"currency":"EUR","vat_rate":"19.0","primary_ips":[],"floating_ips":[],"image":{},"volume":{},"server_backup":{},"server_types":[],"load_balancer_types":[],"floating_ip":{}}}"#;
    assert!(
        decode_response(
            prepared("get_pricing", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, pricing),
        )
        .is_ok()
    );
    let folders = br#"{"folders":["/backup"]}"#;
    assert!(
        decode_response(
            prepared(
                "list_storage_box_folders",
                STORAGE_SERVICE_ID,
                StatusCode::OK,
            ),
            response(StatusCode::OK, folders),
        )
        .is_ok()
    );
    let empty = empty_response(StatusCode::NO_CONTENT);
    assert!(
        decode_response(
            prepared(
                "delete_certificate",
                CLOUD_SERVICE_ID,
                StatusCode::NO_CONTENT,
            ),
            empty,
        )
        .is_ok()
    );
}

#[test]
fn rejects_policy_binding_json_and_model_failures() {
    let duplicate = br#"{"server":{"id":1,"id":2,"status":"running"}}"#;
    assert_eq!(
        decode_response(
            prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, duplicate),
        ),
        Err(HetznerDecodeError::MalformedPayload)
    );
    let unknown = br#"{"server":{"id":1,"status":"future"}}"#;
    assert!(matches!(
        decode_response(
            prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, unknown),
        ),
        Err(HetznerDecodeError::Model(_))
    ));
    assert_eq!(
        decode_response(
            prepared("get_server", STORAGE_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, br#"{"server":{"id":1}}"#),
        ),
        Err(HetznerDecodeError::ServiceMismatch)
    );
    assert!(matches!(
        decode_response(
            prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::CREATED, br#"{"server":{"id":1}}"#),
        ),
        Err(HetznerDecodeError::ResponsePolicy(_))
    ));
}

#[test]
fn returns_typed_redacted_provider_errors() {
    let body = br#"{"error":{"code":"rate_limit_exceeded","message":"slow down"}}"#;
    let decoded = decode_response(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::TOO_MANY_REQUESTS, body),
    );
    let error = match &decoded {
        Err(HetznerDecodeError::Provider(error)) => Some(error),
        _ => None,
    };
    assert_eq!(
        error.map(|error| error.try_with_message(|message| message == "slow down")),
        Some(Ok(true))
    );
    assert!(
        error
            .map(|error| format!("{error:?}"))
            .is_some_and(|debug| !debug.contains("slow down"))
    );
}

#[test]
fn checked_success_and_error_retain_provider_owned_quota() {
    let headers = [
        ("ratelimit-limit", b"3600".as_slice()),
        ("ratelimit-remaining", b"0".as_slice()),
        ("ratelimit-reset", b"42".as_slice()),
        ("retry-after", b"10".as_slice()),
    ];
    let success = decode_response_with_headers(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, br#"{"server":{"id":1}}"#),
        &headers,
    );
    let Ok(success) = success else {
        unreachable!("security fixture construction failed")
    };
    assert_eq!(success.quota().buckets().len(), 1);
    assert_eq!(success.rate_limit().map(|value| value.remaining()), Some(0));
    assert!(success.quota().retry_after().is_some());

    let provider = decode_response_with_headers(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(
            StatusCode::TOO_MANY_REQUESTS,
            br#"{"error":{"code":"rate_limit_exceeded","message":"wait"}}"#,
        ),
        &headers,
    );
    let Err(HetznerDecodeError::Provider(provider)) = provider else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(provider.quota().buckets().len(), 1);
    assert!(provider.quota().retry_after().is_some());
}

#[test]
fn checked_decoder_rejects_partial_quota_before_payload_use() {
    let decoded = decode_response_with_headers(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, br#"{"server":{"id":1}}"#),
        &[("ratelimit-limit", b"3600")],
    );
    assert!(matches!(decoded, Err(HetznerDecodeError::Quota(_))));
}

#[test]
fn checked_clock_aware_decoder_resolves_obsolete_retry_date() {
    let headers = [("retry-after", b"Sunday, 06-Nov-94 08:49:37 GMT".as_slice())];
    let without_clock = decode_response_with_headers(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, br#"{"server":{"id":1}}"#),
        &headers,
    );
    assert!(matches!(without_clock, Err(HetznerDecodeError::Quota(_))));

    let with_clock = decode_response_with_headers_at(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, br#"{"server":{"id":1}}"#),
        &headers,
        cloud_sdk::rate_limit::WallClockTimestamp::new(1_767_225_600),
    );
    assert!(with_clock.is_ok());
}

#[test]
fn every_source_locked_operation_decodes_its_minimal_success_envelope() {
    let table = include_str!("response_operations.tsv");
    let mut checked = 0_usize;
    for line in table.lines().skip(1) {
        let mut fields = line.split('\t');
        let (
            Some(api),
            Some(operation),
            Some(status_text),
            Some(shape),
            Some(root),
            Some(required),
        ) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        )
        else {
            continue;
        };
        assert!(fields.next().is_none(), "invalid response binding row");
        let service_id = if api == "hetzner" {
            STORAGE_SERVICE_ID
        } else {
            CLOUD_SERVICE_ID
        };
        let status = if status_text == "201" {
            StatusCode::CREATED
        } else if status_text == "204" {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::OK
        };
        let body = minimal_body(shape, root, required);
        let response = if status == StatusCode::NO_CONTENT {
            empty_response(status)
        } else {
            response(status, body.as_bytes())
        };
        let decoded = decode_response(prepared(operation, service_id, status), response);
        assert!(decoded.is_ok(), "failed {operation}: {decoded:?}");
        checked = checked.saturating_add(1);
    }
    assert_eq!(checked, 208);
}

fn minimal_body(shape: &str, root: &str, required_fields: &str) -> String {
    let mut envelope = serde_json::Map::new();
    match shape {
        "action" => {
            envelope.insert(String::from("action"), action_value());
        }
        "actions" | "actions-page" => {
            envelope.insert(
                String::from("actions"),
                serde_json::Value::Array(alloc::vec![action_value()]),
            );
        }
        "resource" | "resource-list" | "resource-page" => {
            envelope.insert(
                String::from(root),
                if shape == "resource" {
                    resource_value(root)
                } else {
                    serde_json::Value::Array(alloc::vec![resource_value(root)])
                },
            );
        }
        "metrics" => {
            envelope.insert(
                String::from("metrics"),
                serde_json::json!({
                    "start":"2026-01-01T00:00:00Z",
                    "end":"2026-01-01T01:00:00Z",
                    "step":60.0,
                    "time_series":{}
                }),
            );
        }
        "zonefile" => {
            envelope.insert(
                String::from("zonefile"),
                serde_json::Value::String(String::from("example.com. 60 IN A 192.0.2.1")),
            );
        }
        "pricing" => {
            envelope.insert(
                String::from("pricing"),
                serde_json::json!({
                    "currency":"EUR","vat_rate":"19.0","primary_ips":[],
                    "floating_ips":[],"image":{},"volume":{},"server_backup":{},
                    "server_types":[],"load_balancer_types":[],"floating_ip":{}
                }),
            );
        }
        "folders" => {
            envelope.insert(String::from("folders"), serde_json::json!(["/backup"]));
        }
        "composite" | "empty" => {}
        _ => return String::from("null"),
    }
    if shape.ends_with("page") {
        let meta = serde_json::from_str(pagination()).unwrap_or(serde_json::Value::Null);
        envelope.insert(String::from("meta"), meta);
    }
    for field in required_fields.split(',').filter(|field| *field != "-") {
        if envelope.contains_key(field) {
            continue;
        }
        let value = match field {
            "action" => action_value(),
            "actions" | "next_actions" => serde_json::Value::Array(alloc::vec![action_value()]),
            "root_password" | "password" | "wss_url" => {
                serde_json::Value::String(String::from("sensitive"))
            }
            "meta" => serde_json::from_str(pagination()).unwrap_or(serde_json::Value::Null),
            _ => resource_value(field),
        };
        envelope.insert(String::from(field), value);
    }
    if shape == "composite" && root != "-" && !envelope.contains_key(root) {
        envelope.insert(String::from(root), resource_value(root));
    }
    serde_json::to_string(&serde_json::Value::Object(envelope)).unwrap_or_default()
}

fn action_value() -> serde_json::Value {
    serde_json::from_str(action()).unwrap_or(serde_json::Value::Null)
}

fn resource_value(root: &str) -> serde_json::Value {
    let id = if root == "rrset" || root == "rrsets" {
        serde_json::Value::String(String::from("rrset-id"))
    } else {
        serde_json::Value::from(1_u64)
    };
    let mut resource = serde_json::Map::new();
    resource.insert(String::from("id"), id);
    let status = match root {
        "server" | "servers" => Some("running"),
        "image" | "images" | "volume" | "volumes" => Some("available"),
        "zone" | "zones" => Some("ok"),
        "storage_box" | "storage_boxes" => Some("active"),
        _ => None,
    };
    if let Some(status) = status {
        resource.insert(
            String::from("status"),
            serde_json::Value::String(String::from(status)),
        );
    }
    serde_json::Value::Object(resource)
}
