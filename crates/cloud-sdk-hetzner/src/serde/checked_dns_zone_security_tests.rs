//! DNS zone coherence and operational-bound regressions.

use alloc::format;

use cloud_sdk::transport::StatusCode;

use super::checked_fixtures::resource_value;
use super::checked_test_support::{assert_decode_error, decode_response, prepared, response};
use super::{
    DnsResource, HetznerDecodeError, HetznerSuccess, MAX_ZONE_RECORD_COUNT, ResponseModelError,
};
use crate::DNS_SERVICE_ID;

fn zone_body(zone: serde_json::Value) -> alloc::vec::Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"zone":zone}))
        .unwrap_or_else(|_| unreachable!("zone security fixture serialization failed"))
}

fn assert_zone_model_error(zone: serde_json::Value, expected: ResponseModelError) {
    let body = zone_body(zone);
    assert_decode_error(
        decode_response(
            prepared("get_zone", DNS_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(expected),
    );
}

#[test]
fn secondary_zones_require_at_least_one_unique_primary() {
    for primary_nameservers in [
        serde_json::json!([]),
        serde_json::json!([
            {"address":"2001:db8::1"},
            {"address":"2001:0db8:0:0:0:0:0:1"}
        ]),
    ] {
        let mut zone = resource_value("zone");
        let Some(fields) = zone.as_object_mut() else {
            unreachable!("zone security fixture is not an object")
        };
        fields.insert("primary_nameservers".into(), primary_nameservers);
        assert_zone_model_error(zone, ResponseModelError::EnvelopeMismatch);
    }

    let mut zone = resource_value("zone");
    let Some(fields) = zone.as_object_mut() else {
        unreachable!("zone security fixture is not an object")
    };
    fields.remove("primary_nameservers");
    assert_zone_model_error(zone, ResponseModelError::EnvelopeMismatch);
}

#[test]
fn primary_nameserver_tsig_fields_are_atomic() {
    for primary_nameserver in [
        serde_json::json!({
            "address":"192.0.2.1",
            "tsig_key":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
        }),
        serde_json::json!({
            "address":"192.0.2.1",
            "tsig_algorithm":"hmac-sha256"
        }),
    ] {
        let mut zone = resource_value("zone");
        let Some(fields) = zone.as_object_mut() else {
            unreachable!("zone security fixture is not an object")
        };
        fields.insert(
            "primary_nameservers".into(),
            serde_json::Value::Array(alloc::vec![primary_nameserver]),
        );
        assert_zone_model_error(zone, ResponseModelError::EnvelopeMismatch);
    }
}

#[test]
fn zone_record_count_accepts_the_local_ceiling_and_rejects_plus_one() {
    let mut exact = resource_value("zone");
    let Some(fields) = exact.as_object_mut() else {
        unreachable!("zone security fixture is not an object")
    };
    fields.insert("record_count".into(), MAX_ZONE_RECORD_COUNT.into());
    let body = zone_body(exact);
    let decoded = decode_response(
        prepared("get_zone", DNS_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("exact zone record-count ceiling was rejected")
    };
    let HetznerSuccess::DnsResource(DnsResource::Zone(zone)) = decoded.success() else {
        unreachable!("zone record-count fixture selected the wrong model")
    };
    assert_eq!(zone.record_count(), MAX_ZONE_RECORD_COUNT);

    let mut over = resource_value("zone");
    let Some(fields) = over.as_object_mut() else {
        unreachable!("zone security fixture is not an object")
    };
    fields.insert("record_count".into(), (MAX_ZONE_RECORD_COUNT + 1).into());
    assert_zone_model_error(over, ResponseModelError::InvalidNumber);
}

#[test]
fn checked_zone_labels_remain_redacted_when_accessed_directly() {
    let mut zone = resource_value("zone");
    let Some(fields) = zone.as_object_mut() else {
        unreachable!("zone security fixture is not an object")
    };
    fields.insert(
        "labels".into(),
        serde_json::json!({"internal-zone":"classified-topology"}),
    );
    let body = zone_body(zone);
    let decoded = decode_response(
        prepared("get_zone", DNS_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("zone label redaction fixture failed")
    };
    let HetznerSuccess::DnsResource(DnsResource::Zone(zone)) = decoded.success() else {
        unreachable!("zone label redaction fixture selected the wrong model")
    };
    let debug = format!("{:?}", zone.labels());
    assert!(!debug.contains("internal-zone"));
    assert!(!debug.contains("classified-topology"));
    assert!(debug.contains("[redacted]"));
}
