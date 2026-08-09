//! Typed operation assembly and prepared-request validation.

use core::fmt;
use core::marker::PhantomData;

use cloud_sdk::authentication::{AsyncAuthenticatedTransport, LocalAsyncAuthenticatedTransport};
use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PreparationStorageGuard, PreparedExecutionError,
    PreparedRequest, ResponsePolicyError,
};
use cloud_sdk::transport::{BoundTransport, ResponseBuffer};

use super::ExpectedResponseIdentity;
use super::components::{AssociationError, BodyFor, EndpointFor, QueryFor};
use super::policy::{HetznerOperation, ReadOnlyOperation};
use super::validation::validate_association;
use crate::prepared::{
    BodyWire, EndpointWire, HetznerPreparationError, NoBody, NoQuery, QueryWire,
    clear_preparation_storage, prepare_parts_with_policy,
};

/// Failure while preparing a compile-time-associated operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssociatedPreparationError {
    /// Existing provider request preparation failed.
    Preparation(HetznerPreparationError),
    /// Runtime policy did not match the operation association.
    Association(AssociationError),
}

impl fmt::Display for AssociatedPreparationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Preparation(_) => "Hetzner request preparation failed",
            Self::Association(_) => "Hetzner operation association failed",
        })
    }
}

impl core::error::Error for AssociatedPreparationError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Preparation(error) => Some(error),
            Self::Association(error) => Some(error),
        }
    }
}

/// Endpoint, query, and body components carrying one common operation marker.
///
/// Cross-operation assembly fails to type-check:
///
/// ```compile_fail
/// use cloud_sdk_hetzner::association::{EndpointFor, QueryFor};
/// use cloud_sdk_hetzner::association::operations::{GetAction, GetActions};
///
/// fn combine<O, E, Q>(_: EndpointFor<O, E>, _: QueryFor<O, Q>) {}
/// fn mismatch<E, Q>(endpoint: EndpointFor<GetAction, E>, query: QueryFor<GetActions, Q>) {
///     combine(endpoint, query);
/// }
/// ```
pub struct AssociatedOperation<O, E, Q = NoQuery, B = NoBody> {
    endpoint: EndpointFor<O, E>,
    query: QueryFor<O, Q>,
    body: BodyFor<O, B>,
}

impl<O: HetznerOperation, E: super::EndpointComponent> AssociatedOperation<O, E> {
    /// Creates an associated operation without a query or body.
    pub fn endpoint(endpoint: E) -> Result<Self, AssociationError> {
        Ok(Self {
            endpoint: EndpointFor::try_new(endpoint)?,
            query: QueryFor::none()?,
            body: BodyFor::none()?,
        })
    }
}

impl<O: HetznerOperation, E: super::EndpointComponent, Q: super::QueryComponent>
    AssociatedOperation<O, E, Q>
{
    /// Creates an associated operation with a query and no body.
    pub fn query(endpoint: E, query: Q) -> Result<Self, AssociationError> {
        Ok(Self {
            endpoint: EndpointFor::try_new(endpoint)?,
            query: QueryFor::try_new(query)?,
            body: BodyFor::none()?,
        })
    }
}

impl<O: HetznerOperation, E: super::EndpointComponent, B: super::BodyComponent>
    AssociatedOperation<O, E, NoQuery, B>
{
    /// Creates an associated operation with a JSON body and no query.
    pub fn json(endpoint: E, body: B) -> Result<Self, AssociationError> {
        Ok(Self {
            endpoint: EndpointFor::try_new(endpoint)?,
            query: QueryFor::none()?,
            body: BodyFor::try_new(body)?,
        })
    }
}

impl<O: HetznerOperation, E, Q, B> AssociatedOperation<O, E, Q, B> {
    /// Creates an operation from components that were independently bound to `O`.
    ///
    /// The common `O` parameter makes endpoint/query/body mismatch
    /// unrepresentable at this boundary.
    #[must_use]
    pub const fn from_parts(
        endpoint: EndpointFor<O, E>,
        query: QueryFor<O, Q>,
        body: BodyFor<O, B>,
    ) -> Self {
        Self {
            endpoint,
            query,
            body,
        }
    }
}

