//! Prepared operation storage, endpoint binding, and execution.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

use crate::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, AuthenticationScopePolicy,
    BlockingAuthenticatedTransport, drive_async_authenticated,
};
use crate::operation::{
    CheckedResponseGuard, OperationId, OperationImpact, OperationMetadata, RequestIdPolicy,
    ResponsePolicy, ResponsePolicyError,
};
use crate::transport::{
    BoundTransport, EndpointPolicy, RawResponsePolicy, ResponseBuffer, TransportRequest,
};
use crate::{ProviderId, ProviderMarker, ServiceId, ServiceMarker};

mod error;
use error::{EndpointCheckError, map_endpoint_error};
pub use error::{PreparedExecutionError, PreparedRequestPolicyError};

/// Whether one prepared request body can be sent again byte-for-byte.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BodyReplayability {
    /// The body source cannot guarantee an identical subsequent read.
    NotReplayable,
    /// The complete body is an immutable byte snapshot for the request lifetime.
    Replayable,
}

/// Caller-owned target and request-body storage supplied to preparation.
pub struct PreparationStorage<'storage> {
    target: &'storage mut [u8],
    body: &'storage mut [u8],
}

impl<'storage> PreparationStorage<'storage> {
    /// Creates complete caller-owned storage for one preparation attempt.
    ///
    /// # Security
    ///
    /// Preparation may write credentials or other secrets into `body`. A
    /// successful [`PreparedRequest`] must retain those bytes until transport
    /// use, so this wrapper cannot clear them on success. For secret-bearing
    /// operations, guard `body` with a volatile-clearing type such as
    /// `cloud_sdk_sanitization::SecretBuffer` and drop the guard immediately
    /// after transport use. A plain mutable slice is not cleared when the
    /// prepared request is dropped.
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

/// Provider service and immutable endpoint trust policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderService<'endpoint> {
    provider_id: ProviderId,
    service_id: ServiceId,
    endpoint_policy: EndpointPolicy<'endpoint>,
}

impl<'endpoint> ProviderService<'endpoint> {
    /// Binds validated provider and service IDs to an endpoint trust policy.
    #[must_use]
    pub const fn new(
        provider_id: ProviderId,
        service_id: ServiceId,
        endpoint_policy: EndpointPolicy<'endpoint>,
    ) -> Self {
        Self {
            provider_id,
            service_id,
            endpoint_policy,
        }
    }

    /// Binds a provider-owned service marker to an endpoint trust policy.
    #[must_use]
    pub const fn from_marker<S: ServiceMarker>(endpoint_policy: EndpointPolicy<'endpoint>) -> Self {
        Self::new(<S::Provider as ProviderMarker>::ID, S::ID, endpoint_policy)
    }

    /// Returns the canonical provider namespace.
    #[must_use]
    pub const fn provider_id(self) -> ProviderId {
        self.provider_id
    }

    /// Returns the canonical provider-owned service namespace.
    #[must_use]
    pub const fn service_id(self) -> ServiceId {
        self.service_id
    }

    /// Returns the immutable endpoint trust policy.
    #[must_use]
    pub const fn endpoint_policy(self) -> EndpointPolicy<'endpoint> {
        self.endpoint_policy
    }
}

/// Complete request, endpoint, operation metadata, and response policy.
#[derive(Clone, Copy)]
pub struct PreparedRequest<'request> {
    request: TransportRequest<'request>,
    service: ProviderService<'request>,
    metadata: OperationMetadata,
    response_policy: ResponsePolicy,
    authentication_policy: AuthenticationScopePolicy<'request>,
    raw_response_policy: RawResponsePolicy<'request>,
    operation_id: Option<OperationId>,
    body_replayability: BodyReplayability,
}

impl<'request> PreparedRequest<'request> {
    /// Creates a complete prepared request after checking cross-policy invariants.
    ///
    /// # Errors
    ///
    /// Returns [`PreparedRequestPolicyError::MissingRequestIdHeader`] when
    /// operation metadata protects or retains request IDs but the raw response
    /// policy does not admit `x-request-id`.
    pub fn new(
        request: TransportRequest<'request>,
        service: ProviderService<'request>,
        metadata: OperationMetadata,
        response_policy: ResponsePolicy,
        authentication_policy: AuthenticationScopePolicy<'request>,
        raw_response_policy: RawResponsePolicy<'request>,
    ) -> Result<Self, PreparedRequestPolicyError> {
        if metadata.request_id_policy() != RequestIdPolicy::Discard
            && !raw_response_policy.admits_header("x-request-id")
        {
            return Err(PreparedRequestPolicyError::MissingRequestIdHeader);
        }
        Ok(Self {
            request,
            service,
            metadata,
            response_policy,
            authentication_policy,
            raw_response_policy,
            operation_id: None,
            body_replayability: if request.body().is_empty() {
                BodyReplayability::Replayable
            } else {
                BodyReplayability::NotReplayable
            },
        })
    }

    /// Binds a validated provider operation identifier to this request.
    #[must_use]
    pub const fn with_operation_id(mut self, operation_id: OperationId) -> Self {
        self.operation_id = Some(operation_id);
        self
    }

    /// Marks the immutable prepared body snapshot as byte-for-byte replayable.
    ///
    /// Providers must call this only after preparation has completed and when
    /// the borrowed body bytes cannot change for the prepared request lifetime.
    #[must_use]
    pub const fn with_replayable_body(mut self) -> Self {
        self.body_replayability = BodyReplayability::Replayable;
        self
    }

