use cloud_sdk::operation::{CostIntent, OperationImpact, PreparationStorage, PrepareOperation};
use cloud_sdk::transport::StatusCode;

use crate::actions::{ActionEndpoint, ActionId, ActionListRequest};
use crate::cloud::catalog::{CatalogGetEndpoint, CatalogId};
use crate::cloud::firewalls::actions::{FirewallActionEndpoint, FirewallResourcesRequest};
use crate::cloud::firewalls::{FirewallId, FirewallResource};
use crate::cloud::images::ImageActionEndpoint;
use crate::cloud::load_balancers::{
    LoadBalancerActionEndpoint, LoadBalancerAlgorithm, LoadBalancerCreateRequest, LoadBalancerName,
    LoadBalancerType,
};
use crate::cloud::networks::actions::NetworkActionEndpoint;
use crate::cloud::networks::floating_ips::FloatingIpActionEndpoint;
use crate::cloud::networks::primary_ips::PrimaryIpActionEndpoint;
use crate::cloud::servers::ServerId;
use crate::cloud::servers::actions::{ServerActionEndpoint, ServerActionKind};
use crate::cloud::shared::CloudResourceId;
use crate::cloud::volumes::VolumeActionEndpoint;
use crate::dns::rrsets::{RrsetActionEndpoint, RrsetName, RrsetReference, RrsetType};
use crate::dns::zones::{
    ZoneActionEndpoint, ZoneActionListRequest, ZoneEndpoint, ZoneProtectionRequest, ZoneReference,
};
use crate::storage::storage_boxes::{
    StorageBoxActionEndpoint, StorageBoxCreateRequest, StorageBoxListRequest, StorageBoxLocation,
    StorageBoxName, StorageBoxPassword, StorageBoxSubaccountActionEndpoint, StorageBoxTypeRef,
};

use super::{EndpointWire, HetznerPreparationError, HetznerPreparedOperation};

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

#[test]
fn prepares_firewall_removal_with_destructive_metadata() {
    let firewall = FirewallId::new(42);
    let server = FirewallId::new(7);
    assert!(firewall.is_some() && server.is_some());
    let (Some(firewall), Some(server)) = (firewall, server) else {
        return;
    };
    let resources = [FirewallResource::Server(server)];
    let operation = HetznerPreparedOperation::json(
        FirewallActionEndpoint::RemoveFromResources(firewall),
        FirewallResourcesRequest::remove(&resources),
    );
    let mut target = [0_u8; 80];
    let mut body = [0_u8; 96];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/firewalls/42/actions/remove_from_resources"
    );
    assert_eq!(
        prepared.transport_request().body(),
        br#"{"remove_from":[{"server":{"id":7},"type":"server"}]}"#
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::Destructive);
}

#[test]
fn prepares_cost_bearing_load_balancer_create() {
    let name = LoadBalancerName::new("edge");
    let load_balancer_type = LoadBalancerType::new("lb11");
    assert!(name.is_ok() && load_balancer_type.is_ok());
    let (Ok(name), Ok(load_balancer_type)) = (name, load_balancer_type) else {
        return;
    };
    let request = LoadBalancerCreateRequest::new(name, load_balancer_type)
        .with_algorithm(LoadBalancerAlgorithm::LeastConnections)
        .with_public_interface(false);
    let mut target = [0_u8; 32];
    let mut body = [0_u8; 160];
    let prepared = request.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/load_balancers"
    );
    assert_eq!(
        prepared.transport_request().body(),
        br#"{"name":"edge","load_balancer_type":"lb11","algorithm":{"type":"least_connections"},"public_interface":false}"#
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::Mutation);
    assert_eq!(prepared.metadata().cost_intent(), CostIntent::MayIncurCost);
}

