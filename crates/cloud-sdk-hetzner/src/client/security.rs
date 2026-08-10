//! Generated exhaustive methods for the official Hetzner Security client.
//!
//! Regenerate with `scripts/generate_security_client_methods.py`.

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
use crate::identity::SecurityService;
use crate::prepared::{BodyWire, EndpointWire, QueryWire};
use crate::serde::{CheckedHetznerResponse, HetznerDecodeError};

/// Result returned by a complete read-only Security client method.
pub type SecurityReadResult<E> = Result<
    CheckedHetznerResponse,
    ClientExecutionError<AssociatedPreparationError, E, HetznerDecodeError>,
>;

/// Source-locked Security operation exposed by the service-typed client.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityClientMethodDescriptor {
    operation: OperationDescriptor,
}

impl SecurityClientMethodDescriptor {
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
        impl<T> HetznerClient<T, SecurityService, OfficialEndpointTrust>
        where
            T: BlockingAuthenticatedTransport + BoundTransport,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` synchronously.")]
            pub fn $blocking<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> SecurityReadResult<T::Error>
            where
                E: EndpointWire,
                Q: QueryWire,
                B: BodyWire,
            {
                self.execute_blocking(operation, lease)
            }
        }

        impl<T> HetznerClient<T, SecurityService, OfficialEndpointTrust>
        where
            T: AsyncAuthenticatedTransport + BoundTransport + Sync,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` through a `Send` future.")]
            #[allow(clippy::manual_async_fn)]
            pub fn $asynchronous<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> impl core::future::Future<Output = SecurityReadResult<T::Error>> + Send
            where
                E: EndpointWire + Sync,
                Q: QueryWire + Sync,
                B: BodyWire + Sync,
                T::Error: Send,
            {
                self.execute_async(operation, lease)
            }
        }

        impl<T> HetznerClient<T, SecurityService, OfficialEndpointTrust>
        where
            T: LocalAsyncAuthenticatedTransport + BoundTransport,
        {
            #[doc = concat!("Executes `", stringify!($marker), "` on a local executor.")]
            pub async fn $local<E, Q, B, const N: usize>(
                &self,
                operation: &AssociatedOperation<operations::$marker, E, Q, B>,
                lease: ClientWorkspaceLease<'_, '_, N>,
            ) -> SecurityReadResult<T::Error>
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
        impl<T> HetznerClient<T, SecurityService, OfficialEndpointTrust> {
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

        impl<T> HetznerClient<T, SecurityService, OfficialEndpointTrust>
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

        impl<T> HetznerClient<T, SecurityService, OfficialEndpointTrust>
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

        impl<T> HetznerClient<T, SecurityService, OfficialEndpointTrust>
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

macro_rules! security_client_method {
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, none) => {
        read_method!($marker, $blocking, $asynchronous, $local);
    };
    ($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, $permit:ident) => {
        permitted_method!($marker, $prepare, $blocking, $asynchronous, $local);
    };
}

macro_rules! security_client_methods {
    ($(($marker:ident, $prepare:ident, $blocking:ident, $asynchronous:ident, $local:ident, $permit:ident),)+) => {
        $(security_client_method!($marker, $prepare, $blocking, $asynchronous, $local, $permit);)+

        /// Exhaustive source-locked Security operation client surface.
        pub const SECURITY_CLIENT_METHODS: &[SecurityClientMethodDescriptor] = &[
            $(SecurityClientMethodDescriptor::new(<operations::$marker as HetznerOperation>::DESCRIPTOR),)+
        ];
    };
}

#[rustfmt::skip]
security_client_methods!(
    (CreateCertificate, prepare_create_certificate, create_certificate_blocking, create_certificate_async, create_certificate_local_async, mutation),
    (CreateSshKey, prepare_create_ssh_key, create_ssh_key_blocking, create_ssh_key_async, create_ssh_key_local_async, mutation),
    (DeleteCertificate, prepare_delete_certificate, delete_certificate_blocking, delete_certificate_async, delete_certificate_local_async, destructive),
    (DeleteSshKey, prepare_delete_ssh_key, delete_ssh_key_blocking, delete_ssh_key_async, delete_ssh_key_local_async, destructive),
    (GetCertificate, prepare_get_certificate, get_certificate_blocking, get_certificate_async, get_certificate_local_async, none),
    (GetCertificatesAction, prepare_get_certificates_action, get_certificates_action_blocking, get_certificates_action_async, get_certificates_action_local_async, none),
    (GetSshKey, prepare_get_ssh_key, get_ssh_key_blocking, get_ssh_key_async, get_ssh_key_local_async, none),
    (ListCertificateActions, prepare_list_certificate_actions, list_certificate_actions_blocking, list_certificate_actions_async, list_certificate_actions_local_async, none),
    (ListCertificates, prepare_list_certificates, list_certificates_blocking, list_certificates_async, list_certificates_local_async, none),
    (ListCertificatesActions, prepare_list_certificates_actions, list_certificates_actions_blocking, list_certificates_actions_async, list_certificates_actions_local_async, none),
    (ListSshKeys, prepare_list_ssh_keys, list_ssh_keys_blocking, list_ssh_keys_async, list_ssh_keys_local_async, none),
    (RetryCertificate, prepare_retry_certificate, retry_certificate_blocking, retry_certificate_async, retry_certificate_local_async, mutation),
    (UpdateCertificate, prepare_update_certificate, update_certificate_blocking, update_certificate_async, update_certificate_local_async, mutation),
    (UpdateSshKey, prepare_update_ssh_key, update_ssh_key_blocking, update_ssh_key_async, update_ssh_key_local_async, mutation),
);