    /// Returns the validated transport request.
    #[must_use]
    pub const fn transport_request(self) -> TransportRequest<'request> {
        self.request
    }

    /// Returns the bound provider service.
    #[must_use]
    pub const fn service(self) -> ProviderService<'request> {
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

    /// Returns the complete provider-owned authentication-scope policy.
    #[must_use]
    pub const fn authentication_policy(self) -> AuthenticationScopePolicy<'request> {
        self.authentication_policy
    }

    /// Returns the complete status-class raw response policy.
    #[must_use]
    pub const fn raw_response_policy(self) -> RawResponsePolicy<'request> {
        self.raw_response_policy
    }

    /// Returns the request with its mandatory authentication and raw wire policy.
    #[must_use]
    pub const fn authenticated_request(self) -> AuthenticatedRequest<'request, 'request> {
        AuthenticatedRequest::new(
            self.request,
            self.authentication_policy,
            self.raw_response_policy,
        )
    }

    /// Returns the provider operation identifier when one was bound.
    #[must_use]
    pub const fn operation_id(self) -> Option<OperationId> {
        self.operation_id
    }

    /// Returns the explicit request-body replay capability.
    #[must_use]
    pub const fn body_replayability(self) -> BodyReplayability {
        self.body_replayability
    }

    pub(crate) fn has_same_retry_policy(&self, other: &Self) -> bool {
        self.service == other.service
            && self.metadata == other.metadata
            && self.response_policy == other.response_policy
            && self.authentication_policy == other.authentication_policy
            && self.raw_response_policy == other.raw_response_policy
            && self.operation_id == other.operation_id
            && self.body_replayability == other.body_replayability
            && self.has_same_header_policy(other)
    }

    fn has_same_header_policy(&self, other: &Self) -> bool {
        let left = self.request.headers().as_slice();
        let right = other.request.headers().as_slice();
        left.len() == right.len()
            && left
                .iter()
                .zip(right)
                .all(|(left, right)| left.sensitivity() == right.sensitivity())
    }

    /// Applies the complete prepared response policy without executing transport.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedResponseGuard<'buffer>, ResponsePolicyError> {
        self.response_policy
            .validate(response, self.metadata.request_id_policy())
    }

    /// Applies operation-owned metadata policy before provider error decoding.
    ///
    /// This is the error-status counterpart to [`Self::validate_response`].
    /// It extracts and protects, discards, or admits retention of the provider
    /// request identifier without applying success-status or body policy.
    pub fn apply_response_metadata_policy(
        self,
        response: &mut ResponseBuffer<'_>,
    ) -> Result<(), ResponsePolicyError> {
        super::policy::apply_request_id_policy(response, self.metadata.request_id_policy())
    }

    /// Verifies endpoint identity, executes once, and validates the response.
    pub fn execute_blocking<'buffer, T>(
        self,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
    {
        if self.requires_execution_permit() {
            sanitize_bytes(response_storage);
            sanitize_bytes(response_header_storage);
            return Err(PreparedExecutionError::AuthorizationRequired);
        }
        self.execute_blocking_authorized(transport, response_storage, response_header_storage)
    }

    pub(crate) fn execute_blocking_authorized<'buffer, T>(
        self,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
    {
        let mut response = ResponseBuffer::new(
            response_storage,
            self.raw_response_policy.max_body_bytes(),
            response_header_storage,
        );
        self.verify_endpoint(transport)
            .map_err(map_endpoint_error)?;
        transport
            .send_authenticated(self.authenticated_request(), response.writer())
            .map_err(PreparedExecutionError::Transport)?;
        self.response_policy
            .validate(response, self.metadata.request_id_policy())
            .map_err(PreparedExecutionError::ResponsePolicy)
    }

    /// Async equivalent of [`Self::execute_blocking`] without owning an executor.
    pub async fn execute_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        if self.requires_execution_permit() {
            sanitize_bytes(response_storage);
            sanitize_bytes(response_header_storage);
            return Err(PreparedExecutionError::AuthorizationRequired);
        }
        self.execute_async_authorized(transport, response_storage, response_header_storage)
            .await
    }

    pub(crate) async fn execute_async_authorized<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        let mut response = ResponseBuffer::new(
            response_storage,
            self.raw_response_policy.max_body_bytes(),
            response_header_storage,
        );
        self.verify_endpoint(transport)
            .map_err(map_endpoint_error)?;
        drive_async_authenticated(transport, self.authenticated_request(), response.writer())
            .await
            .map_err(|error| match error {
                crate::transport::AsyncExecutionError::Transport(error) => {
                    PreparedExecutionError::Transport(error)
                }
                crate::transport::AsyncExecutionError::Response(error) => {
                    PreparedExecutionError::ResponseWriter(error)
                }
            })?;
        self.response_policy
            .validate(response, self.metadata.request_id_policy())
            .map_err(PreparedExecutionError::ResponsePolicy)
    }

    pub(crate) const fn requires_execution_permit(self) -> bool {
        !matches!(self.metadata.impact(), OperationImpact::ReadOnly)
            || matches!(self.metadata.cost_intent(), super::CostIntent::MayIncurCost)
    }

    fn verify_endpoint<T>(self, transport: &T) -> Result<(), EndpointCheckError>
    where
        T: BoundTransport,
    {
        let actual = transport
            .endpoint_identity()
            .map_err(EndpointCheckError::Invalid)?;
        self.service
            .endpoint_policy
            .verify(actual)
            .map_err(|_| EndpointCheckError::Mismatch)
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
            .field("authentication_policy", &self.authentication_policy)
            .field("raw_response_policy", &self.raw_response_policy)
            .field("operation_id", &self.operation_id)
            .field("body_replayability", &self.body_replayability)
            .finish()
    }
}
