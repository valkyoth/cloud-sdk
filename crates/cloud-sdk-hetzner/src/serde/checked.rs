//! Policy-bound checked Hetzner response decoding.

use alloc::vec::Vec;
use core::fmt;

use cloud_sdk::Provider;
use cloud_sdk::operation::{PreparedRequest, ResponsePolicyError};
use cloud_sdk::rate_limit::RateLimit;
use cloud_sdk::transport::{MediaType, TransportResponse};

use super::binding::{ResponseBinding, ResponseShape, find};
use super::models::{
    CompositeResult, HetznerSuccess, NamedSensitiveText, ResponseModelError, SensitiveText, object,
    parse_action, parse_actions, parse_folders, parse_metrics, parse_pagination, parse_pricing,
    parse_resource, parse_resources, parse_zonefile, required, value_text,
};
use super::strict_json;
use super::strict_json::{Map, Value};
use super::{MAX_SERDE_RESPONSE_BYTES, ResponseBytes, ResponseSizeError};
use crate::response::ApiErrorCode;

/// Typed provider error returned by a checked operation response.
#[derive(Clone, Eq, PartialEq)]
pub struct HetznerApiError {
    code: ApiErrorCode,
    message: SensitiveText,
}

impl HetznerApiError {
    /// Returns the classified provider error code.
    #[must_use]
    pub const fn code(&self) -> ApiErrorCode {
        self.code
    }

    /// Runs a closure with temporary access to the provider error message.
    pub fn try_with_message<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.message.try_with_secret(inspect)
    }
}

impl fmt::Debug for HetznerApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HetznerApiError")
            .field("code", &self.code)
            .field("message", &"[redacted]")
            .finish()
    }
}

impl fmt::Display for HetznerApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Hetzner API returned an error response")
    }
}

impl core::error::Error for HetznerApiError {}

/// Failure from the checked decoder. Diagnostics never contain response data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HetznerDecodeError {
    /// The prepared request has no provider operation identifier.
    MissingOperationId,
    /// The operation is absent from the source-locked active response table.
    UnknownOperation,
    /// The prepared service does not match the operation's API family.
    ServiceMismatch,
    /// The response failed the prepared success policy.
    ResponsePolicy(ResponsePolicyError),
    /// The response exceeds the parser boundary.
    ResponseSize(ResponseSizeError),
    /// An error status omitted or supplied an invalid JSON content type.
    ErrorContentType,
    /// An error status omitted its required body.
    MissingErrorBody,
    /// The JSON document is malformed, duplicated, too deep, or too large.
    MalformedPayload,
    /// Parsed data failed a resource-specific model invariant.
    Model(ResponseModelError),
    /// The provider returned a validated API error envelope.
    Provider(HetznerApiError),
}

impl fmt::Display for HetznerDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingOperationId => "prepared request has no operation identifier",
            Self::UnknownOperation => "prepared request operation is not source-locked",
            Self::ServiceMismatch => "prepared request service does not match the operation",
            Self::ResponsePolicy(_) => "Hetzner success response failed its prepared policy",
            Self::ResponseSize(_) => "Hetzner response exceeds the parser size limit",
            Self::ErrorContentType => "Hetzner error response content type is invalid",
            Self::MissingErrorBody => "Hetzner error response body is missing",
            Self::MalformedPayload => "Hetzner response JSON is malformed",
            Self::Model(_) => "Hetzner response model validation failed",
            Self::Provider(_) => "Hetzner API returned a validated error response",
        })
    }
}

impl core::error::Error for HetznerDecodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ResponsePolicy(error) => Some(error),
            Self::ResponseSize(error) => Some(error),
            Self::Model(error) => Some(error),
            Self::Provider(error) => Some(error),
            _ => None,
        }
    }
}

/// Successful checked response plus validated rate-limit metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedHetznerResponse {
    success: HetznerSuccess,
    rate_limit: Option<RateLimit>,
}

impl CheckedHetznerResponse {
    /// Returns the typed operation success value.
    #[must_use]
    pub const fn success(&self) -> &HetznerSuccess {
        &self.success
    }

