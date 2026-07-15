use cloud_sdk::operation::{OperationImpact, PreparationStorage, PrepareOperation};
use cloud_sdk::transport::StatusCode;

use crate::actions::{ActionEndpoint, ActionId, ActionListRequest};
use crate::cloud::catalog::{CatalogGetEndpoint, CatalogId};
use crate::cloud::shared::CloudResourceId;
use crate::dns::zones::{
    ZoneActionEndpoint, ZoneActionListRequest, ZoneEndpoint, ZoneProtectionRequest, ZoneReference,
};
use crate::storage::storage_boxes::StorageBoxListRequest;

use super::{HetznerPreparationError, HetznerPreparedOperation};

#[test]
fn prepares_global_actions_and_catalog_gets() {
    let id = ActionId::new(7);
    assert!(id.is_some());
    let Some(id) = id else { return };
    let ids = [id];
    let operation = ActionListRequest::try_new(&ids);
    assert!(operation.is_ok());
    let Ok(operation) = operation else { return };
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 1];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/actions?id=7"
    );
    assert_eq!(prepared.transport_request().body(), b"");
    assert_eq!(prepared.metadata().impact(), OperationImpact::ReadOnly);
    assert_eq!(
        prepared.response_policy().success_statuses(),
        &[StatusCode::OK]
    );

    let catalog_id = CatalogId::new(3);
    assert!(catalog_id.is_some());
    let Some(catalog_id) = catalog_id else {
        return;
    };
    let endpoint = CatalogGetEndpoint::Location(catalog_id);
    let prepared = endpoint.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/locations/3"
    );
}

#[test]
fn missing_required_components_clear_complete_storage() {
    let mut target = [0xA5_u8; 64];
    let mut body = [0x5A_u8; 32];
    assert!(matches!(
        ActionEndpoint::List.prepare(PreparationStorage::new(&mut target, &mut body)),
        Err(HetznerPreparationError::MissingQuery)
    ));
    assert!(target.iter().all(|byte| *byte == 0));
    assert!(body.iter().all(|byte| *byte == 0));
}

#[test]
fn insufficient_target_storage_never_exposes_partial_bytes() {
    let id = ActionId::new(42);
    assert!(id.is_some());
    let Some(id) = id else { return };
    let endpoint = ActionEndpoint::Get(id);
    let mut target = [0xA5_u8; 4];
    let mut body = [0x5A_u8; 4];
    assert!(matches!(
        endpoint.prepare(PreparationStorage::new(&mut target, &mut body)),
        Err(HetznerPreparationError::Path)
    ));
    assert_eq!(target, [0; 4]);
    assert_eq!(body, [0; 4]);
}

#[test]
fn checked_pairing_supports_local_filters_and_rejects_mismatches() {
    let zone_id = CloudResourceId::new(9);
    assert!(zone_id.is_some());
    let Some(zone_id) = zone_id else { return };
    let zone = ZoneReference::Id(zone_id);
    let operation = HetznerPreparedOperation::query(
        ZoneActionEndpoint::ListForZone(zone),
        ZoneActionListRequest::new(),
    );
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 8];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/zones/9/actions"
    );

    target.fill(0xA5);
    body.fill(0x5A);
    let mismatch =
        HetznerPreparedOperation::query(ZoneEndpoint::List, StorageBoxListRequest::new());
    assert!(matches!(
        mismatch.prepare(PreparationStorage::new(&mut target, &mut body)),
        Err(HetznerPreparationError::OperationMismatch)
    ));
    assert!(target.iter().all(|byte| *byte == 0));
    assert!(body.iter().all(|byte| *byte == 0));
}

#[test]
fn prepares_exact_json_and_clears_failed_body_storage() {
    let zone_id = CloudResourceId::new(11);
    assert!(zone_id.is_some());
    let Some(zone_id) = zone_id else { return };
    let request = ZoneProtectionRequest::new(ZoneReference::Id(zone_id), true);
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 32];
    let prepared = request.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/zones/11/actions/change_protection"
    );
    assert_eq!(prepared.transport_request().body(), br#"{"delete":true}"#);

    let mut short_target = [0xA5_u8; 64];
    let mut short_body = [0x5A_u8; 4];
    assert!(matches!(
        request.prepare(PreparationStorage::new(&mut short_target, &mut short_body)),
        Err(HetznerPreparationError::Body)
    ));
    assert!(short_target.iter().all(|byte| *byte == 0));
    assert!(short_body.iter().all(|byte| *byte == 0));
}
