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
use super::{PreparedRobotWol, RobotWolSendRequest};
use super::{RobotWolGetRequest, RobotWolRequestError};
use crate::endpoint::{official_robot_endpoint_identity, official_robot_endpoint_policy};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};
use crate::robot::RobotServerNumber;

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
/// Maximum success-body bytes accepted for either WOL operation.
pub const MAX_ROBOT_WOL_RESPONSE_BYTES: usize = 16_384;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const EMPTY_FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
enum Operation<'a> {
    Get(&'a RobotServerNumber),
    #[cfg(feature = "serde")]
    Send(&'a RobotWolSendRequest<'a>),
}

impl PrepareOperation for RobotWolGetRequest {
    type Error = RobotWolRequestError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Operation::Get(&self.number), storage)
    }
}

fn prepare<'storage>(
    operation: Operation<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotWolRequestError> {
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

    let endpoint =
        admitted!(official_robot_endpoint_policy().map_err(RobotWolRequestError::InvalidEndpoint));
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotWolRequestError::InvalidEndpoint)
    );
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(endpoint_identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let metadata = admitted!(metadata(operation));
    let response = admitted!(
        ResponsePolicy::new(
            OK,
            ContentTypePolicy::Required(JSON),
            ResponseBodyPolicy::Required,
            MAX_ROBOT_WOL_RESPONSE_BYTES,
        )
        .map_err(RobotWolRequestError::InvalidResponsePolicy)
    );
    let content =
        admitted!(HeaderName::new("content-type").map_err(RobotWolRequestError::InvalidHeaders));
    let raw = admitted!(
        RawResponsePolicy::new(
            MAX_ROBOT_WOL_RESPONSE_BYTES,
            crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
            ResponseMediaPolicy::Required(JSON),
            ResponseMediaPolicy::Optional(JSON),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotWolRequestError::InvalidRawPolicy)
    );
    let operation_id = admitted!(
        OperationId::new(operation.id()).map_err(RobotWolRequestError::InvalidOperationId)
    );
    let headers = admitted!(
        RequestHeaders::new(if operation.is_send() {
            &EMPTY_FORM_HEADERS
        } else {
            &ACCEPT
        })
        .map_err(RobotWolRequestError::InvalidHeaders)
    );
    let path_len = admitted!(write_path(operation.number(), target_storage));
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotWolRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot WOL target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot WOL target became invalid")
    };
    let request = TransportRequest::new(operation.method(), target).with_headers(headers);
    let prepared = PreparedRequest::new(
        request,
        service,
        metadata,
        response,
        authentication,
        raw,
        RequestBodySensitivity::Public,
    )
    .map_err(RobotWolRequestError::InvalidPreparedPolicy)?;
    let prepared = prepared
        .with_operation_id(operation_id)
        .with_replayable_body();
    Ok(if operation.is_send() {
        prepared.with_required_authorization_evidence()
    } else {
        prepared
    })
}

#[cfg(feature = "serde")]
impl<'state> RobotWolSendRequest<'state> {
    /// Prepares WOL execution without erasing capability evidence.
    pub fn prepare_bound<'storage, 'request>(
        &'request self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRobotWol<'storage, 'request, Self>, RobotWolRequestError> {
        let inner = prepare(Operation::Send(self), storage)?;
        Ok(PreparedRobotWol {
            request: self,
            inner,
        })
    }
}

impl<'a> Operation<'a> {
    const fn number(self) -> &'a RobotServerNumber {
        match self {
            Self::Get(number) => number,
            #[cfg(feature = "serde")]
            Self::Send(request) => request.number(),
        }
    }

    const fn is_send(self) -> bool {
        match self {
            Self::Get(_) => false,
            #[cfg(feature = "serde")]
            Self::Send(_) => true,
        }
    }

    const fn method(self) -> Method {
        if self.is_send() {
            Method::Post
        } else {
            Method::Get
        }
    }

    const fn id(self) -> &'static str {
        if self.is_send() {
            "robot_send_wol"
        } else {
            "robot_get_wol"
        }
    }
}

fn metadata(operation: Operation<'_>) -> Result<OperationMetadata, RobotWolRequestError> {
    let (impact, semantics, retry) = if operation.is_send() {
        (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        )
    } else {
        (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        )
    };
    OperationMetadata::new(
        impact,
        semantics,
        retry,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .map_err(RobotWolRequestError::InvalidMetadata)
}

fn write_path(
    number: &RobotServerNumber,
    output: &mut [u8],
) -> Result<usize, RobotWolRequestError> {
    let mut len = 0;
    write_str(output, &mut len, "/wol/", RobotWolRequestError::Path)?;
    number.with_decimal_bytes(|digits| {
        for digit in digits {
            write_byte(output, &mut len, *digit, RobotWolRequestError::Path)?;
        }
        Ok(())
    })?;
    Ok(len)
}
