//! Prepared operation storage, endpoint binding, and execution.

use core::fmt;

use crate::operation::{CheckedResponse, OperationMetadata, ResponsePolicy, ResponsePolicyError};
use crate::transport::{
    AsyncTransport, BlockingTransport, BoundTransport, EndpointIdentity, EndpointIdentityError,
    TransportRequest,
};
use crate::{ApiFamily, Provider};

/// Caller-owned target and request-body storage supplied to preparation.
pub struct PreparationStorage<'storage> {
    target: &'storage mut [u8],
    body: &'storage mut [u8],
}

impl<'storage> PreparationStorage<'storage> {
    /// Creates complete caller-owned storage for one preparation attempt.
    #[must_use]
    pub const fn new(target: &'storage mut [u8], body: &'storage mut [u8]) -> Self {
        Self { target, body }
    }

    /// Consumes the storage wrapper and returns both independent buffers.
    #[must_use]
    pub fn into_parts(self) -> (&'storage mut [u8], &'storage mut [u8]) {
        (self.target, self.body)
    }
}

impl fmt::Debug for PreparationStorage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparationStorage")
            .field("target_capacity", &self.target.len())
            .field("body_capacity", &self.body.len())
            .finish()
    }
}

/// Typed provider operation that can prepare one complete request.
///
/// ```compile_fail
/// use cloud_sdk::operation::PrepareOperation;
///
/// fn prepare_without_storage<O: PrepareOperation>(operation: &O) {
///     let _ = operation.prepare();
/// }
/// ```
pub trait PrepareOperation {
    /// Preparation-specific failure.
    type Error;

    /// Writes into caller storage and returns an executable prepared request.
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error>;
}

/// Provider service and immutable expected endpoint identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderService {
    provider: Provider,
    family: ApiFamily,
    endpoint: EndpointIdentity<'static>,
}

impl ProviderService {
    /// Binds a provider API family to one normalized expected endpoint.
    #[must_use]
    pub const fn new(
        provider: Provider,
        family: ApiFamily,
        endpoint: EndpointIdentity<'static>,
    ) -> Self {
        Self {
            provider,
            family,
            endpoint,
        }
    }

    /// Returns the provider namespace.
    #[must_use]
    pub const fn provider(self) -> Provider {
        self.provider
    }

    /// Returns the provider API family.
    #[must_use]
    pub const fn family(self) -> ApiFamily {
        self.family
    }

    /// Returns the immutable expected endpoint identity.
    #[must_use]
    pub const fn endpoint(self) -> EndpointIdentity<'static> {
        self.endpoint
    }
}

/// Complete request, endpoint, operation metadata, and response policy.
#[derive(Clone, Copy)]
pub struct PreparedRequest<'request> {
    request: TransportRequest<'request>,
    service: ProviderService,
    metadata: OperationMetadata,
    response_policy: ResponsePolicy,
}

impl<'request> PreparedRequest<'request> {
    /// Creates a complete prepared request without policy defaults.
    #[must_use]
    pub const fn new(
        request: TransportRequest<'request>,
        service: ProviderService,
        metadata: OperationMetadata,
        response_policy: ResponsePolicy,
    ) -> Self {
        Self {
            request,
            service,
            metadata,
            response_policy,
        }
    }

