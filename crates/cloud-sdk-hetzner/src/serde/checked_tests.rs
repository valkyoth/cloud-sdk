use alloc::format;
use alloc::string::String;

use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestSemantics, ResponseBodyPolicy, ResponsePolicy,
    RetryEligibility,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointScheme, MediaType, RequestTarget, ResponseContentType, StatusCode,
    TransportRequest,
};
use cloud_sdk::{ApiFamily, Method, Provider};

use super::{HetznerDecodeError, HetznerSuccess, ResourceKind, decode_response};

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const CREATED: &[StatusCode] = &[StatusCode::CREATED];
const NO_CONTENT: &[StatusCode] = &[StatusCode::NO_CONTENT];

pub(super) fn prepared(
    operation: &'static str,
    family: ApiFamily,
    status: StatusCode,
) -> PreparedRequest<'static> {
    let target = RequestTarget::new("/test");
    assert!(target.is_ok());
    let statuses = if status == StatusCode::OK {
        OK
    } else if status == StatusCode::CREATED {
        CREATED
    } else {
        NO_CONTENT
    };
    let empty = status == StatusCode::NO_CONTENT;
    let policy = ResponsePolicy::new(
        statuses,
        if empty {
            ContentTypePolicy::Forbidden
        } else {
            ContentTypePolicy::Required(JSON)
        },
        if empty {
            ResponseBodyPolicy::Forbidden
        } else {
            ResponseBodyPolicy::Required
        },
        if empty { 0 } else { 8_388_608 },
    );
    assert!(policy.is_ok());
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
    );
    assert!(metadata.is_ok());
    let endpoint = EndpointIdentity::new(
        EndpointScheme::Https,
        if family == ApiFamily::Storage {
            "api.hetzner.com"
        } else {
            "api.hetzner.cloud"
        },
        443,
        "/v1",
    );
    assert!(endpoint.is_ok());
    let operation_id = OperationId::new(operation);
    assert!(operation_id.is_ok());
    PreparedRequest::new(
        TransportRequest::new(Method::Get, target.unwrap_or_else(|_| unreachable!())),
        ProviderService::new(
            Provider::Hetzner,
            family,
            endpoint.unwrap_or_else(|_| unreachable!()),
        ),
        metadata.unwrap_or_else(|_| unreachable!()),
        policy.unwrap_or_else(|_| unreachable!()),
    )
    .with_operation_id(operation_id.unwrap_or_else(|_| unreachable!()))
}

