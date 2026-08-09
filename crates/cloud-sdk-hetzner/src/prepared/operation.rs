//! Generic operation assembly over sealed provider wire components.

use core::marker::PhantomData;

use cloud_sdk::Method;
use cloud_sdk::authentication::AuthenticationScopePolicy;
use cloud_sdk::operation::{
    BodyReplayability, ContentTypePolicy, CostIntent, OperationId, OperationImpact,
    OperationMetadata, PreparationStorage, PrepareOperation, PreparedRequest, ProviderService,
    RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    ContentType, MediaType, RawResponsePolicy, RequestHeader, RequestHeaders, RequestTarget,
    StatusCode, TransportRequest,
};

use crate::association::validation::ValidatedAssociationPolicy;
use crate::endpoint::EndpointGroup;
use crate::request::ApiBaseUrl;

use super::HetznerPreparationError;
use super::wire_policy::{authentication_policy, provider_service, raw_response_policy};

const JSON_MEDIA: &[MediaType<'static>] = &[MediaType::JSON];
const STATUS_OK: &[StatusCode] = &[StatusCode::OK];
const STATUS_CREATED: &[StatusCode] = &[StatusCode::CREATED];
const STATUS_NO_CONTENT: &[StatusCode] = &[StatusCode::NO_CONTENT];
const MAX_JSON_RESPONSE_BYTES: usize = 8_388_608;
const ACCEPT_JSON_HEADERS: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const JSON_REQUEST_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::JSON),
];

/// Request components admitted by one endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RequestShape {
    None,
    OptionalQuery,
    RequiredQuery,
    RequiredJson,
}

/// Source-locked successful response shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResponseProfile {
    JsonOk,
    JsonCreated,
    NoContent,
}

/// Provider-owned operation safety and retry classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperationClass {
    ReadOnly,
    IdempotentMutation,
    NonIdempotentMutation,
    IdempotentDestructive,
    NonIdempotentDestructive,
}

pub(crate) trait EndpointWire: Copy {
    fn method(self) -> Method;
    fn api_base_url(self) -> ApiBaseUrl;
    fn endpoint_group(self) -> EndpointGroup;
    fn write_path(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError>;
    fn request_shape(self) -> RequestShape;
    fn response_profile(self) -> ResponseProfile;
    fn metadata(self) -> Result<OperationMetadata, HetznerPreparationError>;
    fn operation_key(self) -> &'static str;

    fn expected_response_identity(self) -> crate::association::ExpectedResponseIdentity {
        crate::association::ExpectedResponseIdentity::None
    }
}

pub(crate) trait QueryWire: Copy {
    fn write_query(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError>;
    fn operation_key(self) -> &'static str;

    fn accepts_operation(self, operation_key: &str) -> bool {
        self.operation_key() == operation_key
    }
}

pub(crate) trait BodyWire: Copy {
    fn write_body(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError>;
    fn operation_key(self) -> &'static str;

    fn accepts_operation(self, operation_key: &str) -> bool {
        self.operation_key() == operation_key
    }
}

#[derive(Clone, Copy)]
struct RequestAssemblyPolicy {
    operation_id: OperationId,
    method: Method,
    request_shape: RequestShape,
    service: ProviderService<'static>,
    metadata: OperationMetadata,
    response: ResponsePolicy,
    authentication: AuthenticationScopePolicy<'static>,
    raw_response: RawResponsePolicy<'static>,
    body_replayability: BodyReplayability,
}

impl From<&ValidatedAssociationPolicy> for RequestAssemblyPolicy {
    fn from(policy: &ValidatedAssociationPolicy) -> Self {
        Self {
            operation_id: policy.operation_id,
            method: policy.method,
            request_shape: policy.request_shape,
            service: policy.service,
            metadata: policy.metadata,
            response: policy.response,
            authentication: policy.authentication,
            raw_response: policy.raw_response,
            body_replayability: policy.body_replayability,
        }
    }
}

/// Marker for an operation without query parameters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoQuery;

impl QueryWire for NoQuery {
    fn write_query(self, _output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        Ok(0)
    }

    fn operation_key(self) -> &'static str {
        ""
    }
}

/// Marker for an operation without a request body.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoBody;

impl BodyWire for NoBody {
    fn write_body(self, _output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        Ok(0)
    }

    fn operation_key(self) -> &'static str {
        ""
    }
}

/// Provider-owned operation with a checked endpoint/query/body combination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HetznerPreparedOperation<E, Q = NoQuery, B = NoBody> {
    endpoint: E,
    query: Q,
    body: B,
    marker: PhantomData<fn()>,
}

impl<E> HetznerPreparedOperation<E> {
    /// Creates an operation that has no query parameters or request body.
    #[must_use]
    pub const fn endpoint(endpoint: E) -> Self {
        Self {
            endpoint,
            query: NoQuery,
            body: NoBody,
            marker: PhantomData,
        }
    }
}

