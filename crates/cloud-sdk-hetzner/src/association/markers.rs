//! Generated exhaustive active-operation associations.
//!
//! Regenerate with `scripts/generate_operation_associations.py`.

use cloud_sdk::{ServiceMarker, operation_id};

use super::policy::{HetznerOperation, OperationDescriptor, Sealed};
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
macro_rules! retry {
    (GetMethod) => {
        ExplicitRetry
    };
    (PutMethod) => {
        ExplicitRetry
    };
    ($method:ident) => {
        NeverRetry
    };
}

macro_rules! operation_associations {
    ($(($marker:ident, $id:literal, $service:ident, $endpoint:ident, $authentication:ident,
        $method:ident, $query:ident, $body:ident, $status:ident, $response:ident,
        $pagination:ident, $permit:ident),)+) => {
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
                    type Retry = retry!($method);
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
                        <$pagination as PaginationAssociation>::POLICY,
                        <$method as MethodAssociation>::RETRY,
                        <$permit as PermitAssociation>::CLASS,
                    );
                }
            )+
        }

        /// Descriptors for all active operations in stable operation-ID order.
        pub const ALL_OPERATIONS: &[OperationDescriptor] = &[
            $(<operations::$marker as HetznerOperation>::DESCRIPTOR,)+
        ];
    };
}

