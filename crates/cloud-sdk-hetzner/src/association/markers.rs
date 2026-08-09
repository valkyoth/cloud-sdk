//! Generated exhaustive active-operation associations.
//!
//! Regenerate with `scripts/generate_operation_associations.py`.

use cloud_sdk::{ServiceMarker, operation_id};

use super::policy::{
    HetznerOperation, OperationDescriptor, ReadOnlyOperation, ResponseIdentityClass, Sealed,
};
use super::types::*;
use crate::identity::{CloudService, DnsService, SecurityService, StorageService};

macro_rules! body_headers {
    (BodyForbidden) => {
        AcceptJson
    };
    (JsonBody) => {
        AcceptAndContentTypeJson
    };
}
macro_rules! body_media {
    (BodyForbidden) => {
        NoRequestMedia
    };
    (JsonBody) => {
        JsonRequestMedia
    };
}
macro_rules! success_body {
    (EmptyResponse) => {
        EmptySuccessBody
    };
    ($response:ident) => {
        JsonSuccessBody
    };
}
macro_rules! success_media {
    (EmptyResponse) => {
        ForbiddenSuccessMedia
    };
    ($response:ident) => {
        JsonSuccessMedia
    };
}

macro_rules! read_only_operation {
    ($marker:ident, NoPermit) => {
        impl ReadOnlyOperation for $marker {}
    };
    ($marker:ident, $permit:ident) => {};
}

macro_rules! operation_associations {
    ($(($marker:ident, $id:literal, $service:ident, $endpoint:ident, $authentication:ident,
        $method:ident, $query:ident, $body:ident, $status:ident, $response:ident,
        $path:literal, $response_identity:expr,
        $pagination:ident, $retry:ident, $permit:ident),)+) => {
        /// Sealed markers for every active source-locked Hetzner operation.
        pub mod operations {
            use super::*;
            $(
                #[doc = concat!("Association for Hetzner operation `", $id, "`.")]
                #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
                pub struct $marker;
                impl Sealed for $marker {}
                impl HetznerOperation for $marker {
                    type Service = $service;
                    type EndpointPolicy = $endpoint;
                    type Authentication = $authentication;
                    type AuthenticationScope = RequiredServiceScope;
                    type Query = $query;
                    type Body = $body;
                    type RequestHeaders = body_headers!($body);
                    type RequestMedia = body_media!($body);
                    type SuccessStatus = $status;
                    type SuccessBody = success_body!($response);
                    type SuccessMedia = success_media!($response);
                    type ErrorBody = JsonErrorBody;
                    type ErrorMedia = JsonErrorMedia;
                    type ResponseCaps = JsonResponseCaps;
                    type Pagination = $pagination;
                    type Quota = HetznerQuota;
                    type Retry = $retry;
                    type Streaming = BufferedStreaming;
                    type Success = $response;
                    type Error = HetznerErrorResponse;
                    type Permit = $permit;
                    const DESCRIPTOR: OperationDescriptor = OperationDescriptor::new(
                        operation_id!($id),
                        <$service as ServiceMarker>::ID,
                        <$endpoint as EndpointAssociation>::BASE,
                        <$authentication as AuthenticationAssociation>::CLASS,
                        <$method as MethodAssociation>::METHOD,
                        <$query as QueryAssociation>::POLICY,
                        <$body as BodyAssociation>::POLICY,
                        <$status as StatusAssociation>::STATUS,
                        <$response as ResponseAssociation>::SHAPE,
                        $path,
                        $response_identity,
                        <$pagination as PaginationAssociation>::POLICY,
                        <$retry as RetryAssociation>::POLICY,
                        <$permit as PermitAssociation>::CLASS,
                    );
                }
                read_only_operation!($marker, $permit);
            )+
        }

        /// Descriptors for all active operations in stable operation-ID order.
        pub const ALL_OPERATIONS: &[OperationDescriptor] = &[
            $(<operations::$marker as HetznerOperation>::DESCRIPTOR,)+
        ];

        pub(crate) fn operation_path_template(operation_id: &str) -> Option<&'static str> {
            match operation_id {
                $($id => Some($path),)+
                _ => None,
            }
        }
    };
}

