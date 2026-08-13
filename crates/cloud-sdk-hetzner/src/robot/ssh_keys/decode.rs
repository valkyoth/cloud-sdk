use alloc::vec::Vec;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::model::*;
use super::prepare::{
    MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES, MAX_ROBOT_SSH_KEY_LIST_RESPONSE_BYTES,
};
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::robot::server::protected_parse;
use crate::robot::{RobotSshKeyFingerprint, RobotSshKeyName, RobotSshKeyValueError};
use crate::security::shared::SshAlgorithm;
use crate::serde::SensitiveText;
use crate::serde::models::ssh_wire::{SshKeyIdentity, parse_openssh_key_identity};
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot SSH-key success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotSshKeyDecodeError {
    /// The checked status was not admitted for the operation.
    UnexpectedStatus,
    /// The body exceeded the operation's independent decode limit.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required or extra response fields violated the exact source shape.
    InvalidEnvelope,
    /// A name, fingerprint, algorithm, size, key, or timestamp was invalid.
    InvalidKey,
    /// A list exceeded its bound or repeated a fingerprint identity.
    InvalidList,
    /// The response fingerprint did not match the exact request.
    ResponseIdentityMismatch,
    /// A successful mutation contradicted the requested name or key.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotSshKeyDecodeError,
    Self::UnexpectedStatus => "Robot SSH-key response status is unexpected",
    Self::ResponseTooLarge => "Robot SSH-key response exceeds its operation limit",
    Self::MalformedPayload => "Robot SSH-key response JSON is malformed",
    Self::InvalidEnvelope => "Robot SSH-key response envelope is invalid",
    Self::InvalidKey => "Robot SSH-key response value is invalid",
    Self::InvalidList => "Robot SSH-key response list is invalid",
    Self::ResponseIdentityMismatch => "Robot SSH-key response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot SSH-key mutation response contradicts the request",
    Self::Allocation => "Robot SSH-key response allocation failed",
);

pub(crate) fn decode_robot_ssh_key_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotSshKeyList, RobotSshKeyDecodeError> {
    require_status(checked, StatusCode::OK)?;
    require_body_limit(checked, MAX_ROBOT_SSH_KEY_LIST_RESPONSE_BYTES)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = root
        .take_array()
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_SSH_KEY_LIST_ITEMS {
        return Err(RobotSshKeyDecodeError::InvalidList);
    }
    let mut keys = Vec::new();
    keys.try_reserve_exact(values.len())
        .map_err(|_| RobotSshKeyDecodeError::Allocation)?;
    for mut value in values {
        keys.push(parse_wrapper(&mut value)?);
    }
    reject_duplicates_by_cmp(&keys, |left, right| {
        left.fingerprint.with_text(|left| {
            right
                .fingerprint
                .with_text(|right| left.as_bytes().cmp(right.as_bytes()))
        })
    })
    .map_err(|error| match error {
        DuplicateError::Duplicate => RobotSshKeyDecodeError::InvalidList,
        DuplicateError::Allocation => RobotSshKeyDecodeError::Allocation,
    })?;
    Ok(RobotSshKeyList(keys))
}

pub(crate) fn decode_robot_ssh_key(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
    if !matches!(checked.status(), StatusCode::OK | StatusCode::CREATED) {
        return Err(RobotSshKeyDecodeError::UnexpectedStatus);
    }
    require_body_limit(checked, MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    parse_wrapper(&mut root)
}

fn parse_wrapper(value: &mut Value) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
    let wrapper = value
        .as_object_mut()
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["key"])?;
    let object = wrapper
        .get_mut("key")
        .and_then(Value::as_object_mut)
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)?;
    require_fields(
        object,
        &["name", "fingerprint", "type", "size", "data", "created_at"],
    )?;
    parse_key(object)
}

