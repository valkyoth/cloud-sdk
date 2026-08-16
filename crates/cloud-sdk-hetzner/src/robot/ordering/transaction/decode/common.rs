use alloc::vec::Vec;

use cloud_sdk_sanitization::sanitize_bytes;

use super::super::model::{
    MAX_ROBOT_ORDER_TRANSACTION_KEYS, RobotOrderTransactionKey, RobotOrderTransactionStatus,
    RobotOrderTransactionTimestamp, ServerTransactionCommon,
};
use super::RobotOrderTransactionDecodeError;
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::robot::ordering::{
    RobotOrderChoice, RobotOrderLocation, RobotOrderProductId, RobotOrderText,
    RobotOrderTransactionId, RobotOrderValueError,
};
use crate::serde::strict_json::{Map, Value};

pub(super) fn require_fields(
    object: &Map,
    fields: &[&str],
) -> Result<(), RobotOrderTransactionDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotOrderTransactionDecodeError::InvalidEnvelope)
    }
}

pub(super) fn transaction_object(
    value: &mut Value,
) -> Result<&mut Map, RobotOrderTransactionDecodeError> {
    let wrapper = value
        .as_object_mut()
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["transaction"])?;
    wrapper
        .get_mut("transaction")
        .and_then(Value::as_object_mut)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)
}

pub(super) fn parse_common(
    object: &mut Map,
) -> Result<ServerTransactionCommon, RobotOrderTransactionDecodeError> {
    let status = status(object, "status")?;
    let server_number = nullable_server_number(object, "server_number")?;
    let server_ip = nullable_server_ip(object, "server_ip")?;
    let has_server = server_number.is_some() && server_ip.is_some();
    if matches!(status, RobotOrderTransactionStatus::Ready) != has_server
        || server_number.is_some() != server_ip.is_some()
    {
        return Err(RobotOrderTransactionDecodeError::InvalidStatus);
    }
    Ok(ServerTransactionCommon {
        id: transaction_id(object, "id")?,
        date: timestamp(object, "date")?,
        status,
        server_number,
        server_ip,
        authorized_keys: keys(object, "authorized_key", true)?,
        host_keys: keys(object, "host_key", false)?,
        comment: nullable_text(object, "comment")?,
    })
}

pub(super) fn transaction_id(
    object: &Map,
    field: &str,
) -> Result<RobotOrderTransactionId, RobotOrderTransactionDecodeError> {
    object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .try_with_str(RobotOrderTransactionId::new)
        .map_err(|_| RobotOrderTransactionDecodeError::InvalidTransaction)?
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)
}

pub(super) fn timestamp(
    object: &mut Map,
    field: &str,
) -> Result<RobotOrderTransactionTimestamp, RobotOrderTransactionDecodeError> {
    object
        .get_mut(field)
        .and_then(Value::take_string)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)
        .and_then(RobotOrderTransactionTimestamp::from_provider)
}

pub(super) fn status(
    object: &Map,
    field: &str,
) -> Result<RobotOrderTransactionStatus, RobotOrderTransactionDecodeError> {
    object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .try_with_str(|value| match value {
            "ready" => Ok(RobotOrderTransactionStatus::Ready),
            "in process" => Ok(RobotOrderTransactionStatus::InProcess),
            "cancelled" => Ok(RobotOrderTransactionStatus::Cancelled),
            _ => Err(RobotOrderTransactionDecodeError::InvalidStatus),
        })
        .map_err(|_| RobotOrderTransactionDecodeError::InvalidStatus)?
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
}

pub(super) fn text(
    object: &mut Map,
    field: &str,
) -> Result<RobotOrderText, RobotOrderTransactionDecodeError> {
    let value = object
        .get_mut(field)
        .and_then(Value::take_string)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    RobotOrderText::from_provider(value).map_err(map_catalog_decode)
}