#[rustfmt::skip]
operation_associations!(
        (AddLoadBalancerService, "add_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/add_service", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AddLoadBalancerTarget, "add_load_balancer_target", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/add_target", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AddNetworkRoute, "add_network_route", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/networks/{id}/actions/add_route", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AddNetworkSubnet, "add_network_subnet", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/networks/{id}/actions/add_subnet", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AddServerToPlacementGroup, "add_server_to_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/servers/{id}/actions/add_to_placement_group", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AddZoneRrsetRecords, "add_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/add_records", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ApplyFirewallToResources, "apply_firewall_to_resources", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, "/firewalls/{id}/actions/apply_to_resources", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AssignFloatingIp, "assign_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/floating_ips/{id}/actions/assign", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AssignPrimaryIp, "assign_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/primary_ips/{id}/actions/assign", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AttachLoadBalancerToNetwork, "attach_load_balancer_to_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/attach_to_network", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AttachServerIso, "attach_server_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/servers/{id}/actions/attach_iso", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AttachServerToNetwork, "attach_server_to_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/servers/{id}/actions/attach_to_network", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (AttachVolume, "attach_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/volumes/{id}/actions/attach", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeFloatingIpDnsPtr, "change_floating_ip_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/floating_ips/{id}/actions/change_dns_ptr", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeFloatingIpProtection, "change_floating_ip_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/floating_ips/{id}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeImageProtection, "change_image_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/images/{id}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeLoadBalancerAlgorithm, "change_load_balancer_algorithm", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/change_algorithm", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeLoadBalancerDnsPtr, "change_load_balancer_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/change_dns_ptr", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeLoadBalancerProtection, "change_load_balancer_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeLoadBalancerType, "change_load_balancer_type", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/change_type", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (ChangeNetworkIpRange, "change_network_ip_range", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/networks/{id}/actions/change_ip_range", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeNetworkProtection, "change_network_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/networks/{id}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangePrimaryIpDnsPtr, "change_primary_ip_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/primary_ips/{id}/actions/change_dns_ptr", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangePrimaryIpProtection, "change_primary_ip_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/primary_ips/{id}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeServerAliasIps, "change_server_alias_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/servers/{id}/actions/change_alias_ips", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeServerDnsPtr, "change_server_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/servers/{id}/actions/change_dns_ptr", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeServerProtection, "change_server_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/servers/{id}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeServerType, "change_server_type", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/servers/{id}/actions/change_type", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (ChangeStorageBoxProtection, "change_storage_box_protection", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeStorageBoxSubaccountHomeDirectory, "change_storage_box_subaccount_home_directory", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/subaccounts/{subaccount_id}/actions/change_home_directory", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeStorageBoxType, "change_storage_box_type", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/actions/change_type", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (ChangeVolumeProtection, "change_volume_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/volumes/{id}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeZonePrimaryNameservers, "change_zone_primary_nameservers", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/actions/change_primary_nameservers", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeZoneProtection, "change_zone_protection", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeZoneRrsetProtection, "change_zone_rrset_protection", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/change_protection", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ChangeZoneRrsetTtl, "change_zone_rrset_ttl", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/change_ttl", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ChangeZoneTtl, "change_zone_ttl", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/actions/change_ttl", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (CreateCertificate, "create_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/certificates", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (CreateFirewall, "create_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/firewalls", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (CreateFloatingIp, "create_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/floating_ips", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (CreateLoadBalancer, "create_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/load_balancers", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (CreateNetwork, "create_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ResourceResponse, "/networks", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (CreatePlacementGroup, "create_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/placement_groups", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (CreatePrimaryIp, "create_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/primary_ips", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (CreateServer, "create_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/servers", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (CreateServerImage, "create_server_image", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/servers/{id}/actions/create_image", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (CreateSshKey, "create_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ResourceResponse, "/ssh_keys", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (CreateStorageBox, "create_storage_box", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/storage_boxes", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (CreateStorageBoxSnapshot, "create_storage_box_snapshot", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/storage_boxes/{id}/snapshots", ResponseIdentityClass::ParentResource, NoPagination, NeverRetry, MutationPermit),
        (CreateStorageBoxSubaccount, "create_storage_box_subaccount", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/storage_boxes/{id}/subaccounts", ResponseIdentityClass::ParentResource, NoPagination, NeverRetry, MutationPermit),
        (CreateVolume, "create_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/volumes", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (CreateZone, "create_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/zones", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (CreateZoneRrset, "create_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/zones/{id_or_name}/rrsets", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (DeleteCertificate, "delete_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/certificates/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteFirewall, "delete_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/firewalls/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteFloatingIp, "delete_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/floating_ips/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteImage, "delete_image", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/images/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteLoadBalancer, "delete_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/load_balancers/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteLoadBalancerService, "delete_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/delete_service", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteNetwork, "delete_network", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/networks/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteNetworkRoute, "delete_network_route", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/networks/{id}/actions/delete_route", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteNetworkSubnet, "delete_network_subnet", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/networks/{id}/actions/delete_subnet", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeletePlacementGroup, "delete_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/placement_groups/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeletePrimaryIp, "delete_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/primary_ips/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteServer, "delete_server", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/servers/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteSshKey, "delete_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/ssh_keys/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteStorageBox, "delete_storage_box", StorageService, StorageEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/storage_boxes/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteStorageBoxSnapshot, "delete_storage_box_snapshot", StorageService, StorageEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/storage_boxes/{id}/snapshots/{snapshot_id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteStorageBoxSubaccount, "delete_storage_box_subaccount", StorageService, StorageEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/storage_boxes/{id}/subaccounts/{subaccount_id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteVolume, "delete_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, "/volumes/{id}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteZone, "delete_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/zones/{id_or_name}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DeleteZoneRrset, "delete_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DetachLoadBalancerFromNetwork, "detach_load_balancer_from_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/detach_from_network", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DetachServerFromNetwork, "detach_server_from_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/servers/{id}/actions/detach_from_network", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DetachServerIso, "detach_server_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/detach_iso", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DetachVolume, "detach_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/volumes/{id}/actions/detach", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DisableLoadBalancerPublicInterface, "disable_load_balancer_public_interface", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/disable_public_interface", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DisableServerBackup, "disable_server_backup", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/disable_backup", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DisableServerRescue, "disable_server_rescue", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/disable_rescue", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (DisableStorageBoxSnapshotPlan, "disable_storage_box_snapshot_plan", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/storage_boxes/{id}/actions/disable_snapshot_plan", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (EnableLoadBalancerPublicInterface, "enable_load_balancer_public_interface", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/enable_public_interface", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (EnableServerBackup, "enable_server_backup", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/enable_backup", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (EnableServerRescue, "enable_server_rescue", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/servers/{id}/actions/enable_rescue", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (EnableStorageBoxSnapshotPlan, "enable_storage_box_snapshot_plan", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/actions/enable_snapshot_plan", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (GetAction, "get_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetActions, "get_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, ActionsResponse, "/actions", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetCertificate, "get_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/certificates/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetCertificatesAction, "get_certificates_action", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/certificates/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetFirewall, "get_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/firewalls/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetFirewallsAction, "get_firewalls_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/firewalls/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetFloatingIp, "get_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/floating_ips/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetFloatingIpsAction, "get_floating_ips_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/floating_ips/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetImage, "get_image", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/images/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetImagesAction, "get_images_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/images/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetIso, "get_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/isos/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetLoadBalancer, "get_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/load_balancers/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetLoadBalancerMetrics, "get_load_balancer_metrics", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, MetricsResponse, "/load_balancers/{id}/metrics", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetLoadBalancerType, "get_load_balancer_type", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/load_balancer_types/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetLoadBalancersAction, "get_load_balancers_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/load_balancers/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetLocation, "get_location", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/locations/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetNetwork, "get_network", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/networks/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetNetworksAction, "get_networks_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/networks/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetPlacementGroup, "get_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/placement_groups/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetPricing, "get_pricing", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, PricingResponse, "/pricing", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetPrimaryIp, "get_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/primary_ips/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetPrimaryIpsAction, "get_primary_ips_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/primary_ips/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetServer, "get_server", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/servers/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetServerMetrics, "get_server_metrics", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, MetricsResponse, "/servers/{id}/metrics", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetServerType, "get_server_type", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/server_types/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetServersAction, "get_servers_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/servers/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetSshKey, "get_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/ssh_keys/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBox, "get_storage_box", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/storage_boxes/{id}", ResponseIdentityClass::ExactResource, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBoxSnapshot, "get_storage_box_snapshot", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/storage_boxes/{id}/snapshots/{snapshot_id}", ResponseIdentityClass::ExactResource, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBoxSubaccount, "get_storage_box_subaccount", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/storage_boxes/{id}/subaccounts/{subaccount_id}", ResponseIdentityClass::ExactResource, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBoxType, "get_storage_box_type", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/storage_box_types/{id}", ResponseIdentityClass::ExactResource, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBoxesAction, "get_storage_boxes_action", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/storage_boxes/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetVolume, "get_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/volumes/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetVolumesAction, "get_volumes_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/volumes/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetZone, "get_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/zones/{id_or_name}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetZoneRrset, "get_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetZoneZonefile, "get_zone_zonefile", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ZoneFileResponse, "/zones/{id_or_name}/zonefile", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (GetZonesAction, "get_zones_action", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, "/zones/actions/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (ImportZoneZonefile, "import_zone_zonefile", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/actions/import_zonefile", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ListCertificateActions, "list_certificate_actions", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/certificates/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListCertificates, "list_certificates", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/certificates", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListCertificatesActions, "list_certificates_actions", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/certificates/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFirewallActions, "list_firewall_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/firewalls/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFirewalls, "list_firewalls", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/firewalls", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFirewallsActions, "list_firewalls_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/firewalls/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFloatingIpActions, "list_floating_ip_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/floating_ips/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFloatingIps, "list_floating_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/floating_ips", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFloatingIpsActions, "list_floating_ips_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/floating_ips/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListImageActions, "list_image_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/images/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListImages, "list_images", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/images", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListImagesActions, "list_images_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/images/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListIsos, "list_isos", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/isos", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLoadBalancerActions, "list_load_balancer_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/load_balancers/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLoadBalancerTypes, "list_load_balancer_types", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/load_balancer_types", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLoadBalancers, "list_load_balancers", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/load_balancers", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLoadBalancersActions, "list_load_balancers_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/load_balancers/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLocations, "list_locations", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/locations", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListNetworkActions, "list_network_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/networks/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListNetworks, "list_networks", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/networks", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListNetworksActions, "list_networks_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/networks/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListPlacementGroups, "list_placement_groups", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/placement_groups", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListPrimaryIpActions, "list_primary_ip_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/primary_ips/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListPrimaryIps, "list_primary_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/primary_ips", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListPrimaryIpsActions, "list_primary_ips_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/primary_ips/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListServerActions, "list_server_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/servers/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListServerTypes, "list_server_types", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/server_types", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListServers, "list_servers", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/servers", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListServersActions, "list_servers_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/servers/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListSshKeys, "list_ssh_keys", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/ssh_keys", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxActions, "list_storage_box_actions", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/storage_boxes/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxFolders, "list_storage_box_folders", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, FoldersResponse, "/storage_boxes/{id}/folders", ResponseIdentityClass::None, NoPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxSnapshots, "list_storage_box_snapshots", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourceListResponse, "/storage_boxes/{id}/snapshots", ResponseIdentityClass::ParentResource, NoPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxSubaccounts, "list_storage_box_subaccounts", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourceListResponse, "/storage_boxes/{id}/subaccounts", ResponseIdentityClass::ParentResource, NoPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxTypes, "list_storage_box_types", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/storage_box_types", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxes, "list_storage_boxes", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/storage_boxes", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxesActions, "list_storage_boxes_actions", StorageService, StorageEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/storage_boxes/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListVolumeActions, "list_volume_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/volumes/{id}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListVolumes, "list_volumes", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/volumes", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListVolumesActions, "list_volumes_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/volumes/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListZoneActions, "list_zone_actions", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/zones/{id_or_name}/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListZoneRrsets, "list_zone_rrsets", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/zones/{id_or_name}/rrsets", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListZones, "list_zones", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, "/zones", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (ListZonesActions, "list_zones_actions", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, "/zones/actions", ResponseIdentityClass::None, NumberedPagination, ExplicitRetry, NoPermit),
        (PoweroffServer, "poweroff_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/poweroff", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (PoweronServer, "poweron_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/poweron", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (RebootServer, "reboot_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/reboot", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (RebuildServer, "rebuild_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, "/servers/{id}/actions/rebuild", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (RemoveFirewallFromResources, "remove_firewall_from_resources", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, "/firewalls/{id}/actions/remove_from_resources", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (RemoveLoadBalancerTarget, "remove_load_balancer_target", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/remove_target", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (RemoveServerFromPlacementGroup, "remove_server_from_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/remove_from_placement_group", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (RemoveZoneRrsetRecords, "remove_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/remove_records", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (RequestServerConsole, "request_server_console", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, CompositeResponse, "/servers/{id}/actions/request_console", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (ResetServer, "reset_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/reset", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ResetServerPassword, "reset_server_password", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, CompositeResponse, "/servers/{id}/actions/reset_password", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ResetStorageBoxPassword, "reset_storage_box_password", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/actions/reset_password", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ResetStorageBoxSubaccountPassword, "reset_storage_box_subaccount_password", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/subaccounts/{subaccount_id}/actions/reset_subaccount_password", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ResizeVolume, "resize_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/volumes/{id}/actions/resize", ResponseIdentityClass::None, NoPagination, NeverRetry, CostPermit),
        (RetryCertificate, "retry_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/certificates/{id}/actions/retry", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (RollbackStorageBoxSnapshot, "rollback_storage_box_snapshot", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/actions/rollback_snapshot", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (SetFirewallRules, "set_firewall_rules", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, "/firewalls/{id}/actions/set_rules", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (SetZoneRrsetRecords, "set_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/set_records", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (ShutdownServer, "shutdown_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/servers/{id}/actions/shutdown", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (UnassignFloatingIp, "unassign_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/floating_ips/{id}/actions/unassign", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (UnassignPrimaryIp, "unassign_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, "/primary_ips/{id}/actions/unassign", ResponseIdentityClass::None, NoPagination, NeverRetry, DestructivePermit),
        (UpdateCertificate, "update_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/certificates/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateFirewall, "update_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/firewalls/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateFloatingIp, "update_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/floating_ips/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateImage, "update_image", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/images/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateLoadBalancer, "update_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/load_balancers/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateLoadBalancerService, "update_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/load_balancers/{id}/actions/update_service", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (UpdateNetwork, "update_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/networks/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdatePlacementGroup, "update_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/placement_groups/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdatePrimaryIp, "update_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/primary_ips/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateServer, "update_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/servers/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateSshKey, "update_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/ssh_keys/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateStorageBox, "update_storage_box", StorageService, StorageEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/storage_boxes/{id}", ResponseIdentityClass::ExactResource, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateStorageBoxAccessSettings, "update_storage_box_access_settings", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/actions/update_access_settings", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (UpdateStorageBoxSnapshot, "update_storage_box_snapshot", StorageService, StorageEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/storage_boxes/{id}/snapshots/{snapshot_id}", ResponseIdentityClass::ExactResource, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateStorageBoxSubaccount, "update_storage_box_subaccount", StorageService, StorageEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/storage_boxes/{id}/subaccounts/{subaccount_id}", ResponseIdentityClass::ExactResource, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateStorageBoxSubaccountAccessSettings, "update_storage_box_subaccount_access_settings", StorageService, StorageEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, "/storage_boxes/{id}/subaccounts/{subaccount_id}/actions/update_access_settings", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
        (UpdateVolume, "update_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/volumes/{id}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateZone, "update_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/zones/{id_or_name}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateZoneRrset, "update_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}", ResponseIdentityClass::None, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateZoneRrsetRecords, "update_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusOk, ActionResponse, "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}/actions/update_records", ResponseIdentityClass::None, NoPagination, NeverRetry, MutationPermit),
);
