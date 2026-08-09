//! Provider-neutral client trait bridge for associated read-only operations.

use cloud_sdk::ServiceMarker;
use cloud_sdk::client::{ClientOperation, ClientResponse};
use cloud_sdk::operation::{PreparationStorage, PrepareOperation, PreparedRequest};

use super::{AssociatedOperation, AssociatedPreparationError, ReadOnlyOperation};
use crate::prepared::{BodyWire, EndpointWire, QueryWire};
use crate::serde::{CheckedHetznerResponse, HetznerDecodeError, decode_client_response};

mod private {
    pub trait Sealed {}
}

/// Read-only associated request executable by a service-matched Hetzner client.
///
/// Foreign operation types cannot claim a Hetzner service association:
///
/// ```compile_fail
/// use cloud_sdk::client::{ClientOperation, ClientResponse};
/// use cloud_sdk::operation::{PreparationStorage, PrepareOperation, PreparedRequest};
/// use cloud_sdk_hetzner::association::HetznerClientOperation;
/// use cloud_sdk_hetzner::identity::CloudService;
///
/// struct ForeignOperation;
///
/// impl PrepareOperation for ForeignOperation {
///     type Error = ();
///     fn prepare<'a>(
///         &self,
///         _: PreparationStorage<'a>,
///     ) -> Result<PreparedRequest<'a>, Self::Error> {
///         Err(())
///     }
/// }
///
/// impl ClientOperation for ForeignOperation {
///     type Output = ();
///     type DecodeError = ();
///     fn decode_response(&self, _: ClientResponse<'_, '_>) -> Result<(), ()> {
///         Ok(())
///     }
/// }
///
/// impl HetznerClientOperation for ForeignOperation {
///     type Service = CloudService;
/// }
/// ```
#[allow(private_bounds)]
pub trait HetznerClientOperation: ClientOperation + private::Sealed {
    /// Provider-owned service required by this operation.
    type Service: ServiceMarker<Provider = crate::identity::Hetzner>;
}

impl<O, E, Q, B> private::Sealed for AssociatedOperation<O, E, Q, B>
where
    O: ReadOnlyOperation,
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
}

impl<O, E, Q, B> PrepareOperation for AssociatedOperation<O, E, Q, B>
where
    O: ReadOnlyOperation,
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    type Error = AssociatedPreparationError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        self.prepare_typed(storage)
            .map(|prepared| prepared.into_untyped())
    }
}

impl<O, E, Q, B> ClientOperation for AssociatedOperation<O, E, Q, B>
where
    O: ReadOnlyOperation,
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    type Output = CheckedHetznerResponse;
    type DecodeError = HetznerDecodeError;

    fn decode_response(
        &self,
        response: ClientResponse<'_, '_>,
    ) -> Result<Self::Output, Self::DecodeError> {
        decode_client_response(
            response,
            O::DESCRIPTOR.operation_id(),
            O::DESCRIPTOR.service_id(),
            self.expected_response_identity(),
        )
    }
}

impl<O, E, Q, B> HetznerClientOperation for AssociatedOperation<O, E, Q, B>
where
    O: ReadOnlyOperation,
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    type Service = O::Service;
}
