use core::cell::Cell;

use cloud_sdk::Method;
use cloud_sdk::operation::OperationMetadata;
use cloud_sdk::operation::{PreparationStorage, PreparationStorageGuard};

use super::operations::{
    ChangeZoneProtection, GetAction, GetActions, ListCertificates, ListStorageBoxes,
};
use super::validation::{
    descriptor_policy_is_coherent, prepared_policy_matches, validate_association,
};
use super::{
    ALL_OPERATIONS, AssociatedOperation, AssociationError, AuthenticationClass, BodyPolicy,
    HetznerOperation, PaginationPolicy, PermitClass, QueryPolicy, ResponseShape,
};
use crate::actions::{ActionEndpoint, ActionId, ActionListRequest};
use crate::dns::zones::{ZoneProtectionRequest, ZoneReference};
use crate::endpoint::EndpointGroup;
use crate::prepared::{
    EndpointWire, HetznerPreparationError, NoBody, NoQuery, RequestShape, ResponseProfile,
    prepare_parts_with_policy,
};
use crate::request::ApiBaseUrl;
use crate::security::certificates::CertificateEndpoint;
use crate::storage::storage_boxes::StorageBoxEndpoint;
use crate::{
    CLOUD_SERVICE_ID, DNS_SERVICE_ID, SECURITY_SERVICE_ID, STORAGE_SERVICE_ID,
    cloud::shared::CloudResourceId,
};

#[derive(Clone, Copy)]
struct ChangingMethodEndpoint<'a> {
    inner: ActionEndpoint,
    method_reads: &'a Cell<usize>,
}

impl EndpointWire for ChangingMethodEndpoint<'_> {
    fn method(self) -> Method {
        let reads = self.method_reads.get();
        self.method_reads.set(reads.saturating_add(1));
        if reads == 0 {
            EndpointWire::method(self.inner)
        } else {
            Method::Delete
        }
    }

    fn api_base_url(self) -> ApiBaseUrl {
        EndpointWire::api_base_url(self.inner)
    }

    fn endpoint_group(self) -> EndpointGroup {
        EndpointWire::endpoint_group(self.inner)
    }

    fn write_path(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        EndpointWire::write_path(self.inner, output)
    }

    fn request_shape(self) -> RequestShape {
        EndpointWire::request_shape(self.inner)
    }

    fn response_profile(self) -> ResponseProfile {
        EndpointWire::response_profile(self.inner)
    }

    fn metadata(self) -> Result<OperationMetadata, HetznerPreparationError> {
        EndpointWire::metadata(self.inner)
    }

    fn operation_key(self) -> &'static str {
        EndpointWire::operation_key(self.inner)
    }
}

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
    assert!(
        ALL_OPERATIONS
            .iter()
            .copied()
            .all(descriptor_policy_is_coherent)
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
    prepared_policy_matches::<GetAction>(&prepared.as_untyped())
        .map_err(|_| "exact action policy")?;
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
    prepared_policy_matches::<GetActions>(&prepared.as_untyped())
        .map_err(|_| "exact actions policy")?;
    Ok(())
}

#[test]
fn typed_guard_owns_cleanup_for_complete_request_storage() -> Result<(), &'static str> {
    let action_id = ActionId::new(7).ok_or("action ID")?;
    let action = AssociatedOperation::<GetAction, _>::endpoint(ActionEndpoint::Get(action_id))
        .map_err(|_| "action association")?;
    let mut target = [0xA5_u8; 64];
    let mut body = [0x5A_u8; 64];
    {
        let mut storage = PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = action
            .prepare_typed_guarded(&mut storage)
            .map_err(|_| "guarded preparation")?;
        prepared_policy_matches::<GetAction>(&prepared.as_untyped())
            .map_err(|_| "guarded exact policy")?;
    }
    assert_eq!(target, [0; 64]);
    assert_eq!(body, [0; 64]);
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

#[test]
fn runtime_policy_disagreement_fails_during_write_free_preflight() {
    let action_id = ActionId::new(1).unwrap_or_else(|| unreachable!());
    assert_eq!(
        validate_association::<GetActions, _>(ActionEndpoint::Get(action_id)).map(|_| ()),
        Err(AssociationError::PreparedPolicyMismatch),
    );
}

#[test]
fn request_assembly_consumes_validated_policy_without_rereading_endpoint_method() {
    let action_id = ActionId::new(1).unwrap_or_else(|| unreachable!());
    let method_reads = Cell::new(0);
    let endpoint = ChangingMethodEndpoint {
        inner: ActionEndpoint::Get(action_id),
        method_reads: &method_reads,
    };
    let policy = validate_association::<GetAction, _>(endpoint)
        .unwrap_or_else(|_| unreachable!("canonical first snapshot must validate"));
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 64];
    let prepared = prepare_parts_with_policy(
        endpoint,
        NoQuery,
        NoBody,
        PreparationStorage::new(&mut target, &mut body),
        &policy,
    )
    .unwrap_or_else(|_| unreachable!("validated policy must drive assembly"));

    assert_eq!(method_reads.get(), 1);
    assert_eq!(prepared.transport_request().method(), Method::Get);
    assert!(prepared_policy_matches::<GetAction>(&prepared).is_ok());
}

#[test]
fn typed_validation_failure_clears_reused_storage_before_returning() {
    let action_id = ActionId::new(1).unwrap_or_else(|| unreachable!());
    let method_reads = Cell::new(1);
    let endpoint = ChangingMethodEndpoint {
        inner: ActionEndpoint::Get(action_id),
        method_reads: &method_reads,
    };
    let operation = AssociatedOperation::<GetAction, _>::endpoint(endpoint)
        .unwrap_or_else(|_| unreachable!("operation key remains canonical"));
    let mut target = [0xA5_u8; 64];
    let mut body = [0x5A_u8; 64];
    let result = operation.prepare_typed(PreparationStorage::new(&mut target, &mut body));

    assert!(matches!(
        result,
        Err(super::AssociatedPreparationError::Association(
            AssociationError::PreparedPolicyMismatch
        ))
    ));
    assert_eq!(target, [0; 64]);
    assert_eq!(body, [0; 64]);
}
