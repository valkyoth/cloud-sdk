use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{write_str, write_u64};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationIdError, OperationImpact,
    OperationMetadata, OperationMetadataError, PreparationStorage, PrepareOperation,
    PreparedRequest, PreparedRequestPolicyError, ProviderService, RequestBodySensitivity,
    RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy,
    ResponsePolicyValidationError, RetryEligibility,
};
use cloud_sdk::transport::{
    ContentType, HeaderError, HeaderName, MAX_INFORMATIONAL_RESPONSES, MediaType,
    RawResponsePolicy, RawResponsePolicyError, RequestHeader, RequestHeaders, RequestTarget,
    RequestTargetError, ResponseMediaPolicy, StatusCode, TransportRequest,
};
use cloud_sdk_sanitization::sanitize_bytes;

use crate::endpoint::{
    OfficialEndpointError, official_robot_endpoint_identity, official_robot_endpoint_policy,
};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};
use crate::robot::{RobotForm, RobotFormError, RobotFormField};

use super::identity::RobotServerNumber;

const JSON_MEDIA: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const MAX_SUCCESS_BYTES: usize = 8_388_608;
const MAX_ERROR_BYTES: usize = super::super::MAX_ROBOT_ERROR_BODY_BYTES;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

/// Maximum bytes admitted for a Robot server name.
pub const MAX_ROBOT_SERVER_NAME_BYTES: usize = 63;

/// Failure while validating or preparing a Robot server operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotServerRequestError {
    /// A requested server name violates the bounded hostname grammar.
    InvalidServerName,
    /// Caller-owned path storage was too small or path encoding failed.
    Path,
    /// Form validation or encoding failed.
    Form(RobotFormError),
    /// The constructed origin-form request target was rejected.
    InvalidTarget(RequestTargetError),
    /// Source-locked request headers were rejected.
    InvalidHeaders(HeaderError),
    /// The official Robot endpoint policy was invalid.
    InvalidEndpoint(OfficialEndpointError),
    /// A source-locked operation identifier was invalid.
    InvalidOperationId(OperationIdError),
    /// Operation safety metadata was internally inconsistent.
    InvalidMetadata(OperationMetadataError),
    /// The success-response policy was internally inconsistent.
    InvalidResponsePolicy(ResponsePolicyValidationError),
    /// The raw response-wire policy was internally inconsistent.
    InvalidRawPolicy(RawResponsePolicyError),
    /// Cross-policy prepared-request validation failed.
    InvalidPreparedPolicy(PreparedRequestPolicyError),
}

impl core::fmt::Display for RobotServerRequestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidServerName => "Robot server name is invalid",
            Self::Path => "Robot server path preparation failed",
            Self::Form(_) => "Robot server form preparation failed",
            Self::InvalidTarget(_) => "Robot server target is invalid",
            Self::InvalidHeaders(_) => "Robot server headers are invalid",
            Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
            Self::InvalidOperationId(_) => "Robot server operation identifier is invalid",
            Self::InvalidMetadata(_) => "Robot server metadata is invalid",
            Self::InvalidResponsePolicy(_) => "Robot server response policy is invalid",
            Self::InvalidRawPolicy(_) => "Robot server raw response policy is invalid",
            Self::InvalidPreparedPolicy(_) => "Robot server prepared policy is invalid",
        })
    }
}

impl core::error::Error for RobotServerRequestError {}

/// Validated Robot server name used only through explicit rename intent.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotServerName<'a>(&'a str);

impl<'a> RobotServerName<'a> {
    /// Creates a bounded conservative hostname.
    pub fn new(value: &'a str) -> Result<Self, RobotServerRequestError> {
        if value.is_empty()
            || value.len() > MAX_ROBOT_SERVER_NAME_BYTES
            || value.starts_with('-')
            || value.ends_with('-')
            || value
                .bytes()
                .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'-'))
        {
            return Err(RobotServerRequestError::InvalidServerName);
        }
        Ok(Self(value))
    }
    /// Returns the validated name.
    #[must_use]
    pub const fn as_str(&self) -> &'a str {
        self.0
    }
}