#[rustfmt::skip]
operation_associations!(
        (AddLoadBalancerService, "add_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AddLoadBalancerTarget, "add_load_balancer_target", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AddNetworkRoute, "add_network_route", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AddNetworkSubnet, "add_network_subnet", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AddServerToPlacementGroup, "add_server_to_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AddZoneRrsetRecords, "add_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ApplyFirewallToResources, "apply_firewall_to_resources", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, NoPagination, MutationPermit),
        (AssignFloatingIp, "assign_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AssignPrimaryIp, "assign_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AttachLoadBalancerToNetwork, "attach_load_balancer_to_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AttachServerIso, "attach_server_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AttachServerToNetwork, "attach_server_to_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (AttachVolume, "attach_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeFloatingIpDnsPtr, "change_floating_ip_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeFloatingIpProtection, "change_floating_ip_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeImageProtection, "change_image_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeLoadBalancerAlgorithm, "change_load_balancer_algorithm", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeLoadBalancerDnsPtr, "change_load_balancer_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeLoadBalancerProtection, "change_load_balancer_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeLoadBalancerType, "change_load_balancer_type", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, CostPermit),
        (ChangeNetworkIpRange, "change_network_ip_range", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeNetworkProtection, "change_network_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangePrimaryIpDnsPtr, "change_primary_ip_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangePrimaryIpProtection, "change_primary_ip_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeServerAliasIps, "change_server_alias_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeServerDnsPtr, "change_server_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeServerProtection, "change_server_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeServerType, "change_server_type", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, CostPermit),
        (ChangeStorageBoxProtection, "change_storage_box_protection", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeStorageBoxSubaccountHomeDirectory, "change_storage_box_subaccount_home_directory", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeStorageBoxType, "change_storage_box_type", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, CostPermit),
        (ChangeVolumeProtection, "change_volume_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeZonePrimaryNameservers, "change_zone_primary_nameservers", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeZoneProtection, "change_zone_protection", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeZoneRrsetProtection, "change_zone_rrset_protection", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ChangeZoneRrsetTtl, "change_zone_rrset_ttl", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (ChangeZoneTtl, "change_zone_ttl", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (CreateCertificate, "create_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (CreateFirewall, "create_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (CreateFloatingIp, "create_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, CostPermit),
        (CreateLoadBalancer, "create_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, CostPermit),
        (CreateNetwork, "create_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ResourceResponse, NoPagination, MutationPermit),
        (CreatePlacementGroup, "create_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (CreatePrimaryIp, "create_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, CostPermit),
        (CreateServer, "create_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, CostPermit),
        (CreateServerImage, "create_server_image", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, CostPermit),
        (CreateSshKey, "create_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ResourceResponse, NoPagination, MutationPermit),
        (CreateStorageBox, "create_storage_box", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, CostPermit),
        (CreateStorageBoxSnapshot, "create_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (CreateStorageBoxSubaccount, "create_storage_box_subaccount", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (CreateVolume, "create_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, CostPermit),
        (CreateZone, "create_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (CreateZoneRrset, "create_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (DeleteCertificate, "delete_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteFirewall, "delete_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteFloatingIp, "delete_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteImage, "delete_image", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteLoadBalancer, "delete_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteLoadBalancerService, "delete_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DeleteNetwork, "delete_network", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteNetworkRoute, "delete_network_route", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DeleteNetworkSubnet, "delete_network_subnet", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DeletePlacementGroup, "delete_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeletePrimaryIp, "delete_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteServer, "delete_server", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, DestructivePermit),
        (DeleteSshKey, "delete_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteStorageBox, "delete_storage_box", StorageService, StorageEndpointPolicy, BasicAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DeleteStorageBoxSnapshot, "delete_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DeleteStorageBoxSubaccount, "delete_storage_box_subaccount", StorageService, StorageEndpointPolicy, BasicAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DeleteVolume, "delete_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, DestructivePermit),
        (DeleteZone, "delete_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DeleteZoneRrset, "delete_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DetachLoadBalancerFromNetwork, "detach_load_balancer_from_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DetachServerFromNetwork, "detach_server_from_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DetachServerIso, "detach_server_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DetachVolume, "detach_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DisableLoadBalancerPublicInterface, "disable_load_balancer_public_interface", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DisableServerBackup, "disable_server_backup", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DisableServerRescue, "disable_server_rescue", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (DisableStorageBoxSnapshotPlan, "disable_storage_box_snapshot_plan", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (EnableLoadBalancerPublicInterface, "enable_load_balancer_public_interface", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (EnableServerBackup, "enable_server_backup", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, CostPermit),
        (EnableServerRescue, "enable_server_rescue", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (EnableStorageBoxSnapshotPlan, "enable_storage_box_snapshot_plan", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (GetAction, "get_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetActions, "get_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, ActionsResponse, NoPagination, NoPermit),
        (GetCertificate, "get_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetCertificatesAction, "get_certificates_action", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetFirewall, "get_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetFirewallsAction, "get_firewalls_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetFloatingIp, "get_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetFloatingIpsAction, "get_floating_ips_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetImage, "get_image", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetImagesAction, "get_images_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetIso, "get_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetLoadBalancer, "get_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetLoadBalancerMetrics, "get_load_balancer_metrics", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, MetricsResponse, NoPagination, NoPermit),
        (GetLoadBalancerType, "get_load_balancer_type", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetLoadBalancersAction, "get_load_balancers_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetLocation, "get_location", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetNetwork, "get_network", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetNetworksAction, "get_networks_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetPlacementGroup, "get_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetPricing, "get_pricing", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, PricingResponse, NoPagination, NoPermit),
        (GetPrimaryIp, "get_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetPrimaryIpsAction, "get_primary_ips_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetServer, "get_server", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetServerMetrics, "get_server_metrics", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, MetricsResponse, NoPagination, NoPermit),
        (GetServerType, "get_server_type", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetServersAction, "get_servers_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetSshKey, "get_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetStorageBox, "get_storage_box", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetStorageBoxSnapshot, "get_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetStorageBoxSubaccount, "get_storage_box_subaccount", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetStorageBoxType, "get_storage_box_type", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetStorageBoxesAction, "get_storage_boxes_action", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetVolume, "get_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetVolumesAction, "get_volumes_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (GetZone, "get_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetZoneRrset, "get_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, NoPermit),
        (GetZoneZonefile, "get_zone_zonefile", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ZoneFileResponse, NoPagination, NoPermit),
        (GetZonesAction, "get_zones_action", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NoPermit),
        (ImportZoneZonefile, "import_zone_zonefile", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ListCertificateActions, "list_certificate_actions", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListCertificates, "list_certificates", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListCertificatesActions, "list_certificates_actions", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListFirewallActions, "list_firewall_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListFirewalls, "list_firewalls", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListFirewallsActions, "list_firewalls_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListFloatingIpActions, "list_floating_ip_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListFloatingIps, "list_floating_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListFloatingIpsActions, "list_floating_ips_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListImageActions, "list_image_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListImages, "list_images", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListImagesActions, "list_images_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListIsos, "list_isos", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListLoadBalancerActions, "list_load_balancer_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListLoadBalancerTypes, "list_load_balancer_types", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListLoadBalancers, "list_load_balancers", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListLoadBalancersActions, "list_load_balancers_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListLocations, "list_locations", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListNetworkActions, "list_network_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListNetworks, "list_networks", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListNetworksActions, "list_networks_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListPlacementGroups, "list_placement_groups", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListPrimaryIpActions, "list_primary_ip_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListPrimaryIps, "list_primary_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListPrimaryIpsActions, "list_primary_ips_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListServerActions, "list_server_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListServerTypes, "list_server_types", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListServers, "list_servers", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListServersActions, "list_servers_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListSshKeys, "list_ssh_keys", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListStorageBoxActions, "list_storage_box_actions", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListStorageBoxFolders, "list_storage_box_folders", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, FoldersResponse, NoPagination, NoPermit),
        (ListStorageBoxSnapshots, "list_storage_box_snapshots", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourceListResponse, NoPagination, NoPermit),
        (ListStorageBoxSubaccounts, "list_storage_box_subaccounts", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourceListResponse, NoPagination, NoPermit),
        (ListStorageBoxTypes, "list_storage_box_types", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListStorageBoxes, "list_storage_boxes", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListStorageBoxesActions, "list_storage_boxes_actions", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListVolumeActions, "list_volume_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListVolumes, "list_volumes", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListVolumesActions, "list_volumes_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListZoneActions, "list_zone_actions", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (ListZoneRrsets, "list_zone_rrsets", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListZones, "list_zones", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, NoPermit),
        (ListZonesActions, "list_zones_actions", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, NoPermit),
        (PoweroffServer, "poweroff_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (PoweronServer, "poweron_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (RebootServer, "reboot_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (RebuildServer, "rebuild_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, DestructivePermit),
        (RemoveFirewallFromResources, "remove_firewall_from_resources", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, NoPagination, DestructivePermit),
        (RemoveLoadBalancerTarget, "remove_load_balancer_target", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (RemoveServerFromPlacementGroup, "remove_server_from_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (RemoveZoneRrsetRecords, "remove_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (RequestServerConsole, "request_server_console", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, CompositeResponse, NoPagination, MutationPermit),
        (ResetServer, "reset_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ResetServerPassword, "reset_server_password", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, CompositeResponse, NoPagination, DestructivePermit),
        (ResetStorageBoxPassword, "reset_storage_box_password", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ResetStorageBoxSubaccountPassword, "reset_storage_box_subaccount_password", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ResizeVolume, "resize_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, CostPermit),
        (RetryCertificate, "retry_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (RollbackStorageBoxSnapshot, "rollback_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (SetFirewallRules, "set_firewall_rules", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, NoPagination, DestructivePermit),
        (SetZoneRrsetRecords, "set_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (ShutdownServer, "shutdown_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (UnassignFloatingIp, "unassign_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (UnassignPrimaryIp, "unassign_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, DestructivePermit),
        (UpdateCertificate, "update_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateFirewall, "update_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateFloatingIp, "update_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateImage, "update_image", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateLoadBalancer, "update_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateLoadBalancerService, "update_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (UpdateNetwork, "update_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdatePlacementGroup, "update_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdatePrimaryIp, "update_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateServer, "update_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateSshKey, "update_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateStorageBox, "update_storage_box", StorageService, StorageEndpointPolicy, BasicAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateStorageBoxAccessSettings, "update_storage_box_access_settings", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (UpdateStorageBoxSnapshot, "update_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateStorageBoxSubaccount, "update_storage_box_subaccount", StorageService, StorageEndpointPolicy, BasicAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateStorageBoxSubaccountAccessSettings, "update_storage_box_subaccount_access_settings", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, MutationPermit),
        (UpdateVolume, "update_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateZone, "update_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateZoneRrset, "update_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, MutationPermit),
        (UpdateZoneRrsetRecords, "update_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusOk, ActionResponse, NoPagination, MutationPermit),
);
