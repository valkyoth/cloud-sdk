//! Generated exhaustive methods for the official Hetzner Cloud client.
//!
//! Regenerate with `scripts/generate_cloud_client_methods.py`.

#![allow(private_bounds)]

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::client::{ClientExecutionError, ClientWorkspaceLease};
use cloud_sdk::operation::{PermitClock, PermitExecutionError, PreparationStorageGuard};
use cloud_sdk::transport::{BoundTransport, DeliveryClassified};

use super::{HetznerClient, OfficialEndpointTrust};
use crate::association::{
    AssociatedCheckedResponse, AssociatedOperation, AssociatedPermitAttempt,
    AssociatedPreparationError, HetznerOperation, OperationDescriptor, PaginationPolicy,
    PermitClass, Prepared, operations,
};
use crate::identity::CloudService;
use crate::prepared::{BodyWire, EndpointWire, QueryWire};
use crate::serde::{CheckedHetznerResponse, HetznerDecodeError};

/// Result returned by a complete read-only Cloud client method.
pub type CloudReadResult<E> = Result<
    CheckedHetznerResponse,
    ClientExecutionError<AssociatedPreparationError, E, HetznerDecodeError>,
>;

/// Source-locked Cloud operation exposed by the service-typed client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloudClientMethodDescriptor {
    operation: OperationDescriptor,
}

impl CloudClientMethodDescriptor {
    const fn new(operation: OperationDescriptor) -> Self {
        Self { operation }
    }

    /// Returns the complete operation contract behind this client method.
    #[must_use]
    pub const fn operation(self) -> OperationDescriptor {
        self.operation
    }

    /// Returns the required plan-confirm permit class.
    #[must_use]
    pub const fn permit(self) -> PermitClass {
        self.operation.permit()
    }

    /// Returns the operation pagination policy.
    #[must_use]
    pub const fn pagination(self) -> PaginationPolicy {
        self.operation.pagination()
    }
}

