use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{write_byte, write_str};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparationStorage, PrepareOperation, PreparedRequest, ProviderService, RequestBodySensitivity,
    RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    ContentType, HeaderName, MAX_INFORMATIONAL_RESPONSES, MediaType, RawResponsePolicy,
    RequestHeader, RequestHeaders, RequestTarget, ResponseMediaPolicy, StatusCode,
    TransportRequest,
};
use cloud_sdk_sanitization::sanitize_bytes;

#[cfg(feature = "serde")]
use super::RobotResetExecuteRequest;
use super::{RobotResetGetRequest, RobotResetListRequest, RobotResetRequestError};
use crate::endpoint::{official_robot_endpoint_identity, official_robot_endpoint_policy};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};
use crate::robot::RobotServerNumber;
#[cfg(feature = "serde")]
use crate::robot::{RobotForm, RobotFormField};

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const MAX_SUCCESS_BYTES: usize = 8_388_608;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
enum Operation<'a> {
    List,
    Get(&'a RobotServerNumber),
    #[cfg(feature = "serde")]
    Execute(&'a RobotResetExecuteRequest<'a>),
}

impl PrepareOperation for RobotResetListRequest {
    type Error = RobotResetRequestError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Operation::List, storage)
    }
}

impl PrepareOperation for RobotResetGetRequest {
    type Error = RobotResetRequestError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Operation::Get(&self.number), storage)
    }
}

#[cfg(feature = "serde")]
impl PrepareOperation for RobotResetExecuteRequest<'_> {
    type Error = RobotResetRequestError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Operation::Execute(self), storage)
    }
}

fn prepare<'storage>(
    operation: Operation<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotResetRequestError> {
    let (target_storage, body_storage) = storage.into_parts();
    sanitize_bytes(target_storage);
    sanitize_bytes(body_storage);
    prepare_inner(operation, target_storage, body_storage)
}

fn prepare_inner<'storage>(
    operation: Operation<'_>,
    target_storage: &'storage mut [u8],
    body_storage: &'storage mut [u8],
) -> Result<PreparedRequest<'storage>, RobotResetRequestError> {
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
        official_robot_endpoint_policy().map_err(RobotResetRequestError::InvalidEndpoint)
    );
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotResetRequestError::InvalidEndpoint)
    );
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(endpoint_identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let metadata = admitted!(metadata(operation));
    let response = admitted!(
        ResponsePolicy::new(
            OK,
            ContentTypePolicy::Required(JSON),
            ResponseBodyPolicy::Required,
            MAX_SUCCESS_BYTES,
        )
        .map_err(RobotResetRequestError::InvalidResponsePolicy)
    );
    let content =
        admitted!(HeaderName::new("content-type").map_err(RobotResetRequestError::InvalidHeaders));
    let raw = admitted!(
        RawResponsePolicy::new(
            MAX_SUCCESS_BYTES,
            crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
            ResponseMediaPolicy::Required(JSON),
            ResponseMediaPolicy::Optional(JSON),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotResetRequestError::InvalidRawPolicy)
    );
    let operation_id = admitted!(
        OperationId::new(operation.id()).map_err(RobotResetRequestError::InvalidOperationId)
    );

    let body_len = match operation {
        #[cfg(feature = "serde")]
        Operation::Execute(request) => {
            let field = admitted!(
                RobotFormField::sensitive("type", request.intent.reset_type().wire())
                    .map_err(RobotResetRequestError::Form)
            );
            let form = admitted!(
                RobotForm::new(core::slice::from_ref(&field)).map_err(RobotResetRequestError::Form)
            );
            admitted!(
                form.write_prepared(body_storage)
                    .map_err(RobotResetRequestError::Form)
            )
        }
        Operation::List | Operation::Get(_) => 0,
    };
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotResetRequestError::InvalidHeaders)
    );
    let path_len = admitted!(write_path(operation, target_storage));
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotResetRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot reset target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot reset target became invalid")
    };
    let mut request = TransportRequest::new(operation.method(), target).with_headers(headers);
    if body_len != 0 {
        let Some(body) = body_storage.get(..body_len) else {
            unreachable!("validated Robot reset form exceeded storage")
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
        if body_len == 0 {
            RequestBodySensitivity::Public
        } else {
            RequestBodySensitivity::Sensitive
        },
    )
    .map_err(RobotResetRequestError::InvalidPreparedPolicy)?;
    Ok(prepared
        .with_operation_id(operation_id)
        .with_replayable_body())
}

impl<'a> Operation<'a> {
    const fn number(self) -> Option<&'a RobotServerNumber> {
        match self {
            Self::List => None,
            Self::Get(number) => Some(number),
            #[cfg(feature = "serde")]
            Self::Execute(request) => Some(request.number()),
        }
    }

    const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) => Method::Get,
            #[cfg(feature = "serde")]
            Self::Execute(_) => Method::Post,
        }
    }

    const fn id(self) -> &'static str {
        match self {
            Self::List => "robot_list_resets",
            Self::Get(_) => "robot_get_reset",
            #[cfg(feature = "serde")]
            Self::Execute(_) => "robot_execute_reset",
        }
    }
}

fn metadata(operation: Operation<'_>) -> Result<OperationMetadata, RobotResetRequestError> {
    let (impact, semantics, retry) = match operation {
        Operation::List | Operation::Get(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        #[cfg(feature = "serde")]
        Operation::Execute(_) => (
            OperationImpact::Destructive,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
    };
    OperationMetadata::new(
        impact,
        semantics,
        retry,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .map_err(RobotResetRequestError::InvalidMetadata)
}

fn write_path(
    operation: Operation<'_>,
    output: &mut [u8],
) -> Result<usize, RobotResetRequestError> {
    let mut len = 0;
    write_str(output, &mut len, "/reset", RobotResetRequestError::Path)?;
    if let Some(number) = operation.number() {
        write_str(output, &mut len, "/", RobotResetRequestError::Path)?;
        number.with_decimal_bytes(|digits| {
            for digit in digits {
                write_byte(output, &mut len, *digit, RobotResetRequestError::Path)?;
            }
            Ok(())
        })?;
    }
    Ok(len)
}