#[allow(private_bounds)]
impl<O, E, Q, B> AssociatedOperation<O, E, Q, B>
where
    O: HetznerOperation,
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    /// Prepares and verifies a request while preserving `O` in the result type.
    pub fn prepare_typed<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<Prepared<'storage, O>, AssociatedPreparationError> {
        let endpoint = self.endpoint.into_inner();
        let expected_identity = endpoint.expected_response_identity();
        let storage = clear_preparation_storage(storage);
        let policy = validate_association::<O, _>(endpoint)
            .map_err(AssociatedPreparationError::Association)?;
        let request = prepare_parts_with_policy(
            endpoint,
            self.query.into_inner(),
            self.body.into_inner(),
            storage,
            &policy,
        )
        .map_err(AssociatedPreparationError::Preparation)?;
        Ok(Prepared::new(request, expected_identity))
    }

    /// Prepares through cleanup-owning storage while preserving `O`.
    ///
    /// The guard clears both complete buffers before association validation
    /// and preparation, then clears them again when dropped after transport.
    pub fn prepare_typed_guarded<'guard>(
        &self,
        storage: &'guard mut PreparationStorageGuard<'_>,
    ) -> Result<Prepared<'guard, O>, AssociatedPreparationError> {
        let endpoint = self.endpoint.into_inner();
        let expected_identity = endpoint.expected_response_identity();
        storage.prepare_with(|buffers| {
            let policy = validate_association::<O, _>(endpoint)
                .map_err(AssociatedPreparationError::Association)?;
            prepare_parts_with_policy(
                endpoint,
                self.query.into_inner(),
                self.body.into_inner(),
                buffers,
                &policy,
            )
            .map(|request| Prepared::new(request, expected_identity))
            .map_err(AssociatedPreparationError::Preparation)
        })
    }
}

impl<O: HetznerOperation, E, Q, B> fmt::Debug for AssociatedOperation<O, E, Q, B> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssociatedOperation")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("components", &"[bound]")
            .finish()
    }
}

/// Prepared request retaining its exact operation association.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::association::Prepared;
/// use cloud_sdk_hetzner::association::operations::{GetAction, GetActions};
///
/// fn decode(_: Prepared<'_, GetAction>) {}
/// fn wrong(request: Prepared<'_, GetActions>) {
///     decode(request);
/// }
/// ```
///
/// State-changing operations expose no direct execution method:
///
/// ```compile_fail
/// use cloud_sdk::authentication::BlockingAuthenticatedTransport;
/// use cloud_sdk::transport::BoundTransport;
/// use cloud_sdk_hetzner::association::{Prepared, operations::CreateServer};
///
/// fn bypass<T>(request: Prepared<'_, CreateServer>, transport: &T)
/// where
///     T: BlockingAuthenticatedTransport + BoundTransport,
/// {
///     let mut body = [0_u8; 64];
///     let mut headers = [0_u8; 128];
///     let _ = request.execute_blocking(transport, &mut body, &mut headers);
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Prepared<'request, O> {
    inner: PreparedRequest<'request>,
    #[cfg_attr(not(feature = "serde"), allow(dead_code))]
    expected_identity: ExpectedResponseIdentity,
    operation: PhantomData<fn() -> O>,
}

/// Checked response retaining the exact operation that produced it.
///
/// A response cannot be decoded through another operation marker:
///
/// ```compile_fail
/// use cloud_sdk_hetzner::association::{AssociatedCheckedResponse, operations};
/// use cloud_sdk_hetzner::serde::decode_associated_checked_response;
///
/// fn cross_wire(
///     response: AssociatedCheckedResponse<'_, operations::ListServers>,
/// ) {
///     let _ = decode_associated_checked_response::<operations::ListStorageBoxes>(response);
/// }
/// ```
pub struct AssociatedCheckedResponse<'buffer, O> {
    inner: CheckedResponseGuard<'buffer>,
    #[cfg_attr(not(feature = "serde"), allow(dead_code))]
    expected_identity: ExpectedResponseIdentity,
    operation: PhantomData<fn() -> O>,
}

impl<'buffer, O: HetznerOperation> AssociatedCheckedResponse<'buffer, O> {
    pub(super) fn new(
        inner: CheckedResponseGuard<'buffer>,
        expected_identity: ExpectedResponseIdentity,
    ) -> Self {
        Self {
            inner,
            expected_identity,
            operation: PhantomData,
        }
    }

