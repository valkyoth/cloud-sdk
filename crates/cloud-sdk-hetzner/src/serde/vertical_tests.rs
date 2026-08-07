use alloc::format;
use alloc::string::String;

use cloud_sdk::transport::StatusCode;

use super::checked_test_support::{decode_response, empty_response, prepared, response};
use super::{HetznerDecodeError, HetznerSuccess, ResponseModelError, StorageBoxStatus};
use crate::{CLOUD_SERVICE_ID, DNS_SERVICE_ID, SECURITY_SERVICE_ID, STORAGE_SERVICE_ID};

const LOCATION: &str = r#"{"id":42,"name":"fsn1","description":"Falkenstein DC Park 1","country":"DE","city":"Falkenstein","latitude":50.47612,"longitude":12.370071,"network_zone":"eu-central"}"#;
const PAGINATION: &str = r#"{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":2,"last_page":2,"total_entries":26}}"#;
const CERTIFICATE: &[u8] = br#"{"certificate":{"id":897,"name":"website","labels":{"environment":"prod"},"type":"managed","certificate":"-----BEGIN CERTIFICATE-----\nsecret-fixture\n-----END CERTIFICATE-----","created":"2026-01-01T00:00:00Z","not_valid_before":"2026-01-01T00:00:00Z","not_valid_after":"2027-01-01T00:00:00Z","domain_names":["example.com","www.example.com"],"fingerprint":"03:c7:55","status":{"issuance":"completed","renewal":"scheduled","error":null},"used_by":[{"id":4711,"type":"load_balancer"}]}}"#;
const STORAGE_BOX: &str = r#"{"id":42,"name":"backup","storage_box_type":{"id":7,"name":"bx11","description":"BX11","snapshot_limit":10,"automatic_snapshot_limit":10,"subaccounts_limit":200,"size":1073741824,"prices":[{"location":"fsn1","price_hourly":{"net":"1.0000","gross":"1.1900"},"price_monthly":{"net":"5.0000","gross":"5.9500"},"setup_fee":{"net":"0.0000","gross":"0.0000"}}],"deprecation":{"unavailable_after":"2028-01-01T00:00:00Z","announced":"2027-01-01T00:00:00Z"}},"location":{"id":1,"name":"fsn1","description":"Falkenstein DC Park 1","country":"DE","city":"Falkenstein","latitude":50.47612,"longitude":12.370071,"network_zone":"eu-central"},"access_settings":{"reachable_externally":false,"samba_enabled":true,"ssh_enabled":true,"webdav_enabled":false,"zfs_enabled":true},"snapshot_plan":{"max_snapshots":10,"minute":30,"hour":3,"day_of_week":7,"day_of_month":null},"protection":{"delete":true},"labels":{"empty":""},"status":"active","username":"u12345","server":"u12345.your-storagebox.de","system":"FSN1-BX355","stats":{"size":3,"size_data":2,"size_snapshots":1},"created":"2026-01-01T00:00:00Z"}"#;