    /// Returns validated transport rate-limit metadata when supplied.
    #[must_use]
    pub const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }

    /// Consumes the wrapper and returns the typed success value.
    #[must_use]
    pub fn into_success(self) -> HetznerSuccess {
        self.success
    }
}

/// Decodes one transport response through its exact prepared operation policy.
pub fn decode_response(
    prepared: PreparedRequest<'_>,
    response: TransportResponse<'_>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    let operation = prepared
        .operation_id()
        .ok_or(HetznerDecodeError::MissingOperationId)?;
    let binding = find(operation.as_str()).ok_or(HetznerDecodeError::UnknownOperation)?;
    let service = prepared.service();
    if service.provider() != Provider::Hetzner || service.family() != binding.family {
        return Err(HetznerDecodeError::ServiceMismatch);
    }
    if response.status().is_error() {
        return match decode_provider_error(response) {
            Ok(error) => Err(HetznerDecodeError::Provider(error)),
            Err(error) => Err(error),
        };
    }
    let checked = prepared
        .validate_response(response)
        .map_err(HetznerDecodeError::ResponsePolicy)?;
    if checked.status().get() != binding.status {
        return Err(HetznerDecodeError::ResponsePolicy(
            ResponsePolicyError::UnexpectedStatus,
        ));
    }
    let success = if binding.shape == ResponseShape::Empty {
        HetznerSuccess::Empty
    } else {
        let bytes = ResponseBytes::new(checked.body()).map_err(HetznerDecodeError::ResponseSize)?;
        let mut value = strict_json::parse(bytes.as_slice())
            .map_err(|_| HetznerDecodeError::MalformedPayload)?;
        decode_success(binding, &mut value).map_err(HetznerDecodeError::Model)?
    };
    Ok(CheckedHetznerResponse {
        success,
        rate_limit: checked.rate_limit(),
    })
}

fn decode_provider_error(
    response: TransportResponse<'_>,
) -> Result<HetznerApiError, HetznerDecodeError> {
    if response.body().is_empty() {
        return Err(HetznerDecodeError::MissingErrorBody);
    }
    if response.body().len() > MAX_SERDE_RESPONSE_BYTES {
        return Err(HetznerDecodeError::ResponseSize(
            ResponseSizeError::TooLarge,
        ));
    }
    let content_type = response
        .content_type()
        .ok_or(HetznerDecodeError::ErrorContentType)?;
    if !content_type.matches(MediaType::JSON) {
        return Err(HetznerDecodeError::ErrorContentType);
    }
    let mut value =
        strict_json::parse(response.body()).map_err(|_| HetznerDecodeError::MalformedPayload)?;
    let envelope = object_mut(&mut value).map_err(HetznerDecodeError::Model)?;
    let error = object_mut(required_mut(envelope, "error").map_err(HetznerDecodeError::Model)?)
        .map_err(HetznerDecodeError::Model)?;
    let code = value_text(
        required(error, "code").map_err(HetznerDecodeError::Model)?,
        128,
    )
    .map_err(HetznerDecodeError::Model)?;
    let message = required_mut(error, "message")
        .map_err(HetznerDecodeError::Model)?
        .take_string()
        .map(SensitiveText::new)
        .ok_or(HetznerDecodeError::Model(ResponseModelError::WrongType))?;
    message
        .validate(16_384)
        .map_err(HetznerDecodeError::Model)?;
    Ok(HetznerApiError {
        code: ApiErrorCode::from_api_str(&code),
        message,
    })
}