fn parse_key(object: &mut Map) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
    let name = text_value(object, "name", RobotSshKeyName::new)?;
    let fingerprint = text_value(object, "fingerprint", RobotSshKeyFingerprint::new)?;
    let supplied_md5 = fingerprint.with_text(parse_md5_fingerprint)?;
    let algorithm = object
        .get("type")
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)?
        .try_with_str(parse_algorithm)
        .map_err(|_| RobotSshKeyDecodeError::InvalidKey)?
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)??;
    let size_bits = object
        .get("size")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value != 0 && *value <= 16_384)
        .ok_or(RobotSshKeyDecodeError::InvalidKey)?;
    let created_at = object
        .get("created_at")
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)?
        .try_with_str(parse_created_at)
        .map_err(|_| RobotSshKeyDecodeError::InvalidKey)?
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)??;
    let data = object
        .get_mut("data")
        .and_then(Value::take_string)
        .map(SensitiveText::new)
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)?;
    let identity = data
        .try_with_secret(parse_openssh_key_identity)
        .map_err(|_| RobotSshKeyDecodeError::InvalidKey)?
        .map_err(map_model_error)?;
    require_identity(identity, algorithm, size_bits, supplied_md5)?;
    Ok(RobotSshKey {
        name,
        fingerprint,
        algorithm,
        size_bits,
        data,
        sha256_fingerprint: identity.sha256,
        created_at,
    })
}

fn text_value<T>(
    object: &Map,
    field: &str,
    parse: impl FnOnce(&str) -> Result<T, RobotSshKeyValueError>,
) -> Result<T, RobotSshKeyDecodeError> {
    object
        .get(field)
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)?
        .try_with_str(parse)
        .map_err(|_| RobotSshKeyDecodeError::InvalidKey)?
        .ok_or(RobotSshKeyDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)
}

fn require_identity(
    identity: SshKeyIdentity,
    algorithm: RobotSshKeyAlgorithm,
    size_bits: u32,
    supplied_md5: [u8; 16],
) -> Result<(), RobotSshKeyDecodeError> {
    if source_algorithm(identity.algorithm) == algorithm
        && identity.bits == size_bits
        && identity.md5 == supplied_md5
    {
        Ok(())
    } else {
        Err(RobotSshKeyDecodeError::InvalidKey)
    }
}

pub(super) const fn source_algorithm(algorithm: SshAlgorithm) -> RobotSshKeyAlgorithm {
    match algorithm {
        SshAlgorithm::Rsa => RobotSshKeyAlgorithm::Rsa,
        SshAlgorithm::EcdsaNistP256
        | SshAlgorithm::EcdsaNistP384
        | SshAlgorithm::EcdsaNistP521
        | SshAlgorithm::SkEcdsaNistP256 => RobotSshKeyAlgorithm::Ecdsa,
        SshAlgorithm::Ed25519 | SshAlgorithm::SkEd25519 => RobotSshKeyAlgorithm::Ed25519,
    }
}

fn parse_algorithm(value: &str) -> Result<RobotSshKeyAlgorithm, RobotSshKeyDecodeError> {
    match value {
        "RSA" => Ok(RobotSshKeyAlgorithm::Rsa),
        "ECDSA" => Ok(RobotSshKeyAlgorithm::Ecdsa),
        "ED25519" => Ok(RobotSshKeyAlgorithm::Ed25519),
        _ => Err(RobotSshKeyDecodeError::InvalidKey),
    }
}