pub(super) fn nullable_text(
    object: &mut Map,
    field: &str,
) -> Result<Option<RobotOrderText>, RobotOrderTransactionDecodeError> {
    let value = object
        .get_mut(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        Ok(None)
    } else {
        let value = value
            .take_string()
            .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
        RobotOrderText::from_provider(value)
            .map(Some)
            .map_err(map_catalog_decode)
    }
}

pub(super) fn text_list(
    object: &mut Map,
    field: &str,
) -> Result<Vec<RobotOrderText>, RobotOrderTransactionDecodeError> {
    let values = array(object, field, 4_096)?;
    let mut result = reserved(values.len())?;
    for mut value in values {
        let value = value
            .take_string()
            .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
        result.push(RobotOrderText::from_provider(value).map_err(map_catalog_decode)?);
    }
    Ok(result)
}

pub(super) fn choice(
    object: &Map,
    field: &str,
) -> Result<RobotOrderChoice, RobotOrderTransactionDecodeError> {
    object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .try_with_str(RobotOrderChoice::new)
        .map_err(|_| RobotOrderTransactionDecodeError::InvalidText)?
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)
}

pub(super) fn product_id(
    object: &Map,
    field: &str,
) -> Result<RobotOrderProductId, RobotOrderTransactionDecodeError> {
    object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .try_with_str(RobotOrderProductId::new)
        .map_err(|_| RobotOrderTransactionDecodeError::InvalidProduct)?
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)
}

pub(super) fn nullable_location(
    object: &Map,
    field: &str,
) -> Result<Option<RobotOrderLocation>, RobotOrderTransactionDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .try_with_str(RobotOrderLocation::new)
            .map_err(|_| RobotOrderTransactionDecodeError::InvalidText)?
            .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
            .map(Some)
            .map_err(map_value_error)
    }
}

pub(super) fn architecture(object: &Map) -> Result<u8, RobotOrderTransactionDecodeError> {
    object
        .get("@deprecated arch")
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .try_with_str(|value| match value {
            "32" => Ok(32),
            "64" => Ok(64),
            _ => Err(RobotOrderTransactionDecodeError::InvalidProduct),
        })
        .map_err(|_| RobotOrderTransactionDecodeError::InvalidProduct)?
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
}

pub(super) fn valid_legacy_timestamp(value: &RobotOrderText) -> bool {
    let mut canonical = [0_u8; 20];
    let valid = value
        .try_with_text(|text| {
            if text.len() != 19 || text.as_bytes().get(10) != Some(&b' ') {
                return false;
            }
            canonical[..19].copy_from_slice(text.as_bytes());
            canonical[10] = b'T';
            canonical[19] = b'Z';
            core::str::from_utf8(&canonical)
                .ok()
                .is_some_and(crate::serde::models::cloud_constraints::valid_rfc3339)
        })
        .unwrap_or(false);
    sanitize_bytes(&mut canonical);
    valid
}

pub(super) fn array(
    object: &mut Map,
    field: &str,
    maximum: usize,
) -> Result<Vec<Value>, RobotOrderTransactionDecodeError> {
    let values = object
        .get_mut(field)
        .and_then(Value::take_array)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    if values.len() > maximum {
        Err(RobotOrderTransactionDecodeError::InvalidList)
    } else {
        Ok(values)
    }
}

pub(super) fn reserved<T>(length: usize) -> Result<Vec<T>, RobotOrderTransactionDecodeError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| RobotOrderTransactionDecodeError::Allocation)?;
    Ok(result)
}

pub(super) fn reject_transaction_duplicates<T>(
    values: &[T],
    identity: impl Fn(&T) -> &RobotOrderTransactionId,
) -> Result<(), RobotOrderTransactionDecodeError> {
    reject_duplicates_by_cmp(values, |left, right| {
        identity(left).with_text(|left| identity(right).with_text(|right| left.cmp(right)))
    })
    .map_err(map_duplicate)
}