    /// Returns the compile-time operation associated with this response.
    #[must_use]
    pub const fn association(&self) -> super::OperationDescriptor {
        O::DESCRIPTOR
    }

    /// Explicitly erases the operation marker and expected response identity.
    /// This leaves the operation-associated decoding contract and is intended
    /// only for callers implementing equivalent custom validation.
    #[must_use]
    pub fn into_untyped(self) -> CheckedResponseGuard<'buffer> {
        self.inner
    }

    #[cfg(feature = "serde")]
    pub(crate) fn into_parts(self) -> (CheckedResponseGuard<'buffer>, ExpectedResponseIdentity) {
        (self.inner, self.expected_identity)
    }
}

impl<O: HetznerOperation> fmt::Debug for AssociatedCheckedResponse<'_, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssociatedCheckedResponse")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("response", &"[checked]")
            .finish()
    }
}

impl<'request, O: HetznerOperation> Prepared<'request, O> {
    #[allow(
        clippy::large_types_passed_by_value,
        reason = "ownership transfer keeps typed preparation allocation-free"
    )]
    const fn new(
        inner: PreparedRequest<'request>,
        expected_identity: ExpectedResponseIdentity,
    ) -> Self {
        Self {
            inner,
            expected_identity,
            operation: PhantomData,
        }
    }

    /// Returns the complete compile-time association.
    #[must_use]
    pub const fn association(&self) -> super::OperationDescriptor {
        O::DESCRIPTOR
    }

    /// Borrows the provider-neutral prepared request without erasing this value.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'request> {
        self.inner
    }

    /// Explicitly erases the operation marker and expected response identity.
    /// The returned request cannot enforce provider resource identity during
    /// response decoding.
    #[must_use]
    pub const fn into_untyped(self) -> PreparedRequest<'request> {
        self.inner
    }

    #[cfg(feature = "serde")]
    pub(crate) const fn into_parts(self) -> (PreparedRequest<'request>, ExpectedResponseIdentity) {
        (self.inner, self.expected_identity)
    }

    pub(super) const fn into_associated_plan_parts(
        self,
    ) -> (PreparedRequest<'request>, ExpectedResponseIdentity) {
        (self.inner, self.expected_identity)
    }

    /// Applies the operation-owned response policy without transport execution.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<AssociatedCheckedResponse<'buffer, O>, ResponsePolicyError> {
        let expected_identity = self.expected_identity;
        self.inner
            .validate_response(response)
            .map(|inner| AssociatedCheckedResponse::new(inner, expected_identity))
    }
}

impl<'request, O: ReadOnlyOperation> Prepared<'request, O> {
    /// Executes one read-only operation through a blocking authenticated transport.
    pub fn execute_blocking<'buffer, T>(
        self,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<AssociatedCheckedResponse<'buffer, O>, PreparedExecutionError<T::Error>>
    where
        T: cloud_sdk::authentication::BlockingAuthenticatedTransport + BoundTransport,
    {
        let expected_identity = self.expected_identity;
        self.inner
            .execute_blocking(transport, response_storage, response_header_storage)
            .map(|inner| AssociatedCheckedResponse::new(inner, expected_identity))
    }

    /// Executes one read-only operation through a `Send` asynchronous transport.
    pub async fn execute_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<AssociatedCheckedResponse<'buffer, O>, PreparedExecutionError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        self.inner
            .execute_async(transport, response_storage, response_header_storage)
            .await
            .map(|inner| AssociatedCheckedResponse::new(inner, self.expected_identity))
    }

    /// Executes one read-only operation through a local asynchronous transport.
    pub async fn execute_local_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<AssociatedCheckedResponse<'buffer, O>, PreparedExecutionError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        self.inner
            .execute_local_async(transport, response_storage, response_header_storage)
            .await
            .map(|inner| AssociatedCheckedResponse::new(inner, self.expected_identity))
    }
}

impl<O: HetznerOperation> fmt::Debug for Prepared<'_, O> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Prepared")
            .field("operation", &O::DESCRIPTOR.operation_id())
            .field("request", &self.inner)
            .finish()
    }
}