#[test]
fn source_complete_locations_preserve_every_field_and_pagination() {
    let body = format!(r#"{{"locations":[{LOCATION}],"meta":{PAGINATION}}}"#);
    let decoded = decode_response(
        prepared("list_locations", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, body.as_bytes()),
    );
    let Ok(decoded) = decoded else {
        unreachable!("source-complete location fixture failed")
    };
    let HetznerSuccess::Locations(page) = decoded.success() else {
        unreachable!("list_locations returned the wrong typed model")
    };
    assert_eq!(page.locations.len(), 1);
    let Some(location) = page.locations.first() else {
        unreachable!("location fixture disappeared")
    };
    assert_eq!(location.id, 42);
    assert_eq!(location.country, "DE");
    assert_eq!(location.longitude, 12.370071);
    assert_eq!(page.pagination.total_entries(), Some(26));
}

#[test]
fn certificate_pem_and_status_errors_stay_protected() {
    let decoded = decode_response(
        prepared("get_certificate", SECURITY_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, CERTIFICATE),
    );
    let Ok(decoded) = decoded else {
        unreachable!("source-complete certificate fixture failed")
    };
    let HetznerSuccess::Certificate(certificate) = decoded.success() else {
        unreachable!("get_certificate returned the wrong typed model")
    };
    assert_eq!(certificate.domain_names.len(), 2);
    let Some(used_by) = certificate.used_by.first() else {
        unreachable!("certificate resource fixture disappeared")
    };
    assert_eq!(used_by.resource_type, "load_balancer");
    let Some(pem) = &certificate.certificate else {
        unreachable!("certificate PEM was not retained")
    };
    assert_eq!(
        pem.try_with_secret(|value| value.contains("secret-fixture")),
        Ok(true)
    );
    assert!(!format!("{certificate:?}").contains("secret-fixture"));
}

#[test]
fn storage_box_uses_incremental_admission_and_preserves_nested_fields() {
    assert!(STORAGE_BOX.len() > 257);
    let body = format!(r#"{{"storage_boxes":[{STORAGE_BOX}],"meta":{PAGINATION}}}"#);
    let decoded = decode_response(
        prepared("list_storage_boxes", STORAGE_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, body.as_bytes()),
    );
    let Ok(decoded) = decoded else {
        unreachable!("source-complete Storage Box fixture failed")
    };
    let HetznerSuccess::StorageBoxes(page) = decoded.success() else {
        unreachable!("list_storage_boxes returned the wrong typed model")
    };
    let Some(storage_box) = page.storage_boxes.first() else {
        unreachable!("Storage Box fixture disappeared")
    };
    assert_eq!(storage_box.status, StorageBoxStatus::Active);
    assert!(!storage_box.access_settings.reachable_externally);
    assert!(storage_box.access_settings.ssh_enabled);
    let Some(price) = storage_box.storage_box_type.prices.first() else {
        unreachable!("Storage Box price fixture disappeared")
    };
    assert_eq!(price.monthly.net, "5.0000");
    assert_eq!(
        storage_box.labels.get("empty").map(String::as_str),
        Some("")
    );
}

#[test]
fn source_complete_pages_reject_more_items_than_declared_per_page() {
    let one_item_page = r#"{"pagination":{"page":1,"per_page":1,"previous_page":null,"next_page":2,"last_page":2,"total_entries":2}}"#;
    let locations = format!(
        r#"{{"locations":[{0},{0}],"meta":{1}}}"#,
        LOCATION, one_item_page,
    );
    assert_eq!(
        decode_response(
            prepared("list_locations", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, locations.as_bytes()),
        ),
        Err(HetznerDecodeError::Model(
            ResponseModelError::InvalidPagination,
        )),
    );

    let boxes = format!(
        r#"{{"storage_boxes":[{0},{0}],"meta":{1}}}"#,
        STORAGE_BOX, one_item_page,
    );
    assert_eq!(
        decode_response(
            prepared("list_storage_boxes", STORAGE_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, boxes.as_bytes()),
        ),
        Err(HetznerDecodeError::Model(
            ResponseModelError::InvalidPagination,
        )),
    );
}

#[test]
fn selected_action_zonefile_error_and_empty_paths_remain_exact() {
    let action = br#"{"action":{"id":42,"command":"poweron","status":"running","progress":0,"started":"2026-01-01T00:00:00Z","finished":null,"resources":[{"id":42,"type":"server"}],"error":null}}"#;
    assert!(matches!(
        decode_response(
            prepared("poweron_server", CLOUD_SERVICE_ID, StatusCode::CREATED),
            response(StatusCode::CREATED, action),
        )
        .map(|value| value.into_success()),
        Ok(HetznerSuccess::Action(_))
    ));

    let zone = decode_response(
        prepared("get_zone_zonefile", DNS_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, br#"{"zonefile":"$ORIGIN example.com.\n"}"#),
    );
    assert!(matches!(
        zone.map(|value| value.into_success()),
        Ok(HetznerSuccess::ZoneFile(_))
    ));

    let provider_error = decode_response(
        prepared("list_locations", CLOUD_SERVICE_ID, StatusCode::OK),
        response(
            StatusCode::new(400).unwrap_or(StatusCode::TOO_MANY_REQUESTS),
            br#"{"error":{"code":"invalid_input","message":"protected detail"}}"#,
        ),
    );
    assert!(matches!(
        provider_error,
        Err(HetznerDecodeError::Provider(_))
    ));

    let empty = decode_response(
        prepared(
            "delete_certificate",
            SECURITY_SERVICE_ID,
            StatusCode::NO_CONTENT,
        ),
        empty_response(StatusCode::NO_CONTENT),
    );
    assert!(matches!(
        empty.map(|value| value.into_success()),
        Ok(HetznerSuccess::Empty)
    ));
}