fn keys(
    object: &mut Map,
    field: &str,
    named: bool,
) -> Result<Vec<RobotOrderTransactionKey>, RobotOrderTransactionDecodeError> {
    let values = array(object, field, MAX_ROBOT_ORDER_TRANSACTION_KEYS)?;
    let mut result = reserved(values.len())?;
    for mut value in values {
        let wrapper = value
            .as_object_mut()
            .ok_or(RobotOrderTransactionDecodeError::InvalidKey)?;
        require_fields(wrapper, &["key"])?;
        let key = wrapper
            .get_mut("key")
            .and_then(Value::as_object_mut)
            .ok_or(RobotOrderTransactionDecodeError::InvalidKey)?;
        let fields = if named {
            &["name", "fingerprint", "type", "size"][..]
        } else {
            &["fingerprint", "type", "size"][..]
        };
        require_fields(key, fields).map_err(|_| RobotOrderTransactionDecodeError::InvalidKey)?;
        let name = if named {
            Some(text(key, "name")?)
        } else {
            None
        };
        let fingerprint = text(key, "fingerprint")?;
        let algorithm = text(key, "type")?;
        let size = key
            .get("size")
            .and_then(Value::as_u64)
            .filter(|value| *value != 0)
            .ok_or(RobotOrderTransactionDecodeError::InvalidKey)?;
        result.push(RobotOrderTransactionKey {
            name,
            fingerprint,
            algorithm,
            size,
        });
    }
    reject_duplicates_by_cmp(&result, |left, right| {
        left.fingerprint.compare(&right.fingerprint)
    })
    .map_err(|error| match error {
        DuplicateError::Duplicate => RobotOrderTransactionDecodeError::InvalidKey,
        DuplicateError::Allocation => RobotOrderTransactionDecodeError::Allocation,
    })?;
    Ok(result)
}

fn nullable_server_number(
    object: &Map,
    field: &str,
) -> Result<Option<crate::robot::RobotServerNumber>, RobotOrderTransactionDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_u64()
            .ok_or(RobotOrderTransactionDecodeError::InvalidServer)
            .and_then(|number| {
                crate::robot::RobotServerNumber::new(number).map_err(|error| match error {
                    crate::robot::RobotServerNumberError::Zero => {
                        RobotOrderTransactionDecodeError::InvalidServer
                    }
                    crate::robot::RobotServerNumberError::Allocation => {
                        RobotOrderTransactionDecodeError::Allocation
                    }
                })
            })
            .map(Some)
    }
}

fn nullable_server_ip(
    object: &Map,
    field: &str,
) -> Result<Option<crate::robot::ProtectedIpAddr>, RobotOrderTransactionDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .try_with_str(crate::robot::ProtectedIpAddr::parse)
            .map_err(|_| RobotOrderTransactionDecodeError::InvalidServer)?
            .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
            .map(Some)
            .map_err(|error| match error {
                crate::robot::server::protected_parse::ProtectedValueError::Invalid => {
                    RobotOrderTransactionDecodeError::InvalidServer
                }
                crate::robot::server::protected_parse::ProtectedValueError::Allocation => {
                    RobotOrderTransactionDecodeError::Allocation
                }
            })
    }
}

pub(super) const fn map_value_error(
    error: RobotOrderValueError,
) -> RobotOrderTransactionDecodeError {
    if matches!(error, RobotOrderValueError::Allocation) {
        RobotOrderTransactionDecodeError::Allocation
    } else {
        RobotOrderTransactionDecodeError::InvalidText
    }
}

pub(super) const fn map_duplicate(error: DuplicateError) -> RobotOrderTransactionDecodeError {
    match error {
        DuplicateError::Duplicate => RobotOrderTransactionDecodeError::InvalidList,
        DuplicateError::Allocation => RobotOrderTransactionDecodeError::Allocation,
    }
}

fn map_catalog_decode(
    error: crate::robot::ordering::RobotOrderCatalogDecodeError,
) -> RobotOrderTransactionDecodeError {
    if matches!(
        error,
        crate::robot::ordering::RobotOrderCatalogDecodeError::Allocation
    ) {
        RobotOrderTransactionDecodeError::Allocation
    } else {
        RobotOrderTransactionDecodeError::InvalidText
    }
}