impl<E, Q> HetznerPreparedOperation<E, Q> {
    /// Pairs an endpoint with its query request.
    ///
    /// Preparation rejects a query that is not source-locked to the endpoint
    /// operation before writing any request bytes.
    #[must_use]
    pub const fn query(endpoint: E, query: Q) -> Self {
        Self {
            endpoint,
            query,
            body: NoBody,
            marker: PhantomData,
        }
    }
}

impl<E, B> HetznerPreparedOperation<E, NoQuery, B> {
    /// Pairs an endpoint with its JSON request body.
    ///
    /// Preparation rejects a body that is not source-locked to the endpoint
    /// operation before writing any request bytes.
    #[must_use]
    pub const fn json(endpoint: E, body: B) -> Self {
        Self {
            endpoint,
            query: NoQuery,
            body,
            marker: PhantomData,
        }
    }
}

impl<E, Q, B> PrepareOperation for HetznerPreparedOperation<E, Q, B>
where
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    type Error = HetznerPreparationError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare_parts(self.endpoint, self.query, self.body, storage)
    }
}

pub(crate) fn prepare_parts<'storage, E, Q, B>(
    endpoint: E,
    query: Q,
    body: B,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, HetznerPreparationError>
where
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    let storage = clear_preparation_storage(storage);
    let metadata = endpoint.metadata()?;
    let profile = endpoint.response_profile();
    let response = response_policy(profile)?;
    let service = provider_service(endpoint.endpoint_group())?;
    let authentication = authentication_policy(service, endpoint.api_base_url())?;
    let raw_response = raw_response_policy(profile)?;
    let operation_id = OperationId::new(endpoint.operation_key())
        .map_err(HetznerPreparationError::InvalidOperationId)?;
    let policy = RequestAssemblyPolicy {
        operation_id,
        method: endpoint.method(),
        request_shape: endpoint.request_shape(),
        service,
        metadata,
        response,
        authentication,
        raw_response,
        body_replayability: BodyReplayability::Replayable,
    };
    prepare_parts_using_policy(endpoint, query, body, storage, &policy)
}

pub(crate) fn prepare_parts_with_policy<'storage, E, Q, B>(
    endpoint: E,
    query: Q,
    body: B,
    storage: PreparationStorage<'storage>,
    policy: &ValidatedAssociationPolicy,
) -> Result<PreparedRequest<'storage>, HetznerPreparationError>
where
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    let storage = clear_preparation_storage(storage);
    let assembly_policy = RequestAssemblyPolicy::from(policy);
    prepare_parts_using_policy(endpoint, query, body, storage, &assembly_policy)
}

pub(crate) fn clear_preparation_storage(storage: PreparationStorage<'_>) -> PreparationStorage<'_> {
    let (target, body) = storage.into_parts();
    cloud_sdk_sanitization::sanitize_bytes(target);
    cloud_sdk_sanitization::sanitize_bytes(body);
    PreparationStorage::new(target, body)
}

fn prepare_parts_using_policy<'storage, E, Q, B>(
    endpoint: E,
    query: Q,
    body: B,
    storage: PreparationStorage<'storage>,
    policy: &RequestAssemblyPolicy,
) -> Result<PreparedRequest<'storage>, HetznerPreparationError>
where
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    let (target_storage, body_storage) = storage.into_parts();
    validate_components(
        policy.request_shape,
        policy.operation_id.as_str(),
        query,
        body,
    )?;
    let (target_len, body_len) =
        match write_components(endpoint, query, body, target_storage, body_storage) {
            Ok(lengths) => lengths,
            Err(error) => {
                cloud_sdk_sanitization::sanitize_bytes(target_storage);
                cloud_sdk_sanitization::sanitize_bytes(body_storage);
                return Err(error);
            }
        };
    if body_len > body_storage.len() {
        cloud_sdk_sanitization::sanitize_bytes(target_storage);
        cloud_sdk_sanitization::sanitize_bytes(body_storage);
        return Err(HetznerPreparationError::Body);
    }
    if let Err(error) = validate_target_storage(target_storage, target_len) {
        cloud_sdk_sanitization::sanitize_bytes(target_storage);
        cloud_sdk_sanitization::sanitize_bytes(body_storage);
        return Err(error);
    }
    let body_bytes = body_storage
        .get(..body_len)
        .ok_or(HetznerPreparationError::Body)?;
    let target_text = target_storage
        .get(..target_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .ok_or(HetznerPreparationError::Path)?;
    let target = RequestTarget::new(target_text).map_err(HetznerPreparationError::InvalidTarget)?;
    let header_entries = if body_bytes.is_empty() {
        ACCEPT_JSON_HEADERS.as_slice()
    } else {
        JSON_REQUEST_HEADERS.as_slice()
    };
    let headers =
        RequestHeaders::new(header_entries).map_err(HetznerPreparationError::InvalidHeaders)?;
    let mut request = TransportRequest::new(policy.method, target).with_headers(headers);
    if !body_bytes.is_empty() {
        request = request.with_body(body_bytes);
    }
    PreparedRequest::new(
        request,
        policy.service,
        policy.metadata,
        policy.response,
        policy.authentication,
        policy.raw_response,
    )
    .map(|prepared| {
        let prepared = prepared.with_operation_id(policy.operation_id);
        match policy.body_replayability {
            BodyReplayability::NotReplayable => prepared,
            BodyReplayability::Replayable => prepared.with_replayable_body(),
        }
    })
    .map_err(HetznerPreparationError::InvalidPreparedPolicy)
}

