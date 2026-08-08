use alloc::string::String;

use super::checked_tests::{action, pagination};

pub(super) fn minimal_body(shape: &str, root: &str, required_fields: &str) -> String {
    let mut envelope = serde_json::Map::new();
    match shape {
        "action" => insert(&mut envelope, "action", action_value()),
        "actions" | "actions-page" => insert(
            &mut envelope,
            "actions",
            serde_json::Value::Array(alloc::vec![action_value()]),
        ),
        "resource" | "resource-list" | "resource-page" => insert(
            &mut envelope,
            root,
            if shape == "resource" {
                resource_value(root)
            } else {
                serde_json::Value::Array(alloc::vec![resource_value(root)])
            },
        ),
        "metrics" => insert(
            &mut envelope,
            "metrics",
            serde_json::json!({
                "start":"2026-01-01T00:00:00Z", "end":"2026-01-01T01:00:00Z",
                "step":60.0, "time_series":{}
            }),
        ),
        "zonefile" => insert(
            &mut envelope,
            "zonefile",
            serde_json::Value::String(String::from("example.com. 60 IN A 192.0.2.1")),
        ),
        "pricing" => insert(&mut envelope, "pricing", resource_value("pricing")),
        "folders" => insert(&mut envelope, "folders", serde_json::json!(["/backup"])),
        "composite" | "empty" => {}
        _ => return String::from("null"),
    }
    if shape.ends_with("page") {
        let meta = serde_json::from_str(pagination()).unwrap_or(serde_json::Value::Null);
        insert(&mut envelope, "meta", meta);
    }
    insert_required(&mut envelope, required_fields);
    if shape == "composite" && root != "-" && !envelope.contains_key(root) {
        insert(&mut envelope, root, resource_value(root));
    }
    serde_json::to_string(&serde_json::Value::Object(envelope)).unwrap_or_default()
}

fn insert_required(envelope: &mut serde_json::Map<String, serde_json::Value>, fields: &str) {
    for field in fields.split(',').filter(|field| *field != "-") {
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
        insert(envelope, field, value);
    }
}

fn insert(
    envelope: &mut serde_json::Map<String, serde_json::Value>,
    field: &str,
    value: serde_json::Value,
) {
    envelope.insert(String::from(field), value);
}

pub(super) fn action_value() -> serde_json::Value {
    serde_json::from_str(action()).unwrap_or(serde_json::Value::Null)
}

pub(super) fn resource_value(root: &str) -> serde_json::Value {
    if let Some(model) = cloud_model_name(root) {
        let fixtures =
            serde_json::from_str::<serde_json::Value>(include_str!("cloud_model_fixtures.json"));
        if let Ok(fixtures) = fixtures
            && let Some(value) = fixtures.get(model)
        {
            return value.clone();
        }
        return serde_json::Value::Null;
    }
    match root {
        "location" | "locations" => location_value(),
        "storage_box" | "storage_boxes" => storage_box_value(),
        _ => generic_resource_value(root),
    }
}

fn cloud_model_name(root: &str) -> Option<&'static str> {
    match root {
        "firewall" | "firewalls" => Some("firewall"),
        "floating_ip" | "floating_ips" => Some("floating_ip"),
        "image" | "images" => Some("image"),
        "iso" | "isos" => Some("iso"),
        "load_balancer" | "load_balancers" => Some("load_balancer"),
        "load_balancer_type" | "load_balancer_types" => Some("load_balancer_type"),
        "network" | "networks" => Some("network"),
        "placement_group" | "placement_groups" => Some("placement_group"),
        "pricing" => Some("pricing"),
        "primary_ip" | "primary_ips" => Some("primary_ip"),
        "server" | "servers" => Some("server"),
        "server_type" | "server_types" => Some("server_type"),
        "volume" | "volumes" => Some("volume"),
        "zone" | "zones" => Some("zone"),
        "rrset" | "rrsets" => Some("rrset"),
        "certificate" | "certificates" => Some("certificate"),
        "ssh_key" | "ssh_keys" => Some("ssh_key"),
        _ => None,
    }
}

fn location_value() -> serde_json::Value {
    serde_json::json!({
        "id":1, "name":"fsn1", "description":"Falkenstein DC Park 1",
        "country":"DE", "city":"Falkenstein", "latitude":50.47612,
        "longitude":12.370071, "network_zone":"eu-central"
    })
}

fn generic_resource_value(root: &str) -> serde_json::Value {
    let id = if root == "rrset" || root == "rrsets" {
        serde_json::Value::String(String::from("rrset-id"))
    } else {
        serde_json::Value::from(1_u64)
    };
    let mut resource = serde_json::Map::new();
    resource.insert(String::from("id"), id);
    let status = match root {
        "zone" | "zones" => Some("ok"),
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

fn storage_box_value() -> serde_json::Value {
    serde_json::json!({
        "id":1, "name":"backup",
        "storage_box_type":{
            "id":1, "name":"bx11", "description":"BX11", "snapshot_limit":10,
            "automatic_snapshot_limit":10, "subaccounts_limit":200, "size":1073741824,
            "prices":[{"location":"fsn1", "price_hourly":{"net":"1.0000","gross":"1.1900"},
                "price_monthly":{"net":"1.0000","gross":"1.1900"},
                "setup_fee":{"net":"0.0000","gross":"0.0000"}}], "deprecation":null
        },
        "location":location_value(),
        "access_settings":{"reachable_externally":false,"samba_enabled":true,
            "ssh_enabled":true,"webdav_enabled":false,"zfs_enabled":true},
        "snapshot_plan":{"max_snapshots":10,"minute":30,"hour":3,
            "day_of_week":7,"day_of_month":null},
        "protection":{"delete":false}, "labels":{"environment":"test"}, "status":"active",
        "username":"u12345", "server":"u12345.your-storagebox.de", "system":"FSN1-BX1",
        "stats":{"size":1,"size_data":1,"size_snapshots":0},
        "created":"2026-01-01T00:00:00Z"
    })
}
