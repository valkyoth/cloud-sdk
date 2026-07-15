//! Generic operation assembly over sealed provider wire components.

use core::marker::PhantomData;

use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, PreparationStorage,
    PrepareOperation, PreparedRequest, ProviderService, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    ContentType, EndpointIdentity, EndpointScheme, MediaType, RequestTarget, StatusCode,
    TransportRequest,
};
use cloud_sdk::{ApiFamily, Method, Provider};

use crate::request::ApiBaseUrl;

use super::HetznerPreparationError;

const JSON_MEDIA: &[MediaType<'static>] = &[MediaType::JSON];
const STATUS_OK: &[StatusCode] = &[StatusCode::OK];
const STATUS_CREATED: &[StatusCode] = &[StatusCode::CREATED];
const STATUS_NO_CONTENT: &[StatusCode] = &[StatusCode::NO_CONTENT];
const MAX_JSON_RESPONSE_BYTES: usize = 8_388_608;

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

pub(crate) trait EndpointWire: Copy {
    fn method(self) -> Method;
    fn api_base_url(self) -> ApiBaseUrl;
    fn write_path(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError>;
    fn request_shape(self) -> RequestShape;
    fn response_profile(self) -> ResponseProfile;
    fn metadata(self) -> Result<OperationMetadata, HetznerPreparationError>;
    fn operation_key(self) -> &'static str;
}

pub(crate) trait QueryWire: Copy {
    fn write_query(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError>;
    fn operation_key(self) -> &'static str;
}

pub(crate) trait BodyWire: Copy {
    fn write_body(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError>;
    fn operation_key(self) -> &'static str;
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
    pub(crate) const fn endpoint(endpoint: E) -> Self {
        Self {
            endpoint,
            query: NoQuery,
            body: NoBody,
            marker: PhantomData,
        }
    }
}

impl<E, Q> HetznerPreparedOperation<E, Q> {
    pub(crate) const fn query(endpoint: E, query: Q) -> Self {
        Self {
            endpoint,
            query,
            body: NoBody,
            marker: PhantomData,
        }
    }
}

impl<E, B> HetznerPreparedOperation<E, NoQuery, B> {
    pub(crate) const fn json(endpoint: E, body: B) -> Self {
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
    let (target_storage, body_storage) = storage.into_parts();
    target_storage.fill(0);
    body_storage.fill(0);
    let metadata = endpoint.metadata()?;
    let policy = response_policy(endpoint.response_profile())?;
    let service = provider_service(endpoint.api_base_url())?;
    if let Err(error) = validate_components(endpoint, query, body) {
        return Err(error);
    }
    let (target_len, body_len) =
        match write_components(endpoint, query, body, target_storage, body_storage) {
            Ok(lengths) => lengths,
            Err(error) => {
                target_storage.fill(0);
                body_storage.fill(0);
                return Err(error);
            }
        };
    if body_len > body_storage.len() {
        target_storage.fill(0);
        body_storage.fill(0);
        return Err(HetznerPreparationError::Body);
    }
    if let Err(error) = validate_target_storage(target_storage, target_len) {
        target_storage.fill(0);
        body_storage.fill(0);
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
    let mut request = TransportRequest::new(endpoint.method(), target);
    if !body_bytes.is_empty() {
        request = request
            .with_body(body_bytes)
            .with_content_type(ContentType::JSON);
    }
    Ok(PreparedRequest::new(request, service, metadata, policy))
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

fn validate_components<E, Q, B>(
    endpoint: E,
    query: Q,
    body: B,
) -> Result<(), HetznerPreparationError>
where
    E: EndpointWire,
    Q: QueryWire,
    B: BodyWire,
{
    let has_query = !query.operation_key().is_empty();
    let has_body = !body.operation_key().is_empty();
    let key = endpoint.operation_key();
    if has_query && query.operation_key() != key || has_body && body.operation_key() != key {
        return Err(HetznerPreparationError::OperationMismatch);
    }
    match (endpoint.request_shape(), has_query, has_body) {
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

fn provider_service(base: ApiBaseUrl) -> Result<ProviderService, HetznerPreparationError> {
    let (family, host) = match base {
        ApiBaseUrl::CloudV1 => (ApiFamily::Cloud, "api.hetzner.cloud"),
        ApiBaseUrl::HetznerV1 => (ApiFamily::Storage, "api.hetzner.com"),
    };
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, host, 443, "/v1")
        .map_err(HetznerPreparationError::InvalidOfficialEndpoint)?;
    Ok(ProviderService::new(Provider::Hetzner, family, endpoint))
}

fn response_policy(profile: ResponseProfile) -> Result<ResponsePolicy, HetznerPreparationError> {
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

pub(crate) fn method_metadata(
    method: Method,
    destructive: bool,
    cost: CostIntent,
) -> Result<OperationMetadata, HetznerPreparationError> {
    let (impact, semantics, retry) = match method {
        Method::Get => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Method::Put => (
            if destructive {
                OperationImpact::Destructive
            } else {
                OperationImpact::Mutation
            },
            RequestSemantics::Idempotent,
            RetryEligibility::ExplicitPolicy,
        ),
        Method::Delete => (
            OperationImpact::Destructive,
            RequestSemantics::Idempotent,
            RetryEligibility::Never,
        ),
        Method::Post => (
            if destructive {
                OperationImpact::Destructive
            } else {
                OperationImpact::Mutation
            },
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
    };
    OperationMetadata::new(impact, semantics, retry, cost)
        .map_err(HetznerPreparationError::InvalidMetadata)
}
