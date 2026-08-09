//! Dedicated Hetzner Console response-model regressions.

use alloc::format;
use alloc::vec;

use cloud_sdk::transport::StatusCode;

use super::checked_fixtures::{action_value, resource_value};
use super::checked_test_support::{assert_decode_error, decode_response, prepared, response};
use super::checked_tests::pagination;
use super::{
    HetznerDecodeError, HetznerSuccess, ResponseModelError, StorageBoxResource, StorageBoxStatus,
};
use crate::STORAGE_SERVICE_ID;

fn envelope_body(root: &str, value: serde_json::Value) -> alloc::vec::Vec<u8> {
    serde_json::to_vec(&serde_json::json!({root:value})).unwrap_or_default()
}

fn paged_body(root: &str, value: serde_json::Value) -> alloc::vec::Vec<u8> {
    let meta = serde_json::from_str::<serde_json::Value>(pagination()).unwrap_or_default();
    serde_json::to_vec(&serde_json::json!({root:[value], "meta":meta})).unwrap_or_default()
}

fn assert_model_error(
    operation: &'static str,
    root: &str,
    value: serde_json::Value,
    expected: ResponseModelError,
) {
    let body = envelope_body(root, value);
    assert_decode_error(
        decode_response(
            prepared(operation, STORAGE_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(expected),
    );
}

#[test]
fn boxes_and_types_use_source_complete_singleton_and_page_models() {
    for operation in ["get_storage_box", "update_storage_box"] {
        let body = envelope_body("storage_box", resource_value("storage_box"));
        let decoded = decode_response(
            prepared(operation, STORAGE_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        );
        let Ok(decoded) = decoded else {
            unreachable!("source-derived Storage Box fixture failed")
        };
        let HetznerSuccess::StorageBox(storage_box) = decoded.success() else {
            unreachable!("Storage Box singleton selected the wrong model")
        };
        assert_eq!(storage_box.id(), 1);
        assert_eq!(storage_box.status(), StorageBoxStatus::Active);
        assert_eq!(storage_box.username(), Some("u12345"));
        assert_eq!(storage_box.storage_box_type().prices().len(), 1);
        assert_eq!(storage_box.created(), "2026-01-01T00:00:00Z");
        let debug = format!("{storage_box:?}");
        assert!(!debug.contains("u12345"));
        assert!(!debug.contains("backup"));
    }

    let body = paged_body("storage_boxes", resource_value("storage_box"));
    let decoded = decode_response(
        prepared("list_storage_boxes", STORAGE_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("Storage Box page failed")
    };
    let HetznerSuccess::StorageBoxes(page) = decoded.success() else {
        unreachable!("Storage Box page selected the wrong model")
    };
    assert_eq!(page.storage_boxes().len(), 1);
    assert_eq!(page.pagination().total_entries(), Some(1));

    let body = paged_body("storage_box_types", resource_value("storage_box_type"));
    let decoded = decode_response(
        prepared("list_storage_box_types", STORAGE_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("Storage Box type page failed")
    };
    let HetznerSuccess::StorageBoxTypes(page) = decoded.success() else {
        unreachable!("Storage Box type page selected the wrong model")
    };
    assert_eq!(page.storage_box_types()[0].id(), 1);
    assert_eq!(page.storage_box_types()[0].prices().len(), 1);

    let body = envelope_body("storage_box_type", resource_value("storage_box_type"));
    let decoded = decode_response(
        prepared("get_storage_box_type", STORAGE_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::StorageBoxType(_))
    ));
}

#[test]
fn snapshots_and_subaccounts_preserve_every_source_field() {
    let snapshot = resource_value("snapshot");
    for operation in ["get_storage_box_snapshot", "update_storage_box_snapshot"] {
        let body = envelope_body("snapshot", snapshot.clone());
        let decoded = decode_response(
            prepared(operation, STORAGE_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        );
        let Ok(decoded) = decoded else {
            unreachable!("snapshot singleton failed")
        };
        let HetznerSuccess::StorageBoxSnapshot(snapshot) = decoded.success() else {
            unreachable!("snapshot singleton selected the wrong model")
        };
        assert_eq!(snapshot.storage_box(), 1);
        assert_eq!(snapshot.name(), "manual-2026-01-01");
        assert_eq!(snapshot.description(), "daily backup");
        assert!(!format!("{snapshot:?}").contains("daily backup"));
    }

    let body = envelope_body("snapshots", serde_json::json!([snapshot]));
    let decoded = decode_response(
        prepared(
            "list_storage_box_snapshots",
            STORAGE_SERVICE_ID,
            StatusCode::OK,
        ),
        response(StatusCode::OK, &body),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::StorageBoxSnapshots(values)) if values.len() == 1
    ));

    let subaccount = resource_value("subaccount");
    for operation in [
        "get_storage_box_subaccount",
        "update_storage_box_subaccount",
    ] {
        let body = envelope_body("subaccount", subaccount.clone());
        let decoded = decode_response(
            prepared(operation, STORAGE_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        );
        let Ok(decoded) = decoded else {
            unreachable!("subaccount singleton failed")
        };
        let HetznerSuccess::StorageBoxSubaccount(subaccount) = decoded.success() else {
            unreachable!("subaccount singleton selected the wrong model")
        };
        assert_eq!(subaccount.storage_box(), 1);
        assert_eq!(subaccount.home_directory(), "backups/server01");
        assert_eq!(subaccount.username(), "u12345-sub1");
        assert!(!format!("{subaccount:?}").contains("backups/server01"));
    }

    let body = envelope_body("subaccounts", serde_json::json!([subaccount]));
    let decoded = decode_response(
        prepared(
            "list_storage_box_subaccounts",
            STORAGE_SERVICE_ID,
            StatusCode::OK,
        ),
        response(StatusCode::OK, &body),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::StorageBoxSubaccounts(values)) if values.len() == 1
    ));
}

#[test]
fn create_composites_distinguish_complete_boxes_from_partial_references() {
    for (operation, root, value) in [
        (
            "create_storage_box",
            "storage_box",
            resource_value("storage_box"),
        ),
        (
            "create_storage_box_snapshot",
            "snapshot",
            serde_json::json!({"id":7,"storage_box":9}),
        ),
        (
            "create_storage_box_subaccount",
            "subaccount",
            serde_json::json!({"id":8,"storage_box":9}),
        ),
    ] {
        let payload = serde_json::to_vec(&serde_json::json!({
            root:value, "action":action_value()
        }))
        .unwrap_or_default();
        let decoded = decode_response(
            prepared(operation, STORAGE_SERVICE_ID, StatusCode::CREATED),
            response(StatusCode::CREATED, &payload),
        );
        let Ok(decoded) = decoded else {
            unreachable!("Console create composite failed")
        };
        let HetznerSuccess::Composite(composite) = decoded.success() else {
            unreachable!("Console create selected the wrong envelope")
        };
        assert!(composite.action().is_some());
        assert!(composite.resource().is_none());
        let Some(resource) = composite.storage_box_resource() else {
            unreachable!("Console resource disappeared")
        };
        match (operation, resource) {
            ("create_storage_box", StorageBoxResource::StorageBox(value)) => {
                assert_eq!(value.id(), 1);
            }
            ("create_storage_box_snapshot", StorageBoxResource::SnapshotReference(value)) => {
                assert_eq!((value.id(), value.storage_box()), (7, 9))
            }
            ("create_storage_box_subaccount", StorageBoxResource::SubaccountReference(value)) => {
                assert_eq!((value.id(), value.storage_box()), (8, 9))
            }
            _ => unreachable!("Console create selected the wrong resource model"),
        }
    }
}

#[test]
fn nullability_paths_and_timestamps_fail_closed() {
    for (status, username, expected) in [
        ("initializing", serde_json::json!("u12345"), true),
        ("initializing", serde_json::Value::Null, false),
        ("active", serde_json::Value::Null, true),
    ] {
        let mut value = resource_value("storage_box");
        let Some(fields) = value.as_object_mut() else {
            unreachable!("Storage Box fixture is not an object")
        };
        fields.insert("status".into(), serde_json::json!(status));
        for key in ["username", "server", "system"] {
            fields.insert(key.into(), username.clone());
        }
        if status == "initializing" {
            fields.insert("snapshot_plan".into(), serde_json::Value::Null);
        }
        let body = envelope_body("storage_box", value);
        let decoded = decode_response(
            prepared("get_storage_box", STORAGE_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        );
        assert_eq!(decoded.is_err(), expected);
    }

    for (root, operation) in [
        ("snapshot", "get_storage_box_snapshot"),
        ("subaccount", "get_storage_box_subaccount"),
    ] {
        let mut value = resource_value(root);
        let Some(fields) = value.as_object_mut() else {
            unreachable!("Console fixture is not an object")
        };
        fields.insert("created".into(), serde_json::json!("2026-02-30T00:00:00Z"));
        assert_model_error(operation, root, value, ResponseModelError::InvalidText);
    }
}

#[test]
fn source_character_and_collection_bounds_fail_closed() {
    let mut snapshot = resource_value("snapshot");
    let Some(fields) = snapshot.as_object_mut() else {
        unreachable!("snapshot fixture is not an object")
    };
    fields.insert("description".into(), serde_json::json!("not/allowed"));
    assert_model_error(
        "get_storage_box_snapshot",
        "snapshot",
        snapshot,
        ResponseModelError::InvalidText,
    );

    for home in ["/absolute", "backups\\server"] {
        let mut subaccount = resource_value("subaccount");
        let Some(fields) = subaccount.as_object_mut() else {
            unreachable!("subaccount fixture is not an object")
        };
        fields.insert("home_directory".into(), serde_json::json!(home));
        assert_model_error(
            "get_storage_box_subaccount",
            "subaccount",
            subaccount,
            ResponseModelError::InvalidText,
        );
    }

    let snapshots = vec![resource_value("snapshot"); 1_025];
    let body = envelope_body("snapshots", serde_json::Value::Array(snapshots));
    assert_decode_error(
        decode_response(
            prepared(
                "list_storage_box_snapshots",
                STORAGE_SERVICE_ID,
                StatusCode::OK,
            ),
            response(StatusCode::OK, &body),
        ),
        HetznerDecodeError::Model(ResponseModelError::TooManyItems),
    );
}

#[test]
fn large_console_lists_cross_incremental_chunk_boundaries() {
    let snapshots = vec![resource_value("snapshot"); 12];
    let body = envelope_body("snapshots", serde_json::Value::Array(snapshots));
    assert!(body.len() > 257);
    let decoded = decode_response(
        prepared(
            "list_storage_box_snapshots",
            STORAGE_SERVICE_ID,
            StatusCode::OK,
        ),
        response(StatusCode::OK, &body),
    );
    assert!(matches!(
        decoded.map(|value| value.into_success()),
        Ok(HetznerSuccess::StorageBoxSnapshots(values)) if values.len() == 12
    ));
}
