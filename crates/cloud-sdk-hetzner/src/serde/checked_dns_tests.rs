//! Dedicated DNS response-model regressions.

use alloc::format;
use alloc::vec::Vec;

use cloud_sdk::transport::StatusCode;

use super::checked_fixtures::{action_value, resource_value};
use super::checked_test_support::{assert_decode_error, decode_response, prepared, response};
use super::{
    DnsResource, HetznerDecodeError, HetznerSuccess, ResponseModelError, ZoneMode, ZoneStatus,
};
use crate::DNS_SERVICE_ID;

#[test]
fn secondary_zone_retains_complete_state_and_protects_tsig_key() {
    let body =
        serde_json::to_vec(&serde_json::json!({"zone":resource_value("zone")})).unwrap_or_default();
    let decoded = decode_response(
        prepared("get_zone", DNS_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("source-derived zone fixture failed")
    };
    let HetznerSuccess::DnsResource(DnsResource::Zone(zone)) = decoded.success() else {
        unreachable!("zone fixture selected the wrong response model")
    };
    assert_eq!(zone.id(), 1);
    assert_eq!(zone.name(), "example.com");
    assert_eq!(zone.mode(), ZoneMode::Secondary);
    assert_eq!(zone.status(), ZoneStatus::Ok);
    assert_eq!(zone.ttl(), 60);
    assert_eq!(zone.primary_nameservers().len(), 1);
    let Some(primary) = zone.primary_nameservers().first() else {
        unreachable!("secondary zone lost its primary nameserver")
    };
    assert_eq!(primary.address(), "192.0.2.1");
    assert_eq!(primary.effective_port(), 53);
    assert_eq!(
        primary
            .try_with_tsig_key(|key| key == Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")),
        Ok(true)
    );
    let debug = format!("{zone:?} {primary:?}");
    assert!(!debug.contains("example.com"));
    assert!(!debug.contains("192.0.2.1"));
    assert!(!debug.contains("AAAAAAAA"));
}

#[test]
fn zone_mode_and_nameserver_semantics_fail_closed() {
    let mut zone = resource_value("zone");
    let Some(fields) = zone.as_object_mut() else {
        unreachable!("zone fixture is not an object")
    };
    fields.insert("mode".into(), serde_json::json!("primary"));
    let body = serde_json::to_vec(&serde_json::json!({"zone":zone})).unwrap_or_default();
    assert_decode_error(
        decode_response(
            prepared("get_zone", DNS_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(ResponseModelError::EnvelopeMismatch),
    );

    let mut zone = resource_value("zone");
    let Some(fields) = zone.as_object_mut() else {
        unreachable!("zone fixture is not an object")
    };
    fields.insert("status".into(), serde_json::json!("future-state"));
    let body = serde_json::to_vec(&serde_json::json!({"zone":zone})).unwrap_or_default();
    assert_decode_error(
        decode_response(
            prepared("get_zone", DNS_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(ResponseModelError::UnknownEnumValue),
    );
}

#[test]
fn rrset_retains_unknown_types_but_rejects_ambiguous_records() {
    let mut rrset = resource_value("rrset");
    let Some(fields) = rrset.as_object_mut() else {
        unreachable!("RRSet fixture is not an object")
    };
    fields.insert("type".into(), serde_json::json!("FUTURE42"));
    fields.insert("ttl".into(), serde_json::Value::Null);
    let body = serde_json::to_vec(&serde_json::json!({"rrset":rrset})).unwrap_or_default();
    let decoded = decode_response(
        prepared("get_zone_rrset", DNS_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("open RRSet fixture failed")
    };
    let HetznerSuccess::DnsResource(DnsResource::Rrset(rrset)) = decoded.success() else {
        unreachable!("RRSet fixture selected the wrong response model")
    };
    assert_eq!(rrset.record_type().as_str(), "FUTURE42");
    assert_eq!(rrset.record_type().known(), None);
    assert_eq!(rrset.ttl(), None);
    assert_eq!(rrset.records().len(), 1);

    let mut duplicate = resource_value("rrset");
    let Some(fields) = duplicate.as_object_mut() else {
        unreachable!("RRSet fixture is not an object")
    };
    fields.insert(
        "records".into(),
        serde_json::json!([
            {"value":"192.0.2.1"},
            {"value":"192.0.2.1","comment":"duplicate"}
        ]),
    );
    let body = serde_json::to_vec(&serde_json::json!({"rrset":duplicate})).unwrap_or_default();
    assert_decode_error(
        decode_response(
            prepared("get_zone_rrset", DNS_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(ResponseModelError::InvalidText),
    );
}

#[test]
fn rrset_record_deduplication_handles_the_exact_source_bound() {
    let mut records = Vec::new();
    records
        .try_reserve_exact(4_096)
        .unwrap_or_else(|_| unreachable!("bounded RRSet test allocation failed"));
    for index in 0..4_096 {
        records.push(serde_json::json!({
            "value": format!("record-{index:04}.example.com.")
        }));
    }

    let mut rrset = resource_value("rrset");
    let Some(fields) = rrset.as_object_mut() else {
        unreachable!("RRSet fixture is not an object")
    };
    fields.insert("records".into(), serde_json::Value::Array(records));
    let body = serde_json::to_vec(&serde_json::json!({"rrset":rrset})).unwrap_or_default();
    let decoded = decode_response(
        prepared("get_zone_rrset", DNS_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("exact-bound RRSet fixture failed")
    };
    let HetznerSuccess::DnsResource(DnsResource::Rrset(rrset)) = decoded.success() else {
        unreachable!("RRSet fixture selected the wrong response model")
    };
    assert_eq!(rrset.records().len(), 4_096);

    let mut duplicate = resource_value("rrset");
    let Some(fields) = duplicate.as_object_mut() else {
        unreachable!("RRSet fixture is not an object")
    };
    let mut records = Vec::new();
    records
        .try_reserve_exact(4_096)
        .unwrap_or_else(|_| unreachable!("bounded RRSet test allocation failed"));
    for index in 0..4_095 {
        records.push(serde_json::json!({
            "value": format!("record-{index:04}.example.com.")
        }));
    }
    records.push(serde_json::json!({"value":"record-0000.example.com."}));
    fields.insert("records".into(), serde_json::Value::Array(records));
    let body = serde_json::to_vec(&serde_json::json!({"rrset":duplicate})).unwrap_or_default();
    assert_decode_error(
        decode_response(
            prepared("get_zone_rrset", DNS_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(ResponseModelError::InvalidText),
    );
}

#[test]
fn late_rrset_validation_errors_fail_closed_after_sensitive_allocations() {
    let mut duplicate_records = resource_value("rrset");
    let Some(fields) = duplicate_records.as_object_mut() else {
        unreachable!("RRSet cleanup fixture is not an object")
    };
    fields.insert(
        "records".into(),
        serde_json::json!([
            {"value":"192.0.2.77","comment":"first"},
            {"value":"192.0.2.77","comment":"second"}
        ]),
    );
    fields.insert("labels".into(), serde_json::json!({"topology":"private"}));
    let body =
        serde_json::to_vec(&serde_json::json!({"rrset":duplicate_records})).unwrap_or_default();
    assert_decode_error(
        decode_response(
            prepared("get_zone_rrset", DNS_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(ResponseModelError::InvalidText),
    );
}

#[test]
fn dns_lists_and_create_composites_keep_dedicated_models() {
    let meta = serde_json::json!({"pagination":{
        "page":1,"per_page":1,"previous_page":null,"next_page":null,
        "last_page":1,"total_entries":1
    }});
    let list = serde_json::to_vec(&serde_json::json!({
        "rrsets":[resource_value("rrset")], "meta":meta
    }))
    .unwrap_or_default();
    assert!(matches!(
        decode_response(
            prepared("list_zone_rrsets", DNS_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &list),
        )
        .map(|decoded| decoded.into_success()),
        Ok(HetznerSuccess::DnsResources {
            pagination: Some(_),
            ..
        })
    ));

    let create = serde_json::to_vec(&serde_json::json!({
        "zone":resource_value("zone"), "action":action_value()
    }))
    .unwrap_or_default();
    let decoded = decode_response(
        prepared("create_zone", DNS_SERVICE_ID, StatusCode::CREATED),
        response(StatusCode::CREATED, &create),
    );
    let Ok(decoded) = decoded else {
        unreachable!("zone composite fixture failed")
    };
    let HetznerSuccess::Composite(composite) = decoded.success() else {
        unreachable!("zone create selected the wrong response model")
    };
    assert!(matches!(
        composite.dns_resource(),
        Some(DnsResource::Zone(_))
    ));
    assert!(composite.resource().is_none());
    assert!(composite.cloud_resource().is_none());
}
