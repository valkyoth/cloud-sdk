use cloud_sdk::operation::PreparationStorage;

use super::operations::{
    ChangeZoneProtection, GetAction, GetActions, ListCertificates, ListStorageBoxes,
};
use super::{
    ALL_OPERATIONS, AssociatedOperation, AssociationError, AuthenticationClass, BodyPolicy,
    HetznerOperation, PaginationPolicy, PermitClass, QueryPolicy, ResponseShape,
};
use crate::actions::{ActionEndpoint, ActionId, ActionListRequest};
use crate::dns::zones::{ZoneProtectionRequest, ZoneReference};
use crate::security::certificates::CertificateEndpoint;
use crate::storage::storage_boxes::StorageBoxEndpoint;
use crate::{
    CLOUD_SERVICE_ID, DNS_SERVICE_ID, SECURITY_SERVICE_ID, STORAGE_SERVICE_ID,
    cloud::shared::CloudResourceId,
};

#[test]
fn registry_is_complete_unique_and_stably_sorted() {
    assert_eq!(ALL_OPERATIONS.len(), 208);
    assert!(ALL_OPERATIONS.windows(2).all(|pair| {
        let Some(left) = pair.first() else {
            return false;
        };
        let Some(right) = pair.get(1) else {
            return false;
        };
        left.operation_id() < right.operation_id()
    }));
    assert_eq!(
        ALL_OPERATIONS
            .iter()
            .filter(|descriptor| descriptor.service_id() == CLOUD_SERVICE_ID)
            .count(),
        139
    );
    assert_eq!(
        ALL_OPERATIONS
            .iter()
            .filter(|descriptor| descriptor.service_id() == DNS_SERVICE_ID)
            .count(),
        24
    );
    assert_eq!(
        ALL_OPERATIONS
            .iter()
            .filter(|descriptor| descriptor.service_id() == SECURITY_SERVICE_ID)
            .count(),
        14
    );
    assert_eq!(
        ALL_OPERATIONS
            .iter()
            .filter(|descriptor| descriptor.service_id() == STORAGE_SERVICE_ID)
            .count(),
        31
    );
}

#[test]
fn descriptors_cover_every_policy_dimension() {
    let action = GetAction::DESCRIPTOR;
    assert_eq!(action.service_id(), CLOUD_SERVICE_ID);
    assert_eq!(action.authentication(), AuthenticationClass::Bearer);
    assert_eq!(action.query_policy(), QueryPolicy::Forbidden);
    assert_eq!(action.body_policy(), BodyPolicy::Forbidden);
    assert_eq!(action.response_shape(), ResponseShape::Action);
    assert_eq!(action.pagination(), PaginationPolicy::None);
    assert_eq!(action.permit(), PermitClass::None);

    let actions = GetActions::DESCRIPTOR;
    assert_eq!(actions.query_policy(), QueryPolicy::Required);
    assert_eq!(actions.response_shape(), ResponseShape::Actions);

    let zone = ChangeZoneProtection::DESCRIPTOR;
    assert_eq!(zone.service_id(), DNS_SERVICE_ID);
    assert_eq!(zone.body_policy(), BodyPolicy::RequiredJson);
    assert_eq!(zone.permit(), PermitClass::Destructive);

    let storage = ListStorageBoxes::DESCRIPTOR;
    assert_eq!(storage.service_id(), STORAGE_SERVICE_ID);
    assert_eq!(storage.authentication(), AuthenticationClass::Basic);
    assert_eq!(storage.query_policy(), QueryPolicy::Optional);
}

#[test]
fn typed_preparation_preserves_operation_and_runtime_policy() -> Result<(), &'static str> {
    let action_id = ActionId::new(7).ok_or("action ID")?;
    let action = AssociatedOperation::<GetAction, _>::endpoint(ActionEndpoint::Get(action_id))
        .map_err(|_| "action association")?;
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 64];
    let prepared = action
        .prepare_typed(PreparationStorage::new(&mut target, &mut body))
        .map_err(|_| "action preparation")?;
    assert_eq!(prepared.association(), GetAction::DESCRIPTOR);
    assert_eq!(
        prepared.as_untyped().transport_request().target().as_str(),
        "/actions/7"
    );

    let ids = [action_id];
    let query = ActionListRequest::try_new(&ids).map_err(|_| "action query")?;
    let actions = AssociatedOperation::<GetActions, _, _>::query(ActionEndpoint::List, query)
        .map_err(|_| "query association")?;
    let prepared = actions
        .prepare_typed(PreparationStorage::new(&mut target, &mut body))
        .map_err(|_| "query preparation")?;
    assert_eq!(prepared.association(), GetActions::DESCRIPTOR);
    Ok(())
}

#[test]
fn typed_preparation_checks_dns_security_and_storage_policies() -> Result<(), &'static str> {
    let zone_id = CloudResourceId::new(9).ok_or("zone ID")?;
    let zone = ZoneReference::Id(zone_id);
    let request = ZoneProtectionRequest::new(zone, true);
    let operation =
        AssociatedOperation::<ChangeZoneProtection, _, _, _>::json(request.endpoint(), request)
            .map_err(|_| "DNS association")?;
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = operation
        .prepare_typed(PreparationStorage::new(&mut target, &mut body))
        .map_err(|_| "DNS preparation")?;
    assert_eq!(prepared.association().service_id(), DNS_SERVICE_ID);

    let certificate =
        AssociatedOperation::<ListCertificates, _>::endpoint(CertificateEndpoint::List)
            .map_err(|_| "security association")?;
    assert!(
        certificate
            .prepare_typed(PreparationStorage::new(&mut target, &mut body))
            .is_ok()
    );

    let storage = AssociatedOperation::<ListStorageBoxes, _>::endpoint(StorageBoxEndpoint::List)
        .map_err(|_| "storage association")?;
    assert!(
        storage
            .prepare_typed(PreparationStorage::new(&mut target, &mut body))
            .is_ok()
    );
    Ok(())
}

#[test]
fn wrong_endpoint_and_missing_components_fail_before_writing() {
    assert!(matches!(
        AssociatedOperation::<GetActions, _>::endpoint(ActionEndpoint::List),
        Err(AssociationError::QueryRequired)
    ));
    assert!(matches!(
        AssociatedOperation::<GetAction, _>::endpoint(ActionEndpoint::List),
        Err(AssociationError::EndpointMismatch)
    ));
    assert!(matches!(
        AssociatedOperation::<ChangeZoneProtection, _>::endpoint(
            crate::dns::zones::ZoneActionEndpoint::ChangeProtection(ZoneReference::Id(
                CloudResourceId::new(1).unwrap_or_else(|| unreachable!())
            ))
        ),
        Err(AssociationError::BodyRequired)
    ));
}
