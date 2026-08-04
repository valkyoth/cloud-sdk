//! Generated exhaustive active-operation associations.
//!
//! Regenerate with `scripts/generate_operation_associations.py`.

use cloud_sdk::{ServiceMarker, operation_id};

use super::policy::{HetznerOperation, OperationDescriptor, ReadOnlyOperation, Sealed};
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
    };
}

#[rustfmt::skip]
operation_associations!(
        (AddLoadBalancerService, "add_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AddLoadBalancerTarget, "add_load_balancer_target", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AddNetworkRoute, "add_network_route", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AddNetworkSubnet, "add_network_subnet", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AddServerToPlacementGroup, "add_server_to_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AddZoneRrsetRecords, "add_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ApplyFirewallToResources, "apply_firewall_to_resources", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, NoPagination, NeverRetry, MutationPermit),
        (AssignFloatingIp, "assign_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AssignPrimaryIp, "assign_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AttachLoadBalancerToNetwork, "attach_load_balancer_to_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AttachServerIso, "attach_server_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AttachServerToNetwork, "attach_server_to_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (AttachVolume, "attach_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeFloatingIpDnsPtr, "change_floating_ip_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeFloatingIpProtection, "change_floating_ip_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeImageProtection, "change_image_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeLoadBalancerAlgorithm, "change_load_balancer_algorithm", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeLoadBalancerDnsPtr, "change_load_balancer_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeLoadBalancerProtection, "change_load_balancer_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeLoadBalancerType, "change_load_balancer_type", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, CostPermit),
        (ChangeNetworkIpRange, "change_network_ip_range", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeNetworkProtection, "change_network_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangePrimaryIpDnsPtr, "change_primary_ip_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangePrimaryIpProtection, "change_primary_ip_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeServerAliasIps, "change_server_alias_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeServerDnsPtr, "change_server_dns_ptr", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeServerProtection, "change_server_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeServerType, "change_server_type", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, CostPermit),
        (ChangeStorageBoxProtection, "change_storage_box_protection", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeStorageBoxSubaccountHomeDirectory, "change_storage_box_subaccount_home_directory", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeStorageBoxType, "change_storage_box_type", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, CostPermit),
        (ChangeVolumeProtection, "change_volume_protection", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeZonePrimaryNameservers, "change_zone_primary_nameservers", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeZoneProtection, "change_zone_protection", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeZoneRrsetProtection, "change_zone_rrset_protection", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ChangeZoneRrsetTtl, "change_zone_rrset_ttl", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (ChangeZoneTtl, "change_zone_ttl", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (CreateCertificate, "create_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (CreateFirewall, "create_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (CreateFloatingIp, "create_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, CostPermit),
        (CreateLoadBalancer, "create_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, CostPermit),
        (CreateNetwork, "create_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ResourceResponse, NoPagination, NeverRetry, MutationPermit),
        (CreatePlacementGroup, "create_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (CreatePrimaryIp, "create_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, CostPermit),
        (CreateServer, "create_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, CostPermit),
        (CreateServerImage, "create_server_image", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, CostPermit),
        (CreateSshKey, "create_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ResourceResponse, NoPagination, NeverRetry, MutationPermit),
        (CreateStorageBox, "create_storage_box", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, CostPermit),
        (CreateStorageBoxSnapshot, "create_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (CreateStorageBoxSubaccount, "create_storage_box_subaccount", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (CreateVolume, "create_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, CostPermit),
        (CreateZone, "create_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (CreateZoneRrset, "create_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (DeleteCertificate, "delete_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteFirewall, "delete_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteFloatingIp, "delete_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteImage, "delete_image", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteLoadBalancer, "delete_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteLoadBalancerService, "delete_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteNetwork, "delete_network", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteNetworkRoute, "delete_network_route", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteNetworkSubnet, "delete_network_subnet", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeletePlacementGroup, "delete_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeletePrimaryIp, "delete_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteServer, "delete_server", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteSshKey, "delete_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteStorageBox, "delete_storage_box", StorageService, StorageEndpointPolicy, BasicAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteStorageBoxSnapshot, "delete_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteStorageBoxSubaccount, "delete_storage_box_subaccount", StorageService, StorageEndpointPolicy, BasicAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteVolume, "delete_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusNoContent, EmptyResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteZone, "delete_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DeleteZoneRrset, "delete_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, DeleteMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DetachLoadBalancerFromNetwork, "detach_load_balancer_from_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DetachServerFromNetwork, "detach_server_from_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DetachServerIso, "detach_server_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DetachVolume, "detach_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DisableLoadBalancerPublicInterface, "disable_load_balancer_public_interface", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DisableServerBackup, "disable_server_backup", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DisableServerRescue, "disable_server_rescue", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (DisableStorageBoxSnapshotPlan, "disable_storage_box_snapshot_plan", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (EnableLoadBalancerPublicInterface, "enable_load_balancer_public_interface", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (EnableServerBackup, "enable_server_backup", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, CostPermit),
        (EnableServerRescue, "enable_server_rescue", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (EnableStorageBoxSnapshotPlan, "enable_storage_box_snapshot_plan", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (GetAction, "get_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetActions, "get_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, ActionsResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetCertificate, "get_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetCertificatesAction, "get_certificates_action", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetFirewall, "get_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetFirewallsAction, "get_firewalls_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetFloatingIp, "get_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetFloatingIpsAction, "get_floating_ips_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetImage, "get_image", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetImagesAction, "get_images_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetIso, "get_iso", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetLoadBalancer, "get_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetLoadBalancerMetrics, "get_load_balancer_metrics", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, MetricsResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetLoadBalancerType, "get_load_balancer_type", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetLoadBalancersAction, "get_load_balancers_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetLocation, "get_location", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetNetwork, "get_network", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetNetworksAction, "get_networks_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetPlacementGroup, "get_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetPricing, "get_pricing", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, PricingResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetPrimaryIp, "get_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetPrimaryIpsAction, "get_primary_ips_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetServer, "get_server", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetServerMetrics, "get_server_metrics", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, RequiredQuery, BodyForbidden, StatusOk, MetricsResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetServerType, "get_server_type", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetServersAction, "get_servers_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetSshKey, "get_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBox, "get_storage_box", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBoxSnapshot, "get_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBoxSubaccount, "get_storage_box_subaccount", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBoxType, "get_storage_box_type", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetStorageBoxesAction, "get_storage_boxes_action", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetVolume, "get_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetVolumesAction, "get_volumes_action", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetZone, "get_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetZoneRrset, "get_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetZoneZonefile, "get_zone_zonefile", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ZoneFileResponse, NoPagination, ExplicitRetry, NoPermit),
        (GetZonesAction, "get_zones_action", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, ActionResponse, NoPagination, ExplicitRetry, NoPermit),
        (ImportZoneZonefile, "import_zone_zonefile", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ListCertificateActions, "list_certificate_actions", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListCertificates, "list_certificates", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListCertificatesActions, "list_certificates_actions", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFirewallActions, "list_firewall_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFirewalls, "list_firewalls", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFirewallsActions, "list_firewalls_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFloatingIpActions, "list_floating_ip_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFloatingIps, "list_floating_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListFloatingIpsActions, "list_floating_ips_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListImageActions, "list_image_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListImages, "list_images", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListImagesActions, "list_images_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListIsos, "list_isos", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLoadBalancerActions, "list_load_balancer_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLoadBalancerTypes, "list_load_balancer_types", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLoadBalancers, "list_load_balancers", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLoadBalancersActions, "list_load_balancers_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListLocations, "list_locations", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListNetworkActions, "list_network_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListNetworks, "list_networks", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListNetworksActions, "list_networks_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListPlacementGroups, "list_placement_groups", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListPrimaryIpActions, "list_primary_ip_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListPrimaryIps, "list_primary_ips", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListPrimaryIpsActions, "list_primary_ips_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListServerActions, "list_server_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListServerTypes, "list_server_types", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListServers, "list_servers", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListServersActions, "list_servers_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListSshKeys, "list_ssh_keys", SecurityService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxActions, "list_storage_box_actions", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxFolders, "list_storage_box_folders", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, QueryForbidden, BodyForbidden, StatusOk, FoldersResponse, NoPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxSnapshots, "list_storage_box_snapshots", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourceListResponse, NoPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxSubaccounts, "list_storage_box_subaccounts", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourceListResponse, NoPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxTypes, "list_storage_box_types", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxes, "list_storage_boxes", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListStorageBoxesActions, "list_storage_boxes_actions", StorageService, StorageEndpointPolicy, BasicAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListVolumeActions, "list_volume_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListVolumes, "list_volumes", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListVolumesActions, "list_volumes_actions", CloudService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListZoneActions, "list_zone_actions", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListZoneRrsets, "list_zone_rrsets", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListZones, "list_zones", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ResourcePageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (ListZonesActions, "list_zones_actions", DnsService, CloudEndpointPolicy, BearerAuthentication, GetMethod, OptionalQuery, BodyForbidden, StatusOk, ActionsPageResponse, NumberedPagination, ExplicitRetry, NoPermit),
        (PoweroffServer, "poweroff_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (PoweronServer, "poweron_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (RebootServer, "reboot_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (RebuildServer, "rebuild_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, CompositeResponse, NoPagination, NeverRetry, DestructivePermit),
        (RemoveFirewallFromResources, "remove_firewall_from_resources", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, NoPagination, NeverRetry, DestructivePermit),
        (RemoveLoadBalancerTarget, "remove_load_balancer_target", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (RemoveServerFromPlacementGroup, "remove_server_from_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (RemoveZoneRrsetRecords, "remove_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (RequestServerConsole, "request_server_console", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, CompositeResponse, NoPagination, NeverRetry, MutationPermit),
        (ResetServer, "reset_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ResetServerPassword, "reset_server_password", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, CompositeResponse, NoPagination, NeverRetry, DestructivePermit),
        (ResetStorageBoxPassword, "reset_storage_box_password", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ResetStorageBoxSubaccountPassword, "reset_storage_box_subaccount_password", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ResizeVolume, "resize_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, CostPermit),
        (RetryCertificate, "retry_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (RollbackStorageBoxSnapshot, "rollback_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (SetFirewallRules, "set_firewall_rules", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionsResponse, NoPagination, NeverRetry, DestructivePermit),
        (SetZoneRrsetRecords, "set_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (ShutdownServer, "shutdown_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (UnassignFloatingIp, "unassign_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (UnassignPrimaryIp, "unassign_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, BodyForbidden, StatusCreated, ActionResponse, NoPagination, NeverRetry, DestructivePermit),
        (UpdateCertificate, "update_certificate", SecurityService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateFirewall, "update_firewall", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateFloatingIp, "update_floating_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateImage, "update_image", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateLoadBalancer, "update_load_balancer", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateLoadBalancerService, "update_load_balancer_service", CloudService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (UpdateNetwork, "update_network", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdatePlacementGroup, "update_placement_group", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdatePrimaryIp, "update_primary_ip", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateServer, "update_server", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateSshKey, "update_ssh_key", SecurityService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateStorageBox, "update_storage_box", StorageService, StorageEndpointPolicy, BasicAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateStorageBoxAccessSettings, "update_storage_box_access_settings", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (UpdateStorageBoxSnapshot, "update_storage_box_snapshot", StorageService, StorageEndpointPolicy, BasicAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateStorageBoxSubaccount, "update_storage_box_subaccount", StorageService, StorageEndpointPolicy, BasicAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateStorageBoxSubaccountAccessSettings, "update_storage_box_subaccount_access_settings", StorageService, StorageEndpointPolicy, BasicAuthentication, PostMethod, QueryForbidden, JsonBody, StatusCreated, ActionResponse, NoPagination, NeverRetry, MutationPermit),
        (UpdateVolume, "update_volume", CloudService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateZone, "update_zone", DnsService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateZoneRrset, "update_zone_rrset", DnsService, CloudEndpointPolicy, BearerAuthentication, PutMethod, QueryForbidden, JsonBody, StatusOk, ResourceResponse, NoPagination, ExplicitRetry, MutationPermit),
        (UpdateZoneRrsetRecords, "update_zone_rrset_records", DnsService, CloudEndpointPolicy, BearerAuthentication, PostMethod, QueryForbidden, JsonBody, StatusOk, ActionResponse, NoPagination, NeverRetry, MutationPermit),
);
