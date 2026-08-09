//! Generated exhaustive methods for the official Hetzner DNS client.
//!
//! Regenerate with `scripts/generate_dns_client_methods.py`.

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
use crate::identity::DnsService;
use crate::prepared::{BodyWire, EndpointWire, QueryWire};
use crate::serde::{CheckedHetznerResponse, HetznerDecodeError};

/// Result returned by a complete read-only DNS client method.
pub type DnsReadResult<E> = Result<
    CheckedHetznerResponse,
    ClientExecutionError<AssociatedPreparationError, E, HetznerDecodeError>,
>;

/// Source-locked DNS operation exposed by the service-typed client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DnsClientMethodDescriptor {
    operation: OperationDescriptor,
}

impl DnsClientMethodDescriptor {
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
        impl<T> HetznerClient<T, DnsService, OfficialEndpointTrust>
        where
            T: BlockingAuthenticatedTransport + BoundTransport,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` synchronously.")]
            pub fn $blocking<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> DnsReadResult<T::Error>
            where
                E: EndpointWire,
                Q: QueryWire,
                B: BodyWire,
            {
                self.execute_blocking(operation, lease)
            }
        }

        impl<T> HetznerClient<T, DnsService, OfficialEndpointTrust>
        where
            T: AsyncAuthenticatedTransport + BoundTransport + Sync,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` through a `Send` future.")]
            #[allow(clippy::manual_async_fn)]
            pub fn $asynchronous<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> impl core::future::Future<Output = DnsReadResult<T::Error>> + Send
            where
                E: EndpointWire + Sync,
                Q: QueryWire + Sync,
                B: BodyWire + Sync,
                T::Error: Send,
            {
                self.execute_async(operation, lease)
            }
        }

        impl<T> HetznerClient<T, DnsService, OfficialEndpointTrust>
        where
            T: LocalAsyncAuthenticatedTransport + BoundTransport,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` on a local executor.")]
            pub async fn $local<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> DnsReadResult<T::Error>
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
        impl<T> HetznerClient<T, DnsService, OfficialEndpointTrust> {
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

        impl<T> HetznerClient<T, DnsService, OfficialEndpointTrust>
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

        impl<T> HetznerClient<T, DnsService, OfficialEndpointTrust>
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

        impl<T> HetznerClient<T, DnsService, OfficialEndpointTrust>
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

macro_rules! dns_client_method {
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, none) => {
        read_method!($marker, $blocking, $asynchronous, $local);
    };
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, $permit:ident) => {
        permitted_method!($marker, $prepare, $blocking, $asynchronous, $local);
    };
}

macro_rules! dns_client_methods {
    ($(($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, $permit:ident),)+) => {
        $(dns_client_method!($marker, $prepare, $blocking, $asynchronous, $local, $permit);)+

        /// Exhaustive source-locked DNS operation client surface.
        pub const DNS_CLIENT_METHODS: &[DnsClientMethodDescriptor] = &[
            $(DnsClientMethodDescriptor::new(<operations::$marker as HetznerOperation>::DESCRIPTOR),)+
        ];
    };
}

#[rustfmt::skip]
dns_client_methods!(
    (AddZoneRrsetRecords, prepare_add_zone_rrset_records, add_zone_rrset_records_blocking, add_zone_rrset_records_async, add_zone_rrset_records_local_async, mutation),
    (ChangeZonePrimaryNameservers, prepare_change_zone_primary_nameservers, change_zone_primary_nameservers_blocking, change_zone_primary_nameservers_async, change_zone_primary_nameservers_local_async, mutation),
    (ChangeZoneProtection, prepare_change_zone_protection, change_zone_protection_blocking, change_zone_protection_async, change_zone_protection_local_async, destructive),
    (ChangeZoneRrsetProtection, prepare_change_zone_rrset_protection, change_zone_rrset_protection_blocking, change_zone_rrset_protection_async, change_zone_rrset_protection_local_async, destructive),
    (ChangeZoneRrsetTtl, prepare_change_zone_rrset_ttl, change_zone_rrset_ttl_blocking, change_zone_rrset_ttl_async, change_zone_rrset_ttl_local_async, mutation),
    (ChangeZoneTtl, prepare_change_zone_ttl, change_zone_ttl_blocking, change_zone_ttl_async, change_zone_ttl_local_async, mutation),
    (CreateZone, prepare_create_zone, create_zone_blocking, create_zone_async, create_zone_local_async, mutation),
    (CreateZoneRrset, prepare_create_zone_rrset, create_zone_rrset_blocking, create_zone_rrset_async, create_zone_rrset_local_async, mutation),
    (DeleteZone, prepare_delete_zone, delete_zone_blocking, delete_zone_async, delete_zone_local_async, destructive),
    (DeleteZoneRrset, prepare_delete_zone_rrset, delete_zone_rrset_blocking, delete_zone_rrset_async, delete_zone_rrset_local_async, destructive),
    (GetZone, prepare_get_zone, get_zone_blocking, get_zone_async, get_zone_local_async, none),
    (GetZoneRrset, prepare_get_zone_rrset, get_zone_rrset_blocking, get_zone_rrset_async, get_zone_rrset_local_async, none),
    (GetZoneZonefile, prepare_get_zone_zonefile, get_zone_zonefile_blocking, get_zone_zonefile_async, get_zone_zonefile_local_async, none),
    (GetZonesAction, prepare_get_zones_action, get_zones_action_blocking, get_zones_action_async, get_zones_action_local_async, none),
    (ImportZoneZonefile, prepare_import_zone_zonefile, import_zone_zonefile_blocking, import_zone_zonefile_async, import_zone_zonefile_local_async, destructive),
    (ListZoneActions, prepare_list_zone_actions, list_zone_actions_blocking, list_zone_actions_async, list_zone_actions_local_async, none),
    (ListZoneRrsets, prepare_list_zone_rrsets, list_zone_rrsets_blocking, list_zone_rrsets_async, list_zone_rrsets_local_async, none),
    (ListZones, prepare_list_zones, list_zones_blocking, list_zones_async, list_zones_local_async, none),
    (ListZonesActions, prepare_list_zones_actions, list_zones_actions_blocking, list_zones_actions_async, list_zones_actions_local_async, none),
    (RemoveZoneRrsetRecords, prepare_remove_zone_rrset_records, remove_zone_rrset_records_blocking, remove_zone_rrset_records_async, remove_zone_rrset_records_local_async, destructive),
    (SetZoneRrsetRecords, prepare_set_zone_rrset_records, set_zone_rrset_records_blocking, set_zone_rrset_records_async, set_zone_rrset_records_local_async, destructive),
    (UpdateZone, prepare_update_zone, update_zone_blocking, update_zone_async, update_zone_local_async, mutation),
    (UpdateZoneRrset, prepare_update_zone_rrset, update_zone_rrset_blocking, update_zone_rrset_async, update_zone_rrset_local_async, mutation),
    (UpdateZoneRrsetRecords, prepare_update_zone_rrset_records, update_zone_rrset_records_blocking, update_zone_rrset_records_async, update_zone_rrset_records_local_async, mutation),
);
