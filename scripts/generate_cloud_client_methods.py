#!/usr/bin/env python3
"""Generate exhaustive service-typed Hetzner Cloud client methods."""

from __future__ import annotations

import argparse
from pathlib import Path

import generate_operation_associations as associations

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "crates/cloud-sdk-hetzner/src/client/cloud.rs"
EXPECTED_CLOUD_OPERATIONS = 139


HEADER = """//! Generated exhaustive methods for the official Hetzner Cloud client.
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

"""


def render_row(operation: associations.Operation) -> str:
    marker = associations.pascal(operation.operation_id)
    name = operation.operation_id
    return (
        f"    ({marker}, prepare_{name}, {name}_blocking, {name}_async, "
        f"{name}_local_async, {operation.permit_class}),"
    )


def render() -> str:
    operations = [
        operation
        for operation in associations.load_operations()
        if operation.service == "cloud"
    ]
    if len(operations) != EXPECTED_CLOUD_OPERATIONS:
        raise ValueError("Cloud client operation count changed")
    rows = "\n".join(render_row(operation) for operation in operations)
    return f"{HEADER}#[rustfmt::skip]\ncloud_client_methods!(\n{rows}\n);\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = render()
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_text(encoding="ascii") != generated:
            raise SystemExit("Cloud client methods are stale; regenerate them")
        print(f"{EXPECTED_CLOUD_OPERATIONS} Cloud client operations are current.")
        return 0
    OUTPUT.write_text(generated, encoding="ascii")
    print(f"generated {EXPECTED_CLOUD_OPERATIONS} Cloud client operations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
