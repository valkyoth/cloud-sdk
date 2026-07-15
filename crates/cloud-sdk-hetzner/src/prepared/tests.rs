use cloud_sdk::operation::{OperationImpact, PreparationStorage, PrepareOperation};
use cloud_sdk::transport::StatusCode;

use crate::actions::{ActionEndpoint, ActionId, ActionListRequest};
use crate::cloud::catalog::{CatalogGetEndpoint, CatalogId};

use super::HetznerPreparationError;

#[test]
fn prepares_global_actions_and_catalog_gets() {
    let ids = [ActionId::new(7).expect("nonzero action ID")];
    let operation = ActionListRequest::try_new(&ids).expect("bounded IDs");
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 1];
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .expect("complete action request");
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

    let endpoint = CatalogGetEndpoint::Location(CatalogId::new(3).expect("catalog ID"));
    let prepared = endpoint
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .expect("catalog request");
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
    let endpoint = ActionEndpoint::Get(ActionId::new(42).expect("action ID"));
    let mut target = [0xA5_u8; 4];
    let mut body = [0x5A_u8; 4];
    assert!(matches!(
        endpoint.prepare(PreparationStorage::new(&mut target, &mut body)),
        Err(HetznerPreparationError::Path)
    ));
    assert_eq!(target, [0; 4]);
    assert_eq!(body, [0; 4]);
}
