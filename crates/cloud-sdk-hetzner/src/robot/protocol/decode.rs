use alloc::vec::Vec;

use cloud_sdk::rate_limit::DelaySeconds;
use cloud_sdk::transport::{MediaType, ResponseDecodeWorkspace, StatusCode, TransportResponse};

use super::{
    MAX_ROBOT_ERROR_BODY_BYTES, MAX_ROBOT_ERROR_CODE_BYTES, MAX_ROBOT_ERROR_MESSAGE_BYTES,
    MAX_ROBOT_INPUT_FIELD_BYTES, MAX_ROBOT_INPUT_FIELDS, RobotDecodeError, RobotFailure,
    RobotInvalidInput, RobotProviderError, RobotProviderErrorCode, RobotQuota,
};
use crate::serde::SensitiveText;
use crate::serde::strict_json::{Map, Value, parse_with_scratch};

/// Decodes one admitted Robot error response with strict source-locked rules.
pub fn decode_robot_failure(
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let status = response.status();
    if matches!(status.get(), 401 | 503) {
        if !response.body().is_empty() {
            return Err(RobotDecodeError::UnexpectedBody);
        }
        return Ok(if status.get() == 401 {
            RobotFailure::AuthenticationRejected
        } else {
            RobotFailure::Maintenance
        });
    }
    if status.is_success() {
        return Err(RobotDecodeError::UnexpectedSuccessStatus);
    }
    if !matches!(status.get(), 400 | 403 | 404) {
        return Err(RobotDecodeError::UnsupportedStatus);
    }
    if response.body().is_empty() {
        return Err(RobotDecodeError::MissingBody);
    }
    if response.body().len() > MAX_ROBOT_ERROR_BODY_BYTES {
        return Err(RobotDecodeError::ResponseTooLarge);
    }
    let content_type = response
        .content_type()
        .map_err(|_| RobotDecodeError::InvalidContentType)?
        .ok_or(RobotDecodeError::InvalidContentType)?;
    if !content_type.matches(MediaType::JSON) {
        return Err(RobotDecodeError::InvalidContentType);
    }
    decode_json_failure(status, response.body(), workspace.decoder_scratch_mut())
}

fn decode_json_failure(
    status: StatusCode,
    body: &[u8],
    scratch: &mut [u8],
) -> Result<RobotFailure, RobotDecodeError> {
    let mut value =
        parse_with_scratch(body, scratch).map_err(|_| RobotDecodeError::MalformedPayload)?;
    let root = value
        .as_object_mut()
        .ok_or(RobotDecodeError::InvalidEnvelope)?;
    require_fields(root, &["error"])?;
    let error = root
        .get_mut("error")
        .and_then(Value::as_object_mut)
        .ok_or(RobotDecodeError::InvalidEnvelope)?;
    match status.get() {
        400 => decode_invalid_input(error),
        403 => decode_quota(error),
        404 => decode_provider_error(error),
        _ => Err(RobotDecodeError::UnsupportedStatus),
    }
}

fn decode_invalid_input(error: &mut Map) -> Result<RobotFailure, RobotDecodeError> {
    require_fields(error, &["status", "code", "message", "missing", "invalid"])?;
    require_status(error, 400)?;
    require_code(error, "INVALID_INPUT")?;
    let message = take_text(error, "message", MAX_ROBOT_ERROR_MESSAGE_BYTES)?;
    let missing = take_nullable_texts(error, "missing")?;
    let invalid = take_nullable_texts(error, "invalid")?;
    Ok(RobotFailure::InvalidInput(RobotInvalidInput {
        message,
        missing,
        invalid,
    }))
}

fn decode_quota(error: &mut Map) -> Result<RobotFailure, RobotDecodeError> {
    require_fields(
        error,
        &["status", "code", "message", "max_request", "interval"],
    )?;
    require_status(error, 403)?;
    require_code(error, "RATE_LIMIT_EXCEEDED")?;
    let max_requests = required_u64(error, "max_request")?;
    let interval = required_u64(error, "interval")?;
    if max_requests == 0 || interval == 0 {
        return Err(RobotDecodeError::InvalidQuota);
    }
    let message = take_text(error, "message", MAX_ROBOT_ERROR_MESSAGE_BYTES)?;
    Ok(RobotFailure::QuotaExceeded(RobotQuota {
        message,
        max_requests,
        interval: DelaySeconds::new(interval),
    }))
}

fn decode_provider_error(error: &mut Map) -> Result<RobotFailure, RobotDecodeError> {
    require_fields(error, &["status", "code", "message"])?;
    require_status(error, 404)?;
    require_code(error, "SERVER_NOT_FOUND")?;
    let message = take_text(error, "message", MAX_ROBOT_ERROR_MESSAGE_BYTES)?;
    Ok(RobotFailure::Provider(RobotProviderError {
        code: RobotProviderErrorCode::ServerNotFound,
        message,
    }))
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotDecodeError> {
    if object.len() != fields.len() || fields.iter().any(|field| object.get(field).is_none()) {
        return Err(RobotDecodeError::InvalidEnvelope);
    }
    Ok(())
}

fn require_status(object: &Map, expected: u64) -> Result<(), RobotDecodeError> {
    let status = required_u64(object, "status")?;
    if status != expected {
        return Err(RobotDecodeError::StatusMismatch);
    }
    Ok(())
}

fn required_u64(object: &Map, field: &str) -> Result<u64, RobotDecodeError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(RobotDecodeError::InvalidEnvelope)
}

fn require_code(object: &Map, expected: &str) -> Result<(), RobotDecodeError> {
    let value = object
        .get("code")
        .ok_or(RobotDecodeError::InvalidEnvelope)?;
    let is_expected = value
        .try_with_str(|text| text.len() <= MAX_ROBOT_ERROR_CODE_BYTES && text == expected)
        .map_err(|_| RobotDecodeError::InvalidEnvelope)?
        .ok_or(RobotDecodeError::InvalidEnvelope)?;
    if !is_expected {
        return Err(RobotDecodeError::UnknownCode);
    }
    Ok(())
}

fn take_text(object: &mut Map, field: &str, max: usize) -> Result<SensitiveText, RobotDecodeError> {
    let text = object
        .get_mut(field)
        .and_then(Value::take_string)
        .map(SensitiveText::new)
        .ok_or(RobotDecodeError::InvalidEnvelope)?;
    text.validate(max)
        .map_err(|_| RobotDecodeError::InvalidEnvelope)?;
    Ok(text)
}

fn take_nullable_texts(
    object: &mut Map,
    field: &str,
) -> Result<Vec<SensitiveText>, RobotDecodeError> {
    let value = object
        .get_mut(field)
        .ok_or(RobotDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(Vec::new());
    }
    let values = value
        .take_array()
        .ok_or(RobotDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_INPUT_FIELDS {
        return Err(RobotDecodeError::InvalidEnvelope);
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(values.len())
        .map_err(|_| RobotDecodeError::Allocation)?;
    for mut value in values {
        let text = value
            .take_string()
            .map(SensitiveText::new)
            .ok_or(RobotDecodeError::InvalidEnvelope)?;
        text.validate(MAX_ROBOT_INPUT_FIELD_BYTES)
            .map_err(|_| RobotDecodeError::InvalidEnvelope)?;
        result.push(text);
    }
    Ok(result)
}