#[test]
fn destructive_metadata_covers_replacement_reset_and_protection_operations() {
    macro_rules! assert_destructive {
        ($endpoint:expr) => {{
            let metadata = EndpointWire::metadata($endpoint);
            assert!(metadata.is_ok());
            let Ok(metadata) = metadata else { return };
            assert_eq!(metadata.impact(), OperationImpact::Destructive);
        }};
    }

    let cloud_id = CloudResourceId::new(7);
    let server_id = ServerId::new(7);
    let rrset_name = RrsetName::new("www");
    assert!(cloud_id.is_some() && server_id.is_some() && rrset_name.is_ok());
    let (Some(cloud_id), Some(server_id), Ok(rrset_name)) = (cloud_id, server_id, rrset_name)
    else {
        return;
    };
    let zone = ZoneReference::Id(cloud_id);
    let rrset = RrsetReference::new(zone, rrset_name, RrsetType::A);

    assert_destructive!(ZoneActionEndpoint::ImportZoneFile(zone));
    assert_destructive!(RrsetActionEndpoint::SetRecords(rrset));
    assert_destructive!(RrsetActionEndpoint::RemoveRecords(rrset));
    assert_destructive!(FirewallActionEndpoint::SetRules(cloud_id));
    assert_destructive!(StorageBoxActionEndpoint::DisableSnapshotPlan(cloud_id));
    assert_destructive!(StorageBoxActionEndpoint::ResetPassword(cloud_id));
    assert_destructive!(StorageBoxActionEndpoint::RollbackSnapshot(cloud_id));
    assert_destructive!(StorageBoxSubaccountActionEndpoint::ResetPassword(
        cloud_id, cloud_id
    ));

    assert_destructive!(ServerActionEndpoint::Start(
        server_id,
        ServerActionKind::ChangeProtection
    ));
    assert_destructive!(ImageActionEndpoint::ChangeProtection(cloud_id));
    assert_destructive!(VolumeActionEndpoint::ChangeProtection(cloud_id));
    assert_destructive!(LoadBalancerActionEndpoint::ChangeProtection(cloud_id));
    assert_destructive!(NetworkActionEndpoint::ChangeProtection(cloud_id));
    assert_destructive!(FloatingIpActionEndpoint::ChangeProtection(cloud_id));
    assert_destructive!(PrimaryIpActionEndpoint::ChangeProtection(cloud_id));
    assert_destructive!(ZoneActionEndpoint::ChangeProtection(zone));
    assert_destructive!(RrsetActionEndpoint::ChangeProtection(rrset));
    assert_destructive!(StorageBoxActionEndpoint::ChangeProtection(cloud_id));
}

#[test]
fn enabling_server_backups_requires_cost_approval() {
    let server_id = ServerId::new(7);
    assert!(server_id.is_some());
    let Some(server_id) = server_id else { return };
    let metadata = EndpointWire::metadata(ServerActionEndpoint::Start(
        server_id,
        ServerActionKind::EnableBackup,
    ));
    assert!(metadata.is_ok());
    let Ok(metadata) = metadata else { return };
    assert_eq!(metadata.cost_intent(), CostIntent::MayIncurCost);
}

#[test]
fn prepares_storage_secret_atomically_for_the_storage_api() {
    let name = StorageBoxName::new("backup");
    let location = StorageBoxLocation::new("fsn1");
    let storage_box_type = StorageBoxTypeRef::new("bx20");
    let password = StorageBoxPassword::new("a\"b\\c");
    assert!(name.is_ok() && location.is_ok() && storage_box_type.is_ok() && password.is_ok());
    let (Ok(name), Ok(location), Ok(storage_box_type), Ok(password)) =
        (name, location, storage_box_type, password)
    else {
        return;
    };
    let request = StorageBoxCreateRequest::new(name, location, storage_box_type, password);
    let mut target = [0_u8; 32];
    let mut body = [0_u8; 160];
    let prepared = request.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    assert_eq!(
        prepared.transport_request().target().as_str(),
        "/storage_boxes"
    );
    assert_eq!(
        prepared.transport_request().body(),
        br#"{"location":"fsn1","name":"backup","password":"a\"b\\c","storage_box_type":"bx20"}"#
    );
    assert_eq!(prepared.service().endpoint().host(), "api.hetzner.com");

    let mut short_target = [0xA5_u8; 32];
    let mut short_body = [0x5A_u8; 20];
    assert!(matches!(
        request.prepare(PreparationStorage::new(&mut short_target, &mut short_body)),
        Err(HetznerPreparationError::Body)
    ));
    assert!(short_target.iter().all(|byte| *byte == 0));
    assert!(short_body.iter().all(|byte| *byte == 0));
}
