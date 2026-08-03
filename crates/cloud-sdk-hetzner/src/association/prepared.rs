//! Typed operation assembly and prepared-request validation.

use core::fmt;
use core::marker::PhantomData;

use cloud_sdk::authentication::{AsyncAuthenticatedTransport, LocalAsyncAuthenticatedTransport};
use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PreparedExecutionError, PreparedRequest,
    ProviderService, ResponseBodyPolicy, ResponsePolicyError,
};
use cloud_sdk::transport::{BoundTransport, MediaType, ResponseBuffer, ResponseMediaPolicy};

use super::components::{AssociationError, BodyFor, EndpointFor, QueryFor};
use super::policy::{
    AuthenticationClass, BodyPolicy, HetznerOperation, QueryPolicy, ResponseShape,
};
use super::types::{metadata_permit, metadata_retry};
use crate::endpoint::ApiSurface;
use crate::prepared::{
    BodyWire, EndpointWire, HetznerPreparationError, NoBody, NoQuery, QueryWire,
    authentication_policy, prepare_parts,
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
        let authentication = match endpoint.endpoint_group().surface() {
            ApiSurface::Storage => AuthenticationClass::Basic,
            ApiSurface::Cloud | ApiSurface::Dns | ApiSurface::Security => {
                AuthenticationClass::Bearer
            }
        };
        let request = prepare_parts(
            endpoint,
            self.query.into_inner(),
            self.body.into_inner(),
            storage,
        )
        .map_err(AssociatedPreparationError::Preparation)?;
        Prepared::try_new(request, authentication).map_err(AssociatedPreparationError::Association)
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
#[derive(Clone, Copy)]
pub struct Prepared<'request, O> {
    inner: PreparedRequest<'request>,
    operation: PhantomData<fn() -> O>,
}