fn validate_target_storage(storage: &[u8], len: usize) -> Result<(), HetznerPreparationError> {
    let text = storage
        .get(..len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .ok_or(HetznerPreparationError::Path)?;
    RequestTarget::new(text)
        .map(|_| ())
        .map_err(HetznerPreparationError::InvalidTarget)
}

fn write_components<E, Q, B>(
    endpoint: E,
    query: Q,
    body: B,
    target_storage: &mut [u8],
    body_storage: &mut [u8],
) -> Result<(usize, usize), HetznerPreparationError>
where
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    let path_len = endpoint.write_path(target_storage)?;
    let query_start = path_len
        .checked_add(1)
        .ok_or(HetznerPreparationError::Query)?;
    let query_output = target_storage
        .get_mut(query_start..)
        .ok_or(HetznerPreparationError::Query)?;
    let query_len = query.write_query(query_output)?;
    let target_len = if query_len == 0 {
        path_len
    } else {
        *target_storage
            .get_mut(path_len)
            .ok_or(HetznerPreparationError::Query)? = b'?';
        query_start
            .checked_add(query_len)
            .ok_or(HetznerPreparationError::Query)?
    };
    let body_len = body.write_body(body_storage)?;
    Ok((target_len, body_len))
}

fn validate_components<Q, B>(
    request_shape: RequestShape,
    operation_key: &str,
    query: Q,
    body: B,
) -> Result<(), HetznerPreparationError>
where
    Q: QueryWire,
    B: BodyWire,
{
    let has_query = !query.operation_key().is_empty();
    let has_body = !body.operation_key().is_empty();
    if has_query && !query.accepts_operation(operation_key)
        || has_body && !body.accepts_operation(operation_key)
    {
        return Err(HetznerPreparationError::OperationMismatch);
    }
    match (request_shape, has_query, has_body) {
        (RequestShape::None, true, _) => Err(HetznerPreparationError::UnexpectedQuery),
        (RequestShape::None | RequestShape::OptionalQuery, _, true) => {
            Err(HetznerPreparationError::UnexpectedBody)
        }
        (RequestShape::RequiredQuery, false, _) => Err(HetznerPreparationError::MissingQuery),
        (RequestShape::RequiredQuery, _, true) => Err(HetznerPreparationError::UnexpectedBody),
        (RequestShape::RequiredJson, true, _) => Err(HetznerPreparationError::UnexpectedQuery),
        (RequestShape::RequiredJson, _, false) => Err(HetznerPreparationError::MissingBody),
        _ => Ok(()),
    }
}

pub(crate) fn response_policy(
    profile: ResponseProfile,
) -> Result<ResponsePolicy, HetznerPreparationError> {
    let (statuses, content_type, body, max) = match profile {
        ResponseProfile::JsonOk => (
            STATUS_OK,
            ContentTypePolicy::Required(JSON_MEDIA),
            ResponseBodyPolicy::Required,
            MAX_JSON_RESPONSE_BYTES,
        ),
        ResponseProfile::JsonCreated => (
            STATUS_CREATED,
            ContentTypePolicy::Required(JSON_MEDIA),
            ResponseBodyPolicy::Required,
            MAX_JSON_RESPONSE_BYTES,
        ),
        ResponseProfile::NoContent => (
            STATUS_NO_CONTENT,
            ContentTypePolicy::Forbidden,
            ResponseBodyPolicy::Forbidden,
            0,
        ),
    };
    ResponsePolicy::new(statuses, content_type, body, max)
        .map_err(HetznerPreparationError::InvalidResponsePolicy)
}

pub(crate) fn operation_metadata(
    class: OperationClass,
    cost: CostIntent,
) -> Result<OperationMetadata, HetznerPreparationError> {
    let (impact, semantics, retry) = match class {
        OperationClass::ReadOnly => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        OperationClass::IdempotentMutation => (
            OperationImpact::Mutation,
            RequestSemantics::Idempotent,
            RetryEligibility::ExplicitPolicy,
        ),
        OperationClass::NonIdempotentMutation => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        OperationClass::IdempotentDestructive => (
            OperationImpact::Destructive,
            RequestSemantics::Idempotent,
            RetryEligibility::Never,
        ),
        OperationClass::NonIdempotentDestructive => (
            OperationImpact::Destructive,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
    };
    OperationMetadata::new(impact, semantics, retry, cost, RequestIdPolicy::Protected)
        .map_err(HetznerPreparationError::InvalidMetadata)
}

#[cfg(test)]
mod tests;
