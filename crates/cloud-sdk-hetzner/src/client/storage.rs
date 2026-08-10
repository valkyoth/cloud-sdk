//! Generated exhaustive methods for the official Hetzner Storage client.
//!
//! Regenerate with `scripts/generate_storage_client_methods.py`.

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
use crate::identity::StorageService;
use crate::prepared::{BodyWire, EndpointWire, QueryWire};
use crate::serde::{CheckedHetznerResponse, HetznerDecodeError};

/// Result returned by a complete read-only Storage client method.
pub type StorageReadResult<E> = Result<
    CheckedHetznerResponse,
    ClientExecutionError<AssociatedPreparationError, E, HetznerDecodeError>,
>;

/// Source-locked Storage operation exposed by the service-typed client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageClientMethodDescriptor {
    operation: OperationDescriptor,
}

impl StorageClientMethodDescriptor {
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
        impl<T> HetznerClient<T, StorageService, OfficialEndpointTrust>
        where
            T: BlockingAuthenticatedTransport + BoundTransport,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` synchronously.")]
            pub fn $blocking<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> StorageReadResult<T::Error>
            where
                E: EndpointWire,
                Q: QueryWire,
                B: BodyWire,
            {
                self.execute_blocking(operation, lease)
            }
        }

        impl<T> HetznerClient<T, StorageService, OfficialEndpointTrust>
        where
            T: AsyncAuthenticatedTransport + BoundTransport + Sync,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` through a `Send` future.")]
            #[allow(clippy::manual_async_fn)]
            pub fn $asynchronous<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> impl core::future::Future<Output = StorageReadResult<T::Error>> + Send
            where
                E: EndpointWire + Sync,
                Q: QueryWire + Sync,
                B: BodyWire + Sync,
                T::Error: Send,
            {
                self.execute_async(operation, lease)
            }
        }

        impl<T> HetznerClient<T, StorageService, OfficialEndpointTrust>
        where
            T: LocalAsyncAuthenticatedTransport + BoundTransport,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` on a local executor.")]
            pub async fn $local<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> StorageReadResult<T::Error>
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
        impl<T> HetznerClient<T, StorageService, OfficialEndpointTrust> {
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

        impl<T> HetznerClient<T, StorageService, OfficialEndpointTrust>
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

        impl<T> HetznerClient<T, StorageService, OfficialEndpointTrust>
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

        impl<T> HetznerClient<T, StorageService, OfficialEndpointTrust>
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

macro_rules! storage_client_method {
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, none) => {
        read_method!($marker, $blocking, $asynchronous, $local);
    };
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, $permit:ident) => {
        permitted_method!($marker, $prepare, $blocking, $asynchronous, $local);
    };
}

macro_rules! storage_client_methods {
    ($(($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, $permit:ident),)+) => {
        $(storage_client_method!($marker, $prepare, $blocking, $asynchronous, $local, $permit);)+

        /// Exhaustive source-locked Storage operation client surface.
        pub const STORAGE_CLIENT_METHODS: &[StorageClientMethodDescriptor] = &[
            $(StorageClientMethodDescriptor::new(<operations::$marker as HetznerOperation>::DESCRIPTOR),)+
        ];
    };
}

#[rustfmt::skip]
storage_client_methods!(
    (ChangeStorageBoxProtection, prepare_change_storage_box_protection, change_storage_box_protection_blocking, change_storage_box_protection_async, change_storage_box_protection_local_async, destructive),
    (ChangeStorageBoxSubaccountHomeDirectory, prepare_change_storage_box_subaccount_home_directory, change_storage_box_subaccount_home_directory_blocking, change_storage_box_subaccount_home_directory_async, change_storage_box_subaccount_home_directory_local_async, mutation),
    (ChangeStorageBoxType, prepare_change_storage_box_type, change_storage_box_type_blocking, change_storage_box_type_async, change_storage_box_type_local_async, cost),
    (CreateStorageBox, prepare_create_storage_box, create_storage_box_blocking, create_storage_box_async, create_storage_box_local_async, cost),
    (CreateStorageBoxSnapshot, prepare_create_storage_box_snapshot, create_storage_box_snapshot_blocking, create_storage_box_snapshot_async, create_storage_box_snapshot_local_async, mutation),
    (CreateStorageBoxSubaccount, prepare_create_storage_box_subaccount, create_storage_box_subaccount_blocking, create_storage_box_subaccount_async, create_storage_box_subaccount_local_async, mutation),
    (DeleteStorageBox, prepare_delete_storage_box, delete_storage_box_blocking, delete_storage_box_async, delete_storage_box_local_async, destructive),
    (DeleteStorageBoxSnapshot, prepare_delete_storage_box_snapshot, delete_storage_box_snapshot_blocking, delete_storage_box_snapshot_async, delete_storage_box_snapshot_local_async, destructive),
    (DeleteStorageBoxSubaccount, prepare_delete_storage_box_subaccount, delete_storage_box_subaccount_blocking, delete_storage_box_subaccount_async, delete_storage_box_subaccount_local_async, destructive),
    (DisableStorageBoxSnapshotPlan, prepare_disable_storage_box_snapshot_plan, disable_storage_box_snapshot_plan_blocking, disable_storage_box_snapshot_plan_async, disable_storage_box_snapshot_plan_local_async, destructive),
    (EnableStorageBoxSnapshotPlan, prepare_enable_storage_box_snapshot_plan, enable_storage_box_snapshot_plan_blocking, enable_storage_box_snapshot_plan_async, enable_storage_box_snapshot_plan_local_async, mutation),
    (GetStorageBox, prepare_get_storage_box, get_storage_box_blocking, get_storage_box_async, get_storage_box_local_async, none),
    (GetStorageBoxSnapshot, prepare_get_storage_box_snapshot, get_storage_box_snapshot_blocking, get_storage_box_snapshot_async, get_storage_box_snapshot_local_async, none),
    (GetStorageBoxSubaccount, prepare_get_storage_box_subaccount, get_storage_box_subaccount_blocking, get_storage_box_subaccount_async, get_storage_box_subaccount_local_async, none),
    (GetStorageBoxType, prepare_get_storage_box_type, get_storage_box_type_blocking, get_storage_box_type_async, get_storage_box_type_local_async, none),
    (GetStorageBoxesAction, prepare_get_storage_boxes_action, get_storage_boxes_action_blocking, get_storage_boxes_action_async, get_storage_boxes_action_local_async, none),
    (ListStorageBoxActions, prepare_list_storage_box_actions, list_storage_box_actions_blocking, list_storage_box_actions_async, list_storage_box_actions_local_async, none),
    (ListStorageBoxFolders, prepare_list_storage_box_folders, list_storage_box_folders_blocking, list_storage_box_folders_async, list_storage_box_folders_local_async, none),
    (ListStorageBoxSnapshots, prepare_list_storage_box_snapshots, list_storage_box_snapshots_blocking, list_storage_box_snapshots_async, list_storage_box_snapshots_local_async, none),
    (ListStorageBoxSubaccounts, prepare_list_storage_box_subaccounts, list_storage_box_subaccounts_blocking, list_storage_box_subaccounts_async, list_storage_box_subaccounts_local_async, none),
    (ListStorageBoxTypes, prepare_list_storage_box_types, list_storage_box_types_blocking, list_storage_box_types_async, list_storage_box_types_local_async, none),
    (ListStorageBoxes, prepare_list_storage_boxes, list_storage_boxes_blocking, list_storage_boxes_async, list_storage_boxes_local_async, none),
    (ListStorageBoxesActions, prepare_list_storage_boxes_actions, list_storage_boxes_actions_blocking, list_storage_boxes_actions_async, list_storage_boxes_actions_local_async, none),
    (ResetStorageBoxPassword, prepare_reset_storage_box_password, reset_storage_box_password_blocking, reset_storage_box_password_async, reset_storage_box_password_local_async, destructive),
    (ResetStorageBoxSubaccountPassword, prepare_reset_storage_box_subaccount_password, reset_storage_box_subaccount_password_blocking, reset_storage_box_subaccount_password_async, reset_storage_box_subaccount_password_local_async, destructive),
    (RollbackStorageBoxSnapshot, prepare_rollback_storage_box_snapshot, rollback_storage_box_snapshot_blocking, rollback_storage_box_snapshot_async, rollback_storage_box_snapshot_local_async, destructive),
    (UpdateStorageBox, prepare_update_storage_box, update_storage_box_blocking, update_storage_box_async, update_storage_box_local_async, mutation),
    (UpdateStorageBoxAccessSettings, prepare_update_storage_box_access_settings, update_storage_box_access_settings_blocking, update_storage_box_access_settings_async, update_storage_box_access_settings_local_async, mutation),
    (UpdateStorageBoxSnapshot, prepare_update_storage_box_snapshot, update_storage_box_snapshot_blocking, update_storage_box_snapshot_async, update_storage_box_snapshot_local_async, mutation),
    (UpdateStorageBoxSubaccount, prepare_update_storage_box_subaccount, update_storage_box_subaccount_blocking, update_storage_box_subaccount_async, update_storage_box_subaccount_local_async, mutation),
    (UpdateStorageBoxSubaccountAccessSettings, prepare_update_storage_box_subaccount_access_settings, update_storage_box_subaccount_access_settings_blocking, update_storage_box_subaccount_access_settings_async, update_storage_box_subaccount_access_settings_local_async, mutation),
);
