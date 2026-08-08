use alloc::{format, string::String};

use cloud_sdk::transport::StatusCode;

use super::checked_fixtures::{action_value, minimal_body, resource_value};
use super::checked_test_support::{
    assert_decode_error, decode_response, decode_response_with_headers,
    decode_response_with_headers_at, empty_response, prepared, response,
};
use super::{CloudResourceKind, HetznerDecodeError, HetznerSuccess};
use crate::identity::{CLOUD_SERVICE_ID, STORAGE_SERVICE_ID};

pub(super) fn action() -> &'static str {
    r#"{"id":42,"command":"poweron_server","status":"running","progress":10,"started":"2026-07-16T00:00:00Z","finished":null,"resources":[{"id":7,"type":"server"}],"error":null}"#
}

pub(super) fn pagination() -> &'static str {
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

    let server = serde_json::json!({"server": resource_value("server")});
    let server = serde_json::to_vec(&server).unwrap_or_default();
    let decoded = decode_response(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &server),
    );
    let Ok(decoded) = decoded else {
        unreachable!("security fixture construction failed")
    };
    let HetznerSuccess::CloudResource(resource) = decoded.success() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(resource.kind(), CloudResourceKind::Server);
    assert_eq!(resource.name(), Some("x"));

    let meta =
        serde_json::from_str::<serde_json::Value>(pagination()).unwrap_or(serde_json::Value::Null);
    let servers = serde_json::json!({"servers":[resource_value("server")],"meta":meta});
    let servers = serde_json::to_vec(&servers).unwrap_or_default();
    let decoded = decode_response(
        prepared("list_servers", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &servers),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::CloudResources {
            pagination: Some(_),
            ..
        })
    ));
}

#[test]
fn decodes_composite_special_empty_and_storage_families() {
    let mut server = resource_value("server");
    let Some(server_fields) = server.as_object_mut() else {
        unreachable!("server fixture is not an object")
    };
    server_fields.insert(
        String::from("future_topology"),
        serde_json::json!("private-topology-canary"),
    );
    let create = serde_json::json!({
        "server":server,
        "action":action_value(),
        "next_actions":[],
        "root_password":"dont-log-this"
    });
    let create = serde_json::to_vec(&create).unwrap_or_default();
    let decoded = decode_response(
        prepared("create_server", CLOUD_SERVICE_ID, StatusCode::CREATED),
        response(StatusCode::CREATED, &create),
    );
    let Ok(decoded) = decoded else {
        unreachable!("security fixture construction failed")
    };
    let HetznerSuccess::Composite(composite) = decoded.success() else {
        unreachable!("security fixture construction failed");
    };
    assert!(composite.cloud_resource().is_some());
    assert!(composite.resource().is_none());
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
    let debug = format!("{composite:?}");
    assert!(!debug.contains("dont-log-this"));
    assert!(!debug.contains("private-topology-canary"));

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
        prepared("get_zone_zonefile", crate::DNS_SERVICE_ID, StatusCode::OK),
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
    let mut pricing = resource_value("pricing");
    let Some(currency) = pricing.get_mut("currency") else {
        unreachable!("pricing fixture has no currency")
    };
    *currency = serde_json::json!("CURRENCY-CANARY");
    let pricing = serde_json::to_vec(&serde_json::json!({"pricing":pricing})).unwrap_or_default();
    let decoded = decode_response(
        prepared("get_pricing", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &pricing),
    );
    let Ok(decoded) = decoded else {
        unreachable!("pricing fixture failed")
    };
    let HetznerSuccess::Pricing(pricing) = decoded.success() else {
        unreachable!("pricing model was not selected")
    };
    let pricing_copy = pricing.try_clone();
    assert_eq!(pricing_copy.as_ref(), Ok(pricing));
    assert!(!format!("{pricing:?}").contains("CURRENCY-CANARY"));
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
                crate::SECURITY_SERVICE_ID,
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
    assert_decode_error(
        decode_response(
            prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, duplicate),
        ),
        HetznerDecodeError::MalformedPayload,
    );
    let missing = br#"{"server":{"id":1,"status":"future"}}"#;
    assert!(matches!(
        decode_response(
            prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, missing),
        ),
        Err(HetznerDecodeError::Model(_))
    ));
    assert_decode_error(
        decode_response(
            prepared("get_server", STORAGE_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, br#"{"server":{"id":1}}"#),
        ),
        HetznerDecodeError::ServiceMismatch,
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
    let server = serde_json::to_vec(&serde_json::json!({"server":resource_value("server")}))
        .unwrap_or_default();
    let success = decode_response_with_headers(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &server),
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
    let server = serde_json::to_vec(&serde_json::json!({"server":resource_value("server")}))
        .unwrap_or_default();
    let without_clock = decode_response_with_headers(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &server),
        &headers,
    );
    assert!(matches!(without_clock, Err(HetznerDecodeError::Quota(_))));

    let with_clock = decode_response_with_headers_at(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &server),
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
            Some(service),
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
            fields.next(),
        )
        else {
            continue;
        };
        assert!(fields.next().is_none(), "invalid response binding row");
        let service_id = match (api, service) {
            ("cloud", "cloud") => CLOUD_SERVICE_ID,
            ("cloud", "dns") => crate::DNS_SERVICE_ID,
            ("cloud", "security") => crate::SECURITY_SERVICE_ID,
            ("hetzner", "storage") => STORAGE_SERVICE_ID,
            _ => unreachable!("invalid response binding service"),
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