macro_rules! read_method {
    ($marker:ident, $blocking:ident, $asynchronous:ident, $local:ident) => {
        impl<T> HetznerClient<T, CloudService, OfficialEndpointTrust>
        where
            T: BlockingAuthenticatedTransport + BoundTransport,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` synchronously.")]
            pub fn $blocking<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> CloudReadResult<T::Error>
            where
                E: EndpointWire,
                Q: QueryWire,
                B: BodyWire,
            {
                self.execute_blocking(operation, lease)
            }
        }

        impl<T> HetznerClient<T, CloudService, OfficialEndpointTrust>
        where
            T: AsyncAuthenticatedTransport + BoundTransport + Sync,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` through a `Send` future.")]
            #[allow(clippy::manual_async_fn)]
            pub fn $asynchronous<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> impl core::future::Future<Output = CloudReadResult<T::Error>> + Send
            where
                E: EndpointWire + Sync,
                Q: QueryWire + Sync,
                B: BodyWire + Sync,
                T::Error: Send,
            {
                self.execute_async(operation, lease)
            }
        }

        impl<T> HetznerClient<T, CloudService, OfficialEndpointTrust>
        where
            T: LocalAsyncAuthenticatedTransport + BoundTransport,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` on a local executor.")]
            pub async fn $local<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> CloudReadResult<T::Error>
            where
                E: EndpointWire,
                Q: QueryWire,
                B: BodyWire,
            {
                self.execute_local_async(operation, lease).await
            }
        }
    };
}

macro_rules! permitted_method {
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident) => {
        impl<T> HetznerClient<T, CloudService, OfficialEndpointTrust> {
            #[doc = concat!("Prepares `", stringify!($marker), "` in cleanup-owning storage.")]
            pub fn $prepare<'guard, E, Q, B>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                storage: &'guard mut PreparationStorageGuard<'_>,
            ) -> Result<Prepared<'guard, operations::$marker>, AssociatedPreparationError>
            where
                E: EndpointWire,
                Q: QueryWire,
                B: BodyWire,
            {
                operation.prepare_typed_guarded(storage)
            }
        }

        impl<T> HetznerClient<T, CloudService, OfficialEndpointTrust>
        where
            T: BlockingAuthenticatedTransport + BoundTransport,
            T::Error: DeliveryClassified,
        {
            #[doc = concat!("Executes an authorized `", stringify!($marker), "` attempt synchronously.")]
            pub fn $blocking<'permit, 'request, 'fingerprint, 'buffer, C>(
                &self,
                attempt: AssociatedPermitAttempt<
                    'permit,
                    'request,
                    'fingerprint,
                    operations::$marker,
                >,
                clock: &C,
                body: &'buffer mut [u8],
                headers: &'buffer mut [u8],
            ) -> Result<
                AssociatedCheckedResponse<'buffer, operations::$marker>,
                PermitExecutionError<T::Error>,
            >
            where
                C: PermitClock + ?Sized,
            {
                attempt.execute_blocking(clock, self.transport(), body, headers)
            }
        }

        impl<T> HetznerClient<T, CloudService, OfficialEndpointTrust>
        where
            T: AsyncAuthenticatedTransport + BoundTransport + Sync,
            T::Error: DeliveryClassified + Send,
        {
            #[doc = concat!("Executes an authorized `", stringify!($marker), "` attempt through a `Send` future.")]
            #[allow(clippy::manual_async_fn)]
            pub fn $asynchronous<
                'transport,
                'permit,
                'request,
                'fingerprint,
                'buffer,
                C,
            >(
                &'transport self,
                attempt: AssociatedPermitAttempt<
                    'permit,
                    'request,
                    'fingerprint,
                    operations::$marker,
                >,
                clock: &'transport C,
                body: &'buffer mut [u8],
                headers: &'buffer mut [u8],
            ) -> impl core::future::Future<
                Output = Result<
                    AssociatedCheckedResponse<'buffer, operations::$marker>,
                    PermitExecutionError<T::Error>,
                >,
            > + Send
                   + 'transport
            where
                C: PermitClock + Sync + ?Sized,
                'permit: 'transport,
                'request: 'transport,
                'fingerprint: 'transport,
                'buffer: 'transport,
            {
                attempt.execute_async(clock, self.transport(), body, headers)
            }
        }

        impl<T> HetznerClient<T, CloudService, OfficialEndpointTrust>
        where
            T: LocalAsyncAuthenticatedTransport + BoundTransport,
            T::Error: DeliveryClassified,
        {
            #[doc = concat!("Executes an authorized `", stringify!($marker), "` attempt locally.")]
            #[allow(clippy::manual_async_fn)]
            pub fn $local<'transport, 'permit, 'request, 'fingerprint, 'buffer, C>(
                &'transport self,
                attempt: AssociatedPermitAttempt<
                    'permit,
                    'request,
                    'fingerprint,
                    operations::$marker,
                >,
                clock: &'transport C,
                body: &'buffer mut [u8],
                headers: &'buffer mut [u8],
            ) -> impl core::future::Future<
                Output = Result<
                    AssociatedCheckedResponse<'buffer, operations::$marker>,
                    PermitExecutionError<T::Error>,
                >,
            > + 'transport
            where
                C: PermitClock + ?Sized,
                'permit: 'transport,
                'request: 'transport,
                'fingerprint: 'transport,
                'buffer: 'transport,
            {
                attempt.execute_local_async(clock, self.transport(), body, headers)
            }
        }
    };
}

macro_rules! cloud_client_method {
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, none) => {
        read_method!($marker, $blocking, $asynchronous, $local);
    };
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, $permit:ident) => {
        permitted_method!($marker, $prepare, $blocking, $asynchronous, $local);
    };
}

macro_rules! cloud_client_methods {
    ($(($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, $permit:ident),)+) => {
        $(cloud_client_method!($marker, $prepare, $blocking, $asynchronous, $local, $permit);)+

        /// Exhaustive source-locked Cloud operation client surface.
        pub const CLOUD_CLIENT_METHODS: &[CloudClientMethodDescriptor] = &[
            $(CloudClientMethodDescriptor::new(<operations::$marker as HetznerOperation>::DESCRIPTOR),)+
        ];
    };
}

#[rustfmt::skip]
cloud_client_methods!(
    (AddLoadBalancerService, prepare_add_load_balancer_service, add_load_balancer_service_blocking, add_load_balancer_service_async, add_load_balancer_service_local_async, mutation),
    (AddLoadBalancerTarget, prepare_add_load_balancer_target, add_load_balancer_target_blocking, add_load_balancer_target_async, add_load_balancer_target_local_async, mutation),
    (AddNetworkRoute, prepare_add_network_route, add_network_route_blocking, add_network_route_async, add_network_route_local_async, mutation),
    (AddNetworkSubnet, prepare_add_network_subnet, add_network_subnet_blocking, add_network_subnet_async, add_network_subnet_local_async, mutation),
    (AddServerToPlacementGroup, prepare_add_server_to_placement_group, add_server_to_placement_group_blocking, add_server_to_placement_group_async, add_server_to_placement_group_local_async, mutation),
    (ApplyFirewallToResources, prepare_apply_firewall_to_resources, apply_firewall_to_resources_blocking, apply_firewall_to_resources_async, apply_firewall_to_resources_local_async, mutation),
    (AssignFloatingIp, prepare_assign_floating_ip, assign_floating_ip_blocking, assign_floating_ip_async, assign_floating_ip_local_async, mutation),
    (AssignPrimaryIp, prepare_assign_primary_ip, assign_primary_ip_blocking, assign_primary_ip_async, assign_primary_ip_local_async, mutation),
    (AttachLoadBalancerToNetwork, prepare_attach_load_balancer_to_network, attach_load_balancer_to_network_blocking, attach_load_balancer_to_network_async, attach_load_balancer_to_network_local_async, mutation),
    (AttachServerIso, prepare_attach_server_iso, attach_server_iso_blocking, attach_server_iso_async, attach_server_iso_local_async, mutation),
    (AttachServerToNetwork, prepare_attach_server_to_network, attach_server_to_network_blocking, attach_server_to_network_async, attach_server_to_network_local_async, mutation),
    (AttachVolume, prepare_attach_volume, attach_volume_blocking, attach_volume_async, attach_volume_local_async, mutation),
    (ChangeFloatingIpDnsPtr, prepare_change_floating_ip_dns_ptr, change_floating_ip_dns_ptr_blocking, change_floating_ip_dns_ptr_async, change_floating_ip_dns_ptr_local_async, mutation),
    (ChangeFloatingIpProtection, prepare_change_floating_ip_protection, change_floating_ip_protection_blocking, change_floating_ip_protection_async, change_floating_ip_protection_local_async, destructive),
    (ChangeImageProtection, prepare_change_image_protection, change_image_protection_blocking, change_image_protection_async, change_image_protection_local_async, destructive),
    (ChangeLoadBalancerAlgorithm, prepare_change_load_balancer_algorithm, change_load_balancer_algorithm_blocking, change_load_balancer_algorithm_async, change_load_balancer_algorithm_local_async, mutation),
    (ChangeLoadBalancerDnsPtr, prepare_change_load_balancer_dns_ptr, change_load_balancer_dns_ptr_blocking, change_load_balancer_dns_ptr_async, change_load_balancer_dns_ptr_local_async, mutation),
    (ChangeLoadBalancerProtection, prepare_change_load_balancer_protection, change_load_balancer_protection_blocking, change_load_balancer_protection_async, change_load_balancer_protection_local_async, destructive),
    (ChangeLoadBalancerType, prepare_change_load_balancer_type, change_load_balancer_type_blocking, change_load_balancer_type_async, change_load_balancer_type_local_async, cost),
    (ChangeNetworkIpRange, prepare_change_network_ip_range, change_network_ip_range_blocking, change_network_ip_range_async, change_network_ip_range_local_async, mutation),
    (ChangeNetworkProtection, prepare_change_network_protection, change_network_protection_blocking, change_network_protection_async, change_network_protection_local_async, destructive),
    (ChangePrimaryIpDnsPtr, prepare_change_primary_ip_dns_ptr, change_primary_ip_dns_ptr_blocking, change_primary_ip_dns_ptr_async, change_primary_ip_dns_ptr_local_async, mutation),
    (ChangePrimaryIpProtection, prepare_change_primary_ip_protection, change_primary_ip_protection_blocking, change_primary_ip_protection_async, change_primary_ip_protection_local_async, destructive),
    (ChangeServerAliasIps, prepare_change_server_alias_ips, change_server_alias_ips_blocking, change_server_alias_ips_async, change_server_alias_ips_local_async, mutation),
    (ChangeServerDnsPtr, prepare_change_server_dns_ptr, change_server_dns_ptr_blocking, change_server_dns_ptr_async, change_server_dns_ptr_local_async, mutation),
    (ChangeServerProtection, prepare_change_server_protection, change_server_protection_blocking, change_server_protection_async, change_server_protection_local_async, destructive),
    (ChangeServerType, prepare_change_server_type, change_server_type_blocking, change_server_type_async, change_server_type_local_async, cost),
    (ChangeVolumeProtection, prepare_change_volume_protection, change_volume_protection_blocking, change_volume_protection_async, change_volume_protection_local_async, destructive),
    (CreateFirewall, prepare_create_firewall, create_firewall_blocking, create_firewall_async, create_firewall_local_async, mutation),
    (CreateFloatingIp, prepare_create_floating_ip, create_floating_ip_blocking, create_floating_ip_async, create_floating_ip_local_async, cost),
    (CreateLoadBalancer, prepare_create_load_balancer, create_load_balancer_blocking, create_load_balancer_async, create_load_balancer_local_async, cost),
    (CreateNetwork, prepare_create_network, create_network_blocking, create_network_async, create_network_local_async, mutation),
    (CreatePlacementGroup, prepare_create_placement_group, create_placement_group_blocking, create_placement_group_async, create_placement_group_local_async, mutation),
    (CreatePrimaryIp, prepare_create_primary_ip, create_primary_ip_blocking, create_primary_ip_async, create_primary_ip_local_async, cost),
    (CreateServer, prepare_create_server, create_server_blocking, create_server_async, create_server_local_async, cost),
    (CreateServerImage, prepare_create_server_image, create_server_image_blocking, create_server_image_async, create_server_image_local_async, cost),
    (CreateVolume, prepare_create_volume, create_volume_blocking, create_volume_async, create_volume_local_async, cost),
    (DeleteFirewall, prepare_delete_firewall, delete_firewall_blocking, delete_firewall_async, delete_firewall_local_async, destructive),
    (DeleteFloatingIp, prepare_delete_floating_ip, delete_floating_ip_blocking, delete_floating_ip_async, delete_floating_ip_local_async, destructive),
    (DeleteImage, prepare_delete_image, delete_image_blocking, delete_image_async, delete_image_local_async, destructive),
    (DeleteLoadBalancer, prepare_delete_load_balancer, delete_load_balancer_blocking, delete_load_balancer_async, delete_load_balancer_local_async, destructive),
    (DeleteLoadBalancerService, prepare_delete_load_balancer_service, delete_load_balancer_service_blocking, delete_load_balancer_service_async, delete_load_balancer_service_local_async, destructive),
    (DeleteNetwork, prepare_delete_network, delete_network_blocking, delete_network_async, delete_network_local_async, destructive),
    (DeleteNetworkRoute, prepare_delete_network_route, delete_network_route_blocking, delete_network_route_async, delete_network_route_local_async, destructive),
    (DeleteNetworkSubnet, prepare_delete_network_subnet, delete_network_subnet_blocking, delete_network_subnet_async, delete_network_subnet_local_async, destructive),
    (DeletePlacementGroup, prepare_delete_placement_group, delete_placement_group_blocking, delete_placement_group_async, delete_placement_group_local_async, destructive),
    (DeletePrimaryIp, prepare_delete_primary_ip, delete_primary_ip_blocking, delete_primary_ip_async, delete_primary_ip_local_async, destructive),
    (DeleteServer, prepare_delete_server, delete_server_blocking, delete_server_async, delete_server_local_async, destructive),
    (DeleteVolume, prepare_delete_volume, delete_volume_blocking, delete_volume_async, delete_volume_local_async, destructive),
    (DetachLoadBalancerFromNetwork, prepare_detach_load_balancer_from_network, detach_load_balancer_from_network_blocking, detach_load_balancer_from_network_async, detach_load_balancer_from_network_local_async, destructive),
    (DetachServerFromNetwork, prepare_detach_server_from_network, detach_server_from_network_blocking, detach_server_from_network_async, detach_server_from_network_local_async, destructive),
    (DetachServerIso, prepare_detach_server_iso, detach_server_iso_blocking, detach_server_iso_async, detach_server_iso_local_async, destructive),
    (DetachVolume, prepare_detach_volume, detach_volume_blocking, detach_volume_async, detach_volume_local_async, destructive),
    (DisableLoadBalancerPublicInterface, prepare_disable_load_balancer_public_interface, disable_load_balancer_public_interface_blocking, disable_load_balancer_public_interface_async, disable_load_balancer_public_interface_local_async, destructive),
    (DisableServerBackup, prepare_disable_server_backup, disable_server_backup_blocking, disable_server_backup_async, disable_server_backup_local_async, destructive),
    (DisableServerRescue, prepare_disable_server_rescue, disable_server_rescue_blocking, disable_server_rescue_async, disable_server_rescue_local_async, destructive),
    (EnableLoadBalancerPublicInterface, prepare_enable_load_balancer_public_interface, enable_load_balancer_public_interface_blocking, enable_load_balancer_public_interface_async, enable_load_balancer_public_interface_local_async, mutation),
    (EnableServerBackup, prepare_enable_server_backup, enable_server_backup_blocking, enable_server_backup_async, enable_server_backup_local_async, cost),
    (EnableServerRescue, prepare_enable_server_rescue, enable_server_rescue_blocking, enable_server_rescue_async, enable_server_rescue_local_async, mutation),
    (GetAction, prepare_get_action, get_action_blocking, get_action_async, get_action_local_async, none),
    (GetActions, prepare_get_actions, get_actions_blocking, get_actions_async, get_actions_local_async, none),
    (GetFirewall, prepare_get_firewall, get_firewall_blocking, get_firewall_async, get_firewall_local_async, none),
    (GetFirewallsAction, prepare_get_firewalls_action, get_firewalls_action_blocking, get_firewalls_action_async, get_firewalls_action_local_async, none),
    (GetFloatingIp, prepare_get_floating_ip, get_floating_ip_blocking, get_floating_ip_async, get_floating_ip_local_async, none),
    (GetFloatingIpsAction, prepare_get_floating_ips_action, get_floating_ips_action_blocking, get_floating_ips_action_async, get_floating_ips_action_local_async, none),
    (GetImage, prepare_get_image, get_image_blocking, get_image_async, get_image_local_async, none),
    (GetImagesAction, prepare_get_images_action, get_images_action_blocking, get_images_action_async, get_images_action_local_async, none),
    (GetIso, prepare_get_iso, get_iso_blocking, get_iso_async, get_iso_local_async, none),
    (GetLoadBalancer, prepare_get_load_balancer, get_load_balancer_blocking, get_load_balancer_async, get_load_balancer_local_async, none),
    (GetLoadBalancerMetrics, prepare_get_load_balancer_metrics, get_load_balancer_metrics_blocking, get_load_balancer_metrics_async, get_load_balancer_metrics_local_async, none),
    (GetLoadBalancerType, prepare_get_load_balancer_type, get_load_balancer_type_blocking, get_load_balancer_type_async, get_load_balancer_type_local_async, none),
    (GetLoadBalancersAction, prepare_get_load_balancers_action, get_load_balancers_action_blocking, get_load_balancers_action_async, get_load_balancers_action_local_async, none),
    (GetLocation, prepare_get_location, get_location_blocking, get_location_async, get_location_local_async, none),
    (GetNetwork, prepare_get_network, get_network_blocking, get_network_async, get_network_local_async, none),
    (GetNetworksAction, prepare_get_networks_action, get_networks_action_blocking, get_networks_action_async, get_networks_action_local_async, none),
    (GetPlacementGroup, prepare_get_placement_group, get_placement_group_blocking, get_placement_group_async, get_placement_group_local_async, none),
    (GetPricing, prepare_get_pricing, get_pricing_blocking, get_pricing_async, get_pricing_local_async, none),
    (GetPrimaryIp, prepare_get_primary_ip, get_primary_ip_blocking, get_primary_ip_async, get_primary_ip_local_async, none),
    (GetPrimaryIpsAction, prepare_get_primary_ips_action, get_primary_ips_action_blocking, get_primary_ips_action_async, get_primary_ips_action_local_async, none),
    (GetServer, prepare_get_server, get_server_blocking, get_server_async, get_server_local_async, none),
    (GetServerMetrics, prepare_get_server_metrics, get_server_metrics_blocking, get_server_metrics_async, get_server_metrics_local_async, none),
    (GetServerType, prepare_get_server_type, get_server_type_blocking, get_server_type_async, get_server_type_local_async, none),
    (GetServersAction, prepare_get_servers_action, get_servers_action_blocking, get_servers_action_async, get_servers_action_local_async, none),
    (GetVolume, prepare_get_volume, get_volume_blocking, get_volume_async, get_volume_local_async, none),
    (GetVolumesAction, prepare_get_volumes_action, get_volumes_action_blocking, get_volumes_action_async, get_volumes_action_local_async, none),
    (ListFirewallActions, prepare_list_firewall_actions, list_firewall_actions_blocking, list_firewall_actions_async, list_firewall_actions_local_async, none),
    (ListFirewalls, prepare_list_firewalls, list_firewalls_blocking, list_firewalls_async, list_firewalls_local_async, none),
    (ListFirewallsActions, prepare_list_firewalls_actions, list_firewalls_actions_blocking, list_firewalls_actions_async, list_firewalls_actions_local_async, none),
    (ListFloatingIpActions, prepare_list_floating_ip_actions, list_floating_ip_actions_blocking, list_floating_ip_actions_async, list_floating_ip_actions_local_async, none),
    (ListFloatingIps, prepare_list_floating_ips, list_floating_ips_blocking, list_floating_ips_async, list_floating_ips_local_async, none),
    (ListFloatingIpsActions, prepare_list_floating_ips_actions, list_floating_ips_actions_blocking, list_floating_ips_actions_async, list_floating_ips_actions_local_async, none),
    (ListImageActions, prepare_list_image_actions, list_image_actions_blocking, list_image_actions_async, list_image_actions_local_async, none),
    (ListImages, prepare_list_images, list_images_blocking, list_images_async, list_images_local_async, none),
    (ListImagesActions, prepare_list_images_actions, list_images_actions_blocking, list_images_actions_async, list_images_actions_local_async, none),
    (ListIsos, prepare_list_isos, list_isos_blocking, list_isos_async, list_isos_local_async, none),
    (ListLoadBalancerActions, prepare_list_load_balancer_actions, list_load_balancer_actions_blocking, list_load_balancer_actions_async, list_load_balancer_actions_local_async, none),
    (ListLoadBalancerTypes, prepare_list_load_balancer_types, list_load_balancer_types_blocking, list_load_balancer_types_async, list_load_balancer_types_local_async, none),
    (ListLoadBalancers, prepare_list_load_balancers, list_load_balancers_blocking, list_load_balancers_async, list_load_balancers_local_async, none),
    (ListLoadBalancersActions, prepare_list_load_balancers_actions, list_load_balancers_actions_blocking, list_load_balancers_actions_async, list_load_balancers_actions_local_async, none),
    (ListLocations, prepare_list_locations, list_locations_blocking, list_locations_async, list_locations_local_async, none),
    (ListNetworkActions, prepare_list_network_actions, list_network_actions_blocking, list_network_actions_async, list_network_actions_local_async, none),
    (ListNetworks, prepare_list_networks, list_networks_blocking, list_networks_async, list_networks_local_async, none),
    (ListNetworksActions, prepare_list_networks_actions, list_networks_actions_blocking, list_networks_actions_async, list_networks_actions_local_async, none),
    (ListPlacementGroups, prepare_list_placement_groups, list_placement_groups_blocking, list_placement_groups_async, list_placement_groups_local_async, none),
    (ListPrimaryIpActions, prepare_list_primary_ip_actions, list_primary_ip_actions_blocking, list_primary_ip_actions_async, list_primary_ip_actions_local_async, none),
    (ListPrimaryIps, prepare_list_primary_ips, list_primary_ips_blocking, list_primary_ips_async, list_primary_ips_local_async, none),
    (ListPrimaryIpsActions, prepare_list_primary_ips_actions, list_primary_ips_actions_blocking, list_primary_ips_actions_async, list_primary_ips_actions_local_async, none),
    (ListServerActions, prepare_list_server_actions, list_server_actions_blocking, list_server_actions_async, list_server_actions_local_async, none),
    (ListServerTypes, prepare_list_server_types, list_server_types_blocking, list_server_types_async, list_server_types_local_async, none),
    (ListServers, prepare_list_servers, list_servers_blocking, list_servers_async, list_servers_local_async, none),
    (ListServersActions, prepare_list_servers_actions, list_servers_actions_blocking, list_servers_actions_async, list_servers_actions_local_async, none),
    (ListVolumeActions, prepare_list_volume_actions, list_volume_actions_blocking, list_volume_actions_async, list_volume_actions_local_async, none),
    (ListVolumes, prepare_list_volumes, list_volumes_blocking, list_volumes_async, list_volumes_local_async, none),
    (ListVolumesActions, prepare_list_volumes_actions, list_volumes_actions_blocking, list_volumes_actions_async, list_volumes_actions_local_async, none),
    (PoweroffServer, prepare_poweroff_server, poweroff_server_blocking, poweroff_server_async, poweroff_server_local_async, destructive),
    (PoweronServer, prepare_poweron_server, poweron_server_blocking, poweron_server_async, poweron_server_local_async, mutation),
    (RebootServer, prepare_reboot_server, reboot_server_blocking, reboot_server_async, reboot_server_local_async, mutation),
    (RebuildServer, prepare_rebuild_server, rebuild_server_blocking, rebuild_server_async, rebuild_server_local_async, destructive),
    (RemoveFirewallFromResources, prepare_remove_firewall_from_resources, remove_firewall_from_resources_blocking, remove_firewall_from_resources_async, remove_firewall_from_resources_local_async, destructive),
    (RemoveLoadBalancerTarget, prepare_remove_load_balancer_target, remove_load_balancer_target_blocking, remove_load_balancer_target_async, remove_load_balancer_target_local_async, destructive),
    (RemoveServerFromPlacementGroup, prepare_remove_server_from_placement_group, remove_server_from_placement_group_blocking, remove_server_from_placement_group_async, remove_server_from_placement_group_local_async, destructive),
    (RequestServerConsole, prepare_request_server_console, request_server_console_blocking, request_server_console_async, request_server_console_local_async, mutation),
    (ResetServer, prepare_reset_server, reset_server_blocking, reset_server_async, reset_server_local_async, destructive),
    (ResetServerPassword, prepare_reset_server_password, reset_server_password_blocking, reset_server_password_async, reset_server_password_local_async, destructive),
    (ResizeVolume, prepare_resize_volume, resize_volume_blocking, resize_volume_async, resize_volume_local_async, cost),
    (SetFirewallRules, prepare_set_firewall_rules, set_firewall_rules_blocking, set_firewall_rules_async, set_firewall_rules_local_async, destructive),
    (ShutdownServer, prepare_shutdown_server, shutdown_server_blocking, shutdown_server_async, shutdown_server_local_async, destructive),
    (UnassignFloatingIp, prepare_unassign_floating_ip, unassign_floating_ip_blocking, unassign_floating_ip_async, unassign_floating_ip_local_async, destructive),
    (UnassignPrimaryIp, prepare_unassign_primary_ip, unassign_primary_ip_blocking, unassign_primary_ip_async, unassign_primary_ip_local_async, destructive),
    (UpdateFirewall, prepare_update_firewall, update_firewall_blocking, update_firewall_async, update_firewall_local_async, mutation),
    (UpdateFloatingIp, prepare_update_floating_ip, update_floating_ip_blocking, update_floating_ip_async, update_floating_ip_local_async, mutation),
    (UpdateImage, prepare_update_image, update_image_blocking, update_image_async, update_image_local_async, mutation),
    (UpdateLoadBalancer, prepare_update_load_balancer, update_load_balancer_blocking, update_load_balancer_async, update_load_balancer_local_async, mutation),
    (UpdateLoadBalancerService, prepare_update_load_balancer_service, update_load_balancer_service_blocking, update_load_balancer_service_async, update_load_balancer_service_local_async, mutation),
    (UpdateNetwork, prepare_update_network, update_network_blocking, update_network_async, update_network_local_async, mutation),
    (UpdatePlacementGroup, prepare_update_placement_group, update_placement_group_blocking, update_placement_group_async, update_placement_group_local_async, mutation),
    (UpdatePrimaryIp, prepare_update_primary_ip, update_primary_ip_blocking, update_primary_ip_async, update_primary_ip_local_async, mutation),
    (UpdateServer, prepare_update_server, update_server_blocking, update_server_async, update_server_local_async, mutation),
    (UpdateVolume, prepare_update_volume, update_volume_blocking, update_volume_async, update_volume_local_async, mutation),
);
