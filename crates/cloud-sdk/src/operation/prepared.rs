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
    BoundTransport, EndpointIdentity, RawResponsePolicy, RequestHeaders, ResponseBuffer,
    TransportRequest,
};

mod body;
mod error;
mod service;
mod storage;
pub use body::{BodyReplayability, RequestBodySensitivity};
use error::{EndpointCheckError, map_endpoint_error};
pub use error::{PreparedExecutionError, PreparedRequestPolicyError};
pub use service::ProviderService;
pub use storage::{PreparationStorage, PrepareOperation};

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
    body_sensitivity: RequestBodySensitivity,
}

impl<'request> PreparedRequest<'request> {
    /// Creates a complete prepared request after checking cross-policy invariants.
    ///
    /// # Errors
    ///
    /// Returns [`PreparedRequestPolicyError::MissingRequestIdHeader`] when
    /// operation metadata protects or retains request IDs but the raw response
    /// policy does not admit `x-request-id`. Returns
    /// [`PreparedRequestPolicyError::ReadOnlyMethodMismatch`] when read-only
    /// metadata is paired with any method other than `GET` or `HEAD`.
    /// `body_sensitivity` is mandatory so provider implementations cannot omit
    /// the confidential-body review and silently receive a public default.
    pub fn new(
        request: TransportRequest<'request>,
        service: ProviderService<'request>,
        metadata: OperationMetadata,
        response_policy: ResponsePolicy,
        authentication_policy: AuthenticationScopePolicy<'request>,
        raw_response_policy: RawResponsePolicy<'request>,
        body_sensitivity: RequestBodySensitivity,
    ) -> Result<Self, PreparedRequestPolicyError> {
        if matches!(metadata.impact(), OperationImpact::ReadOnly)
            && !request.method().permits_direct_read_only()
        {
            return Err(PreparedRequestPolicyError::ReadOnlyMethodMismatch);
        }
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
            body_sensitivity,
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

    /// Upgrades the explicit body classification to sensitive.
    ///
    /// This cannot downgrade a sensitive body. Providers should classify the
    /// body at construction; this helper supports reviewed wrapper policies.
    #[must_use]
    pub const fn with_sensitive_body(mut self) -> Self {
        self.body_sensitivity = RequestBodySensitivity::Sensitive;
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
    pub(crate) const fn authenticated_request(self) -> AuthenticatedRequest<'request, 'request> {
        AuthenticatedRequest::new(
            self.request,
            self.authentication_policy,
            &self.raw_response_policy,
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

    /// Returns the provider-declared request-body sensitivity.
    #[must_use]
    pub const fn body_sensitivity(self) -> RequestBodySensitivity {
        self.body_sensitivity
    }

    pub(crate) fn with_request_headers<'headers>(
        self,
        headers: RequestHeaders<'headers>,
    ) -> PreparedRequest<'headers>
    where
        'request: 'headers,
    {
        let request: TransportRequest<'headers> = self.request;
        PreparedRequest {
            request: request.with_headers(headers),
            service: self.service,
            metadata: self.metadata,
            response_policy: self.response_policy,
            authentication_policy: self.authentication_policy,
            raw_response_policy: self.raw_response_policy,
            operation_id: self.operation_id,
            body_replayability: self.body_replayability,
            body_sensitivity: self.body_sensitivity,
        }
    }

    pub(crate) fn has_same_retry_policy(&self, other: &Self) -> bool {
        self.service == other.service
            && self.metadata == other.metadata
            && self.response_policy == other.response_policy
            && self.authentication_policy == other.authentication_policy
            && self.raw_response_policy == other.raw_response_policy
            && self.operation_id == other.operation_id
            && self.body_replayability == other.body_replayability
            && self.body_sensitivity == other.body_sensitivity
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
        let response = self.send_blocking(transport, response_storage, response_header_storage)?;
        self.response_policy
            .validate(response, self.metadata.request_id_policy())
            .map_err(PreparedExecutionError::ResponsePolicy)
    }

    pub(crate) fn send_blocking<'buffer, T>(
        self,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<ResponseBuffer<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
    {
        if self.requires_execution_permit() {
            sanitize_bytes(response_storage);
            sanitize_bytes(response_header_storage);
            return Err(PreparedExecutionError::AuthorizationRequired);
        }
        self.send_blocking_authorized(transport, None, response_storage, response_header_storage)
    }

    pub(crate) fn execute_blocking_authorized<'buffer, T>(
        self,
        transport: &T,
        confirmed_endpoint: Option<EndpointIdentity<'_>>,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
    {
        let response = self.send_blocking_authorized(
            transport,
            confirmed_endpoint,
            response_storage,
            response_header_storage,
        )?;
        self.response_policy
            .validate(response, self.metadata.request_id_policy())
            .map_err(PreparedExecutionError::ResponsePolicy)
    }

    pub(crate) fn send_blocking_authorized<'buffer, T>(
        self,
        transport: &T,
        confirmed_endpoint: Option<EndpointIdentity<'_>>,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<ResponseBuffer<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
    {
        let mut response = ResponseBuffer::new(
            response_storage,
            self.raw_response_policy.max_body_bytes(),
            response_header_storage,
        );
        self.verify_endpoint(transport, confirmed_endpoint)
            .map_err(map_endpoint_error)?;
        transport
            .send_authenticated(self.authenticated_request(), response.writer())
            .map_err(PreparedExecutionError::Transport)?;
        Ok(response)
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
        let response = self
            .send_async(transport, response_storage, response_header_storage)
            .await?;
        self.response_policy
            .validate(response, self.metadata.request_id_policy())
            .map_err(PreparedExecutionError::ResponsePolicy)
    }

    pub(crate) async fn send_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<ResponseBuffer<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        if self.requires_execution_permit() {
            sanitize_bytes(response_storage);
            sanitize_bytes(response_header_storage);
            return Err(PreparedExecutionError::AuthorizationRequired);
        }
        self.send_async_authorized(transport, None, response_storage, response_header_storage)
            .await
    }

    pub(crate) async fn execute_async_authorized<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        confirmed_endpoint: Option<EndpointIdentity<'_>>,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        let response = self
            .send_async_authorized(
                transport,
                confirmed_endpoint,
                response_storage,
                response_header_storage,
            )
            .await?;
        self.response_policy
            .validate(response, self.metadata.request_id_policy())
            .map_err(PreparedExecutionError::ResponsePolicy)
    }

    pub(crate) async fn send_async_authorized<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        confirmed_endpoint: Option<EndpointIdentity<'_>>,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<ResponseBuffer<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        let mut response = ResponseBuffer::new(
            response_storage,
            self.raw_response_policy.max_body_bytes(),
            response_header_storage,
        );
        self.verify_endpoint(transport, confirmed_endpoint)
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
        Ok(response)
    }

    pub(crate) const fn requires_execution_permit(self) -> bool {
        !self.request.method().permits_direct_read_only()
            || !matches!(self.metadata.impact(), OperationImpact::ReadOnly)
            || matches!(self.metadata.cost_intent(), super::CostIntent::MayIncurCost)
    }

    fn verify_endpoint<T>(
        self,
        transport: &T,
        confirmed_endpoint: Option<EndpointIdentity<'_>>,
    ) -> Result<(), EndpointCheckError>
    where
        T: BoundTransport,
    {
        let actual = transport
            .endpoint_identity()
            .map_err(EndpointCheckError::Invalid)?;
        match confirmed_endpoint {
            Some(expected) if actual == expected => Ok(()),
            Some(_) => Err(EndpointCheckError::Mismatch),
            None => self
                .service
                .endpoint_policy()
                .verify(actual)
                .map_err(|_| EndpointCheckError::Mismatch),
        }
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
            .field("body_sensitivity", &self.body_sensitivity)
            .finish()
    }
}