    /// Returns the validated transport request.
    #[must_use]
    pub const fn transport_request(self) -> TransportRequest<'request> {
        self.request
    }

    /// Returns the bound provider service.
    #[must_use]
    pub const fn service(self) -> ProviderService {
        self.service
    }

    /// Returns complete safety and retry metadata.
    #[must_use]
    pub const fn metadata(self) -> OperationMetadata {
        self.metadata
    }

    /// Returns complete checked-response policy.
    #[must_use]
    pub const fn response_policy(self) -> ResponsePolicy {
        self.response_policy
    }

    /// Verifies endpoint identity, executes once, and validates the response.
    pub fn execute_blocking<'buffer, T>(
        self,
        transport: &T,
        response_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponse<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: BlockingTransport + BoundTransport,
    {
        self.verify_endpoint(transport)
            .map_err(map_endpoint_error)?;
        let admitted = self.admit_response_storage(response_storage)?;
        let response = transport
            .send(self.request, admitted)
            .map_err(PreparedExecutionError::Transport)?;
        self.response_policy
            .validate(response)
            .map_err(PreparedExecutionError::ResponsePolicy)
    }

    /// Async equivalent of [`Self::execute_blocking`] without owning an executor.
    pub async fn execute_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponse<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: AsyncTransport + BoundTransport,
        'request: 'transport,
        'buffer: 'transport,
    {
        self.verify_endpoint(transport)
            .map_err(map_endpoint_error)?;
        let admitted = self.admit_response_storage(response_storage)?;
        let response = transport
            .send(self.request, admitted)
            .await
            .map_err(PreparedExecutionError::Transport)?;
        self.response_policy
            .validate(response)
            .map_err(PreparedExecutionError::ResponsePolicy)
    }

    fn verify_endpoint<T>(self, transport: &T) -> Result<(), EndpointCheckError>
    where
        T: BoundTransport,
    {
        let actual = transport
            .endpoint_identity()
            .map_err(EndpointCheckError::Invalid)?;
        if actual != self.service.endpoint {
            return Err(EndpointCheckError::Mismatch);
        }
        Ok(())
    }

    fn admit_response_storage<E>(
        self,
        storage: &mut [u8],
    ) -> Result<&mut [u8], PreparedExecutionError<E>> {
        let admitted_len = core::cmp::min(storage.len(), self.response_policy.max_body_bytes());
        storage
            .get_mut(..admitted_len)
            .ok_or(PreparedExecutionError::ResponseStorageUnavailable)
    }
}

impl fmt::Debug for PreparedRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedRequest")
            .field("request", &self.request)
            .field("service", &self.service)
            .field("metadata", &self.metadata)
            .field("response_policy", &self.response_policy)
            .finish()
    }
}

/// Prepared execution failure with transport details redacted from diagnostics.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum PreparedExecutionError<E> {
    /// The bound transport returned invalid endpoint identity.
    EndpointIdentity(EndpointIdentityError),
    /// The bound endpoint differs from the prepared provider service.
    EndpointMismatch,
    /// Response storage could not be structurally limited.
    ResponseStorageUnavailable,
    /// The concrete transport failed.
    Transport(E),
    /// The response failed provider-neutral policy.
    ResponsePolicy(ResponsePolicyError),
}

impl<E> fmt::Debug for PreparedExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndpointIdentity(error) => formatter
                .debug_tuple("EndpointIdentity")
                .field(error)
                .finish(),
            Self::EndpointMismatch => formatter.write_str("EndpointMismatch"),
            Self::ResponseStorageUnavailable => formatter.write_str("ResponseStorageUnavailable"),
            Self::Transport(_) => formatter.write_str("Transport([redacted])"),
            Self::ResponsePolicy(error) => formatter
                .debug_tuple("ResponsePolicy")
                .field(error)
                .finish(),
        }
    }
}

impl<E> fmt::Display for PreparedExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EndpointIdentity(_) => "transport endpoint identity is invalid",
            Self::EndpointMismatch => "transport endpoint differs from prepared service",
            Self::ResponseStorageUnavailable => "response storage is unavailable",
            Self::Transport(_) => "prepared request transport failed",
            Self::ResponsePolicy(_) => "prepared response policy failed",
        })
    }
}

impl<E: fmt::Debug> core::error::Error for PreparedExecutionError<E> {}

enum EndpointCheckError {
    Invalid(EndpointIdentityError),
    Mismatch,
}

fn map_endpoint_error<E>(error: EndpointCheckError) -> PreparedExecutionError<E> {
    match error {
        EndpointCheckError::Invalid(error) => PreparedExecutionError::EndpointIdentity(error),
        EndpointCheckError::Mismatch => PreparedExecutionError::EndpointMismatch,
    }
}