impl core::fmt::Debug for RobotServerName<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotServerName([redacted])")
    }
}

/// Explicit state change admitted by the server update endpoint.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RobotServerUpdateIntent<'a> {
    /// Replace the server name.
    Rename(RobotServerName<'a>),
}

impl core::fmt::Debug for RobotServerUpdateIntent<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotServerUpdateIntent([redacted])")
    }
}

/// Lists all Robot servers.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct RobotServerListRequest;
impl RobotServerListRequest {
    /// Creates the bodyless list request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl core::fmt::Debug for RobotServerListRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotServerListRequest")
    }
}

/// Gets one server by canonical positive number.
#[derive(Eq, PartialEq)]
pub struct RobotServerGetRequest {
    number: RobotServerNumber,
}
impl RobotServerGetRequest {
    /// Creates a canonical server-number request.
    #[must_use]
    pub const fn new(number: RobotServerNumber) -> Self {
        Self { number }
    }
    /// Returns the requested server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }
}

impl core::fmt::Debug for RobotServerGetRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotServerGetRequest([redacted])")
    }
}

/// Renames one server by canonical positive number.
#[derive(Eq, PartialEq)]
pub struct RobotServerUpdateRequest<'a> {
    number: RobotServerNumber,
    intent: RobotServerUpdateIntent<'a>,
}
impl<'a> RobotServerUpdateRequest<'a> {
    /// Creates an update request with explicit intent.
    #[must_use]
    pub const fn new(number: RobotServerNumber, intent: RobotServerUpdateIntent<'a>) -> Self {
        Self { number, intent }
    }
    /// Creates a rename request.
    #[must_use]
    pub const fn rename(number: RobotServerNumber, name: RobotServerName<'a>) -> Self {
        Self::new(number, RobotServerUpdateIntent::Rename(name))
    }
    /// Returns the canonical target number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }
    /// Returns the explicit update intent.
    #[must_use]
    pub const fn intent(&self) -> RobotServerUpdateIntent<'a> {
        self.intent
    }
}

impl core::fmt::Debug for RobotServerUpdateRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotServerUpdateRequest([redacted])")
    }
}

impl PrepareOperation for RobotServerListRequest {
    type Error = RobotServerRequestError;
    fn prepare<'s>(
        &self,
        storage: PreparationStorage<'s>,
    ) -> Result<PreparedRequest<'s>, Self::Error> {
        prepare(Operation::List, None, storage)
    }
}
impl PrepareOperation for RobotServerGetRequest {
    type Error = RobotServerRequestError;
    fn prepare<'s>(
        &self,
        storage: PreparationStorage<'s>,
    ) -> Result<PreparedRequest<'s>, Self::Error> {
        prepare(Operation::Get(&self.number), None, storage)
    }
}
impl PrepareOperation for RobotServerUpdateRequest<'_> {
    type Error = RobotServerRequestError;
    fn prepare<'s>(
        &self,
        storage: PreparationStorage<'s>,
    ) -> Result<PreparedRequest<'s>, Self::Error> {
        prepare(Operation::Update(&self.number), Some(self.intent), storage)
    }
}