fn decode_success(
    binding: ResponseBinding,
    value: &mut Value,
) -> Result<HetznerSuccess, ResponseModelError> {
    {
        let envelope = object(value)?;
        validate_required(envelope, binding.required)?;
    }
    if binding.shape == ResponseShape::Composite {
        return decode_composite(binding, object_mut(value)?);
    }
    if binding.shape == ResponseShape::ZoneFile {
        let envelope = object_mut(value)?;
        return parse_zonefile(required_mut(envelope, "zonefile")?).map(HetznerSuccess::ZoneFile);
    }
    match binding.shape {
        ResponseShape::Empty => Ok(HetznerSuccess::Empty),
        ResponseShape::Action => {
            parse_action(required_mut(object_mut(value)?, "action")?).map(HetznerSuccess::Action)
        }
        ResponseShape::Actions | ResponseShape::ActionsPage => {
            let envelope = object_mut(value)?;
            let actions = parse_actions(required_mut(envelope, "actions")?)?;
            let pagination = if binding.shape == ResponseShape::ActionsPage {
                Some(parse_pagination(required(envelope, "meta")?)?)
            } else {
                None
            };
            Ok(HetznerSuccess::Actions {
                actions,
                pagination,
            })
        }
        ResponseShape::Resource | ResponseShape::ResourceList | ResponseShape::ResourcePage => {
            decode_resources(binding, object(value)?)
        }
        ResponseShape::Composite => Err(ResponseModelError::EnvelopeMismatch),
        ResponseShape::Metrics => {
            parse_metrics(required(object(value)?, "metrics")?).map(HetznerSuccess::Metrics)
        }
        ResponseShape::ZoneFile => Err(ResponseModelError::EnvelopeMismatch),
        ResponseShape::Pricing => {
            parse_pricing(required(object(value)?, "pricing")?).map(HetznerSuccess::Pricing)
        }
        ResponseShape::Folders => {
            parse_folders(required(object(value)?, "folders")?).map(HetznerSuccess::Folders)
        }
    }
}

fn decode_resources(
    binding: ResponseBinding,
    envelope: &Map,
) -> Result<HetznerSuccess, ResponseModelError> {
    let value = required(envelope, binding.root)?;
    if binding.shape == ResponseShape::Resource {
        return parse_resource(binding.root, value).map(HetznerSuccess::Resource);
    }
    let resources = parse_resources(binding.root, value)?;
    let pagination = if binding.shape == ResponseShape::ResourcePage {
        Some(parse_pagination(required(envelope, "meta")?)?)
    } else {
        None
    };
    Ok(HetznerSuccess::Resources {
        resources,
        pagination,
    })
}

fn decode_composite(
    binding: ResponseBinding,
    envelope: &mut Map,
) -> Result<HetznerSuccess, ResponseModelError> {
    let secrets = take_composite_secrets(envelope)?;
    let resource = if binding.root == "-" {
        None
    } else {
        envelope
            .get(binding.root)
            .map(|value| parse_resource(binding.root, value))
            .transpose()?
    };
    let mut actions = Vec::new();
    if let Some(value) = envelope.get_mut("action") {
        actions.push(parse_action(value)?);
    }
    for key in ["actions", "next_actions"] {
        if let Some(value) = envelope.get_mut(key) {
            actions.extend(parse_actions(value)?);
        }
    }
    Ok(HetznerSuccess::Composite(CompositeResult {
        resource,
        actions,
        secrets,
    }))
}

fn take_composite_secrets(
    envelope: &mut Map,
) -> Result<Vec<NamedSensitiveText>, ResponseModelError> {
    let mut secrets = Vec::new();
    for key in ["root_password", "password", "wss_url"] {
        if let Some(value) = envelope.get_mut(key) {
            if value.is_null() {
                continue;
            }
            let secret = value
                .take_string()
                .map(SensitiveText::new)
                .ok_or(ResponseModelError::WrongType)?;
            secret.validate(65_536)?;
            secrets.push(NamedSensitiveText::new(key, secret));
        }
    }
    Ok(secrets)
}

fn object_mut(value: &mut Value) -> Result<&mut Map, ResponseModelError> {
    value.as_object_mut().ok_or(ResponseModelError::WrongType)
}

fn required_mut<'a>(object: &'a mut Map, key: &str) -> Result<&'a mut Value, ResponseModelError> {
    object.get_mut(key).ok_or(ResponseModelError::MissingField)
}

fn validate_required(envelope: &Map, required_fields: &str) -> Result<(), ResponseModelError> {
    if required_fields == "-" {
        return Ok(());
    }
    for field in required_fields.split(',') {
        if !envelope.contains_key(field) {
            return Err(ResponseModelError::MissingField);
        }
    }
    Ok(())
}