fn parse_created_at(value: &str) -> Result<RobotSshKeyCreatedAt, RobotSshKeyDecodeError> {
    let bytes = value.as_bytes();
    if bytes.len() != 19
        || bytes.get(10) != Some(&b' ')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return Err(RobotSshKeyDecodeError::InvalidKey);
    }
    let date = value.get(..10).ok_or(RobotSshKeyDecodeError::InvalidKey)?;
    drop(protected_parse::date(date).map_err(|_| RobotSshKeyDecodeError::InvalidKey)?);
    let year = decimal_u16(bytes.get(..4).ok_or(RobotSshKeyDecodeError::InvalidKey)?)?;
    let month = decimal_u8(bytes.get(5..7).ok_or(RobotSshKeyDecodeError::InvalidKey)?)?;
    let day = decimal_u8(bytes.get(8..10).ok_or(RobotSshKeyDecodeError::InvalidKey)?)?;
    let hour = decimal_u8(
        bytes
            .get(11..13)
            .ok_or(RobotSshKeyDecodeError::InvalidKey)?,
    )?;
    let minute = decimal_u8(
        bytes
            .get(14..16)
            .ok_or(RobotSshKeyDecodeError::InvalidKey)?,
    )?;
    let second = decimal_u8(
        bytes
            .get(17..19)
            .ok_or(RobotSshKeyDecodeError::InvalidKey)?,
    )?;
    if hour > 23 || minute > 59 || second > 59 {
        return Err(RobotSshKeyDecodeError::InvalidKey);
    }
    Ok(RobotSshKeyCreatedAt {
        year,
        month,
        day,
        hour,
        minute,
        second,
    })
}

fn decimal_u8(value: &[u8]) -> Result<u8, RobotSshKeyDecodeError> {
    u8::try_from(decimal_u16(value)?).map_err(|_| RobotSshKeyDecodeError::InvalidKey)
}

fn decimal_u16(value: &[u8]) -> Result<u16, RobotSshKeyDecodeError> {
    let mut output = 0_u16;
    for byte in value {
        let digit = byte
            .checked_sub(b'0')
            .filter(|digit| *digit <= 9)
            .ok_or(RobotSshKeyDecodeError::InvalidKey)?;
        output = output
            .checked_mul(10)
            .and_then(|current| current.checked_add(u16::from(digit)))
            .ok_or(RobotSshKeyDecodeError::InvalidKey)?;
    }
    Ok(output)
}

fn parse_md5_fingerprint(value: &str) -> Result<[u8; 16], RobotSshKeyDecodeError> {
    let mut output = [0_u8; 16];
    let mut pieces = value.split(':');
    for byte in &mut output {
        let piece = pieces.next().ok_or(RobotSshKeyDecodeError::InvalidKey)?;
        *byte = u8::from_str_radix(piece, 16).map_err(|_| RobotSshKeyDecodeError::InvalidKey)?;
    }
    if pieces.next().is_some() {
        return Err(RobotSshKeyDecodeError::InvalidKey);
    }
    Ok(output)
}

fn require_status(
    checked: CheckedResponse<'_>,
    expected: StatusCode,
) -> Result<(), RobotSshKeyDecodeError> {
    if checked.status() == expected {
        Ok(())
    } else {
        Err(RobotSshKeyDecodeError::UnexpectedStatus)
    }
}

fn require_body_limit(
    checked: CheckedResponse<'_>,
    maximum: usize,
) -> Result<(), RobotSshKeyDecodeError> {
    if checked.body().len() <= maximum {
        Ok(())
    } else {
        Err(RobotSshKeyDecodeError::ResponseTooLarge)
    }
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotSshKeyDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotSshKeyDecodeError::InvalidEnvelope)
    }
}

const fn map_value_error(error: RobotSshKeyValueError) -> RobotSshKeyDecodeError {
    match error {
        RobotSshKeyValueError::Invalid => RobotSshKeyDecodeError::InvalidKey,
        RobotSshKeyValueError::Allocation => RobotSshKeyDecodeError::Allocation,
    }
}

const fn map_model_error(error: crate::serde::ResponseModelError) -> RobotSshKeyDecodeError {
    if matches!(error, crate::serde::ResponseModelError::Allocation) {
        RobotSshKeyDecodeError::Allocation
    } else {
        RobotSshKeyDecodeError::InvalidKey
    }
}

const fn map_json_error(error: JsonError) -> RobotSshKeyDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotSshKeyDecodeError::Allocation
    } else {
        RobotSshKeyDecodeError::MalformedPayload
    }
}