pub(super) fn response(
    status: StatusCode,
    body: &[u8],
) -> cloud_sdk::transport::TransportResponse<'_> {
    let content_type = ResponseContentType::new("application/json; charset=utf-8");
    assert!(content_type.is_ok());
    cloud_sdk::transport::TransportResponse::new(status, body)
        .with_content_type(content_type.unwrap_or_else(|_| unreachable!()))
}

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
        prepared("get_action", ApiFamily::Cloud, StatusCode::OK),
        response(StatusCode::OK, single.as_bytes()),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::Action(_))
    ));

    let actions = format!(r#"{{"actions":[{}]}}"#, action());
    let decoded = decode_response(
        prepared("get_actions", ApiFamily::Cloud, StatusCode::OK),
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
            prepared("list_servers_actions", ApiFamily::Cloud, StatusCode::OK),
            response(StatusCode::OK, paged_actions.as_bytes()),
        )
        .is_ok()
    );

    let server = br#"{"server":{"id":42,"name":"web-1","status":"running","future":true}}"#;
    let decoded = decode_response(
        prepared("get_server", ApiFamily::Cloud, StatusCode::OK),
        response(StatusCode::OK, server),
    );
    let Ok(decoded) = decoded else { return };
    let HetznerSuccess::Resource(resource) = decoded.success() else {
        return;
    };
    assert_eq!(resource.kind(), ResourceKind::Server);
    assert_eq!(resource.name(), Some("web-1"));

    let servers = format!(
        r#"{{"servers":[{{"id":42,"name":"web-1","status":"running"}}],"meta":{}}}"#,
        pagination()
    );
    let decoded = decode_response(
        prepared("list_servers", ApiFamily::Cloud, StatusCode::OK),
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
        prepared("create_server", ApiFamily::Cloud, StatusCode::CREATED),
        response(StatusCode::CREATED, create.as_bytes()),
    );
    let Ok(decoded) = decoded else { return };
    let HetznerSuccess::Composite(composite) = decoded.success() else {
        return;
    };
    assert_eq!(composite.secrets().len(), 1);
    let Some(secret) = composite.secrets().first() else {
        return;
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
            prepared("get_server_metrics", ApiFamily::Cloud, StatusCode::OK),
            response(StatusCode::OK, metrics),
        )
        .is_ok()
    );
    let zonefile = br#"{"zonefile":"example.com. 60 IN A 192.0.2.1"}"#;
    let decoded = decode_response(
        prepared("get_zone_zonefile", ApiFamily::Cloud, StatusCode::OK),
        response(StatusCode::OK, zonefile),
    );
    let Ok(decoded) = decoded else { return };
    let HetznerSuccess::ZoneFile(zonefile) = decoded.success() else {
        return;
    };
    assert_eq!(
        zonefile.try_with_zonefile(|value| value == "example.com. 60 IN A 192.0.2.1"),
        Ok(true)
    );
    let pricing = br#"{"pricing":{"currency":"EUR","vat_rate":"19.0","primary_ips":[],"floating_ips":[],"image":{},"volume":{},"server_backup":{},"server_types":[],"load_balancer_types":[],"floating_ip":{}}}"#;
    assert!(
        decode_response(
            prepared("get_pricing", ApiFamily::Cloud, StatusCode::OK),
            response(StatusCode::OK, pricing),
        )
        .is_ok()
    );
    let folders = br#"{"folders":["/backup"]}"#;
    assert!(
        decode_response(
            prepared(
                "list_storage_box_folders",
                ApiFamily::Storage,
                StatusCode::OK,
            ),
            response(StatusCode::OK, folders),
        )
        .is_ok()
    );
    let empty = cloud_sdk::transport::TransportResponse::new(StatusCode::NO_CONTENT, b"");
    assert!(
        decode_response(
            prepared(
                "delete_certificate",
                ApiFamily::Cloud,
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
            prepared("get_server", ApiFamily::Cloud, StatusCode::OK),
            response(StatusCode::OK, duplicate),
        ),
        Err(HetznerDecodeError::MalformedPayload)
    );
    let unknown = br#"{"server":{"id":1,"status":"future"}}"#;
    assert!(matches!(
        decode_response(
            prepared("get_server", ApiFamily::Cloud, StatusCode::OK),
            response(StatusCode::OK, unknown),
        ),
        Err(HetznerDecodeError::Model(_))
    ));
    assert_eq!(
        decode_response(
            prepared("get_server", ApiFamily::Storage, StatusCode::OK),
            response(StatusCode::OK, br#"{"server":{"id":1}}"#),
        ),
        Err(HetznerDecodeError::ServiceMismatch)
    );
    assert!(matches!(
        decode_response(
            prepared("get_server", ApiFamily::Cloud, StatusCode::OK),
            response(StatusCode::CREATED, br#"{"server":{"id":1}}"#),
        ),
        Err(HetznerDecodeError::ResponsePolicy(_))
    ));
}

#[test]
fn returns_typed_redacted_provider_errors() {
    let body = br#"{"error":{"code":"rate_limit_exceeded","message":"slow down"}}"#;
    let decoded = decode_response(
        prepared("get_server", ApiFamily::Cloud, StatusCode::OK),
        response(StatusCode::TOO_MANY_REQUESTS, body),
    );
    let error = match &decoded {
        Err(HetznerDecodeError::Provider(error)) => Some(error),
        _ => None,
    };
    assert_eq!(error.map(|error| error.message()), Some("slow down"));
    assert!(
        error
            .map(|error| format!("{error:?}"))
            .is_some_and(|debug| !debug.contains("slow down"))
    );
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
        let family = if api == "hetzner" {
            ApiFamily::Storage
        } else {
            ApiFamily::Cloud
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
            cloud_sdk::transport::TransportResponse::new(status, b"")
        } else {
            response(status, body.as_bytes())
        };
        let decoded = decode_response(prepared(operation, family, status), response);
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