#[derive(Clone, Copy)]
enum Operation<'a> {
    List,
    Get(&'a RobotServerNumber),
    Update(&'a RobotServerNumber),
}

fn prepare<'s>(
    operation: Operation<'_>,
    intent: Option<RobotServerUpdateIntent<'_>>,
    storage: PreparationStorage<'s>,
) -> Result<PreparedRequest<'s>, RobotServerRequestError> {
    let (target_storage, body_storage) = storage.into_parts();
    sanitize_bytes(target_storage);
    sanitize_bytes(body_storage);
    macro_rules! admitted {
        ($result:expr) => {
            match $result {
                Ok(value) => value,
                Err(error) => {
                    sanitize_bytes(target_storage);
                    sanitize_bytes(body_storage);
                    return Err(error);
                }
            }
        };
    }

    let endpoint = admitted!(
        official_robot_endpoint_policy().map_err(RobotServerRequestError::InvalidEndpoint)
    );
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotServerRequestError::InvalidEndpoint)
    );
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(endpoint_identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let metadata = admitted!(operation.metadata());
    let response = admitted!(
        ResponsePolicy::new(
            OK,
            ContentTypePolicy::Required(JSON_MEDIA),
            ResponseBodyPolicy::Required,
            MAX_SUCCESS_BYTES,
        )
        .map_err(RobotServerRequestError::InvalidResponsePolicy)
    );
    let content =
        admitted!(HeaderName::new("content-type").map_err(RobotServerRequestError::InvalidHeaders));
    let raw = admitted!(
        RawResponsePolicy::new(
            MAX_SUCCESS_BYTES,
            MAX_ERROR_BYTES,
            ResponseMediaPolicy::Required(JSON_MEDIA),
            ResponseMediaPolicy::Optional(JSON_MEDIA),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotServerRequestError::InvalidRawPolicy)
    );
    let operation_id = admitted!(
        OperationId::new(operation.id()).map_err(RobotServerRequestError::InvalidOperationId)
    );
    let body_len = match intent {
        None => 0,
        Some(RobotServerUpdateIntent::Rename(name)) => {
            let field = admitted!(
                RobotFormField::public("server_name", name.as_str())
                    .map_err(RobotServerRequestError::Form)
            );
            let form = admitted!(
                RobotForm::new(core::slice::from_ref(&field))
                    .map_err(RobotServerRequestError::Form)
            );
            admitted!(
                form.write_prepared(body_storage)
                    .map_err(RobotServerRequestError::Form)
            )
        }
    };
    let headers = RequestHeaders::new(if body_len == 0 {
        &ACCEPT
    } else {
        &FORM_HEADERS
    })
    .map_err(RobotServerRequestError::InvalidHeaders);
    let headers = admitted!(headers);
    let path_len = admitted!(write_path(operation, target_storage));
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotServerRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot target became invalid")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot target became noncanonical")
    };
    let mut request = TransportRequest::new(operation.method(), target).with_headers(headers);
    if body_len != 0 {
        let Some(body) = body_storage.get(..body_len) else {
            unreachable!("validated Robot form length exceeded storage")
        };
        request = request.with_body(body);
    }
    let prepared = PreparedRequest::new(
        request,
        service,
        metadata,
        response,
        authentication,
        raw,
        RequestBodySensitivity::Public,
    )
    .map_err(RobotServerRequestError::InvalidPreparedPolicy)?;
    Ok(prepared
        .with_operation_id(operation_id)
        .with_replayable_body())
}

impl Operation<'_> {
    const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) => Method::Get,
            Self::Update(_) => Method::Post,
        }
    }
    const fn id(self) -> &'static str {
        match self {
            Self::List => "robot_list_servers",
            Self::Get(_) => "robot_get_server",
            Self::Update(_) => "robot_update_server",
        }
    }
    fn metadata(self) -> Result<OperationMetadata, RobotServerRequestError> {
        let (impact, semantics, retry) = match self {
            Self::List | Self::Get(_) => (
                OperationImpact::ReadOnly,
                RequestSemantics::Safe,
                RetryEligibility::ExplicitPolicy,
            ),
            Self::Update(_) => (
                OperationImpact::Mutation,
                RequestSemantics::Idempotent,
                RetryEligibility::ExplicitPolicy,
            ),
        };
        OperationMetadata::new(
            impact,
            semantics,
            retry,
            CostIntent::NoKnownCost,
            RequestIdPolicy::Discard,
        )
        .map_err(RobotServerRequestError::InvalidMetadata)
    }
}

fn write_path(
    operation: Operation<'_>,
    output: &mut [u8],
) -> Result<usize, RobotServerRequestError> {
    let mut len = 0;
    write_str(output, &mut len, "/server", RobotServerRequestError::Path)?;
    if let Operation::Get(number) | Operation::Update(number) = operation {
        write_str(output, &mut len, "/", RobotServerRequestError::Path)?;
        write_u64(
            output,
            &mut len,
            number.value(),
            RobotServerRequestError::Path,
        )?;
    }
    Ok(len)
}