impl<'request, O: HetznerOperation> Prepared<'request, O> {
    #[allow(
        clippy::large_types_passed_by_value,
        reason = "ownership transfer keeps typed preparation allocation-free"
    )]
    fn try_new(
        inner: PreparedRequest<'request>,
        authentication: AuthenticationClass,
    ) -> Result<Self, AssociationError> {
        validate_policy::<O>(&inner, authentication)?;
        Ok(Self {
            inner,
            operation: PhantomData,
        })
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

    /// Explicitly erases the operation marker.
    #[must_use]
    pub const fn into_untyped(self) -> PreparedRequest<'request> {
        self.inner
    }

    /// Applies the operation-owned response policy without transport execution.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedResponseGuard<'buffer>, ResponsePolicyError> {
        self.inner.validate_response(response)
    }

    /// Executes once through a blocking authenticated transport.
    pub fn execute_blocking<'buffer, T>(
        self,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: cloud_sdk::authentication::BlockingAuthenticatedTransport + BoundTransport,
    {
        self.inner
            .execute_blocking(transport, response_storage, response_header_storage)
    }

    /// Executes once through a `Send` asynchronous authenticated transport.
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
        self.inner
            .execute_async(transport, response_storage, response_header_storage)
            .await
    }

    /// Executes once through a local asynchronous authenticated transport.
    pub async fn execute_local_async<'transport, 'buffer, T>(
        &'transport self,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
    {
        self.inner
            .execute_local_async(transport, response_storage, response_header_storage)
            .await
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

fn validate_policy<O: HetznerOperation>(
    prepared: &PreparedRequest<'_>,
    authentication: AuthenticationClass,
) -> Result<(), AssociationError> {
    let descriptor = O::DESCRIPTOR;
    let request = prepared.transport_request();
    let service = prepared.service();
    let metadata = prepared.metadata();
    let response = prepared.response_policy();
    let raw = prepared.raw_response_policy();
    let expected_service = provider_service_for(descriptor)?;
    let expected_authentication =
        authentication_policy(expected_service, descriptor.api_base_url())
            .map_err(|_| AssociationError::PreparedPolicyMismatch)?;

    if prepared.operation_id() != Some(descriptor.operation_id())
        || request.method() != descriptor.method()
        || service != expected_service
        || authentication != descriptor.authentication()
        || prepared.authentication_policy() != expected_authentication
        || metadata_retry(metadata.semantics(), metadata.retry_eligibility()) != descriptor.retry()
        || metadata_permit(metadata.impact(), metadata.cost_intent()) != descriptor.permit()
        || !request_shape_matches(descriptor, request)
        || !request_headers_match(descriptor, request)
        || response.success_statuses() != [descriptor.success_status()]
        || response.max_body_bytes() != descriptor.success_body_bytes()
        || !success_response_matches(descriptor.response_shape(), response)
        || raw.body_limit(descriptor.success_status()) != descriptor.success_body_bytes()
        || raw.body_limit(cloud_sdk::transport::StatusCode::TOO_MANY_REQUESTS)
            != descriptor.error_body_bytes()
        || !raw_media_matches(descriptor, &raw)
    {
        return Err(AssociationError::PreparedPolicyMismatch);
    }
    Ok(())
}

fn provider_service_for(
    descriptor: super::OperationDescriptor,
) -> Result<cloud_sdk::operation::ProviderService<'static>, AssociationError> {
    let policy = crate::official_endpoint_policy(descriptor.api_base_url())
        .map_err(|_| AssociationError::PreparedPolicyMismatch)?;
    Ok(ProviderService::new(
        crate::HETZNER_PROVIDER_ID,
        descriptor.service_id(),
        policy,
    ))
}

fn request_shape_matches(
    descriptor: super::OperationDescriptor,
    request: cloud_sdk::transport::TransportRequest<'_>,
) -> bool {
    let has_query = request.target().as_str().contains('?');
    let has_body = !request.body().is_empty();
    let query = match descriptor.query_policy() {
        QueryPolicy::Forbidden => !has_query,
        QueryPolicy::Optional => true,
        QueryPolicy::Required => has_query,
    };
    let body = match descriptor.body_policy() {
        BodyPolicy::Forbidden => !has_body,
        BodyPolicy::RequiredJson => has_body,
    };
    query && body
}

fn request_headers_match(
    descriptor: super::OperationDescriptor,
    request: cloud_sdk::transport::TransportRequest<'_>,
) -> bool {
    let headers = request.headers();
    let accept = headers
        .get("accept")
        .is_some_and(|header| header.value().as_str() == MediaType::JSON.as_str());
    let content_type = headers
        .get("content-type")
        .is_some_and(|header| header.value().as_str() == MediaType::JSON.as_str());
    match descriptor.body_policy() {
        BodyPolicy::Forbidden => accept && !content_type && headers.as_slice().len() == 1,
        BodyPolicy::RequiredJson => accept && content_type && headers.as_slice().len() == 2,
    }
}

fn success_response_matches(
    shape: ResponseShape,
    response: cloud_sdk::operation::ResponsePolicy,
) -> bool {
    match shape {
        ResponseShape::Empty => {
            response.body_policy() == ResponseBodyPolicy::Forbidden
                && matches!(
                    response.content_type_policy(),
                    cloud_sdk::operation::ContentTypePolicy::Forbidden
                )
        }
        _ => {
            response.body_policy() == ResponseBodyPolicy::Required
                && matches!(
                    response.content_type_policy(),
                    cloud_sdk::operation::ContentTypePolicy::Required(types)
                        if types == [MediaType::JSON]
                )
        }
    }
}

fn raw_media_matches(
    descriptor: super::OperationDescriptor,
    raw: &cloud_sdk::transport::RawResponsePolicy<'_>,
) -> bool {
    let success = raw.media_policy(descriptor.success_status());
    let error = raw.media_policy(cloud_sdk::transport::StatusCode::TOO_MANY_REQUESTS);
    let expected_success = if matches!(descriptor.response_shape(), ResponseShape::Empty) {
        matches!(success, ResponseMediaPolicy::Forbidden)
    } else {
        matches!(success, ResponseMediaPolicy::Required(types) if types == [MediaType::JSON])
    };
    expected_success
        && matches!(error, ResponseMediaPolicy::Required(types) if types == [MediaType::JSON])
}
