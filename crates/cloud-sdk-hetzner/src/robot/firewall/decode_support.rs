use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::StatusCode;

use super::decode::RobotFirewallDecodeError;
use super::prepare::MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES;
use super::reconcile::rule_models_equal;
use crate::serde::strict_json::{JsonError, Map, Value};

pub(super) fn require_item(checked: CheckedResponse<'_>) -> Result<(), RobotFirewallDecodeError> {
    if !matches!(checked.status(), StatusCode::OK | StatusCode::CREATED) {
        return Err(RobotFirewallDecodeError::UnexpectedStatus);
    }
    require_limit(checked, MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES)
}

pub(super) fn require_status(
    checked: CheckedResponse<'_>,
    expected: StatusCode,
) -> Result<(), RobotFirewallDecodeError> {
    if checked.status() == expected {
        Ok(())
    } else {
        Err(RobotFirewallDecodeError::UnexpectedStatus)
    }
}

pub(super) fn require_limit(
    checked: CheckedResponse<'_>,
    maximum: usize,
) -> Result<(), RobotFirewallDecodeError> {
    if checked.body().len() <= maximum {
        Ok(())
    } else {
        Err(RobotFirewallDecodeError::ResponseTooLarge)
    }
}

pub(super) fn object_mut(value: &mut Value) -> Result<&mut Map, RobotFirewallDecodeError> {
    value
        .as_object_mut()
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)
}

pub(super) fn require_fields(
    object: &Map,
    fields: &[&str],
) -> Result<(), RobotFirewallDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotFirewallDecodeError::InvalidEnvelope)
    }
}

pub(super) fn require_fields_with_optional(
    object: &Map,
    required: &[&str],
    optional: &[&str],
) -> Result<(), RobotFirewallDecodeError> {
    let present_optional = optional
        .iter()
        .filter(|field| object.get(field).is_some())
        .count();
    let expected_count = required
        .len()
        .checked_add(present_optional)
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
    if object.len() == expected_count && required.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotFirewallDecodeError::InvalidEnvelope)
    }
}

pub(super) fn rules_equal(
    left: &super::model::RobotFirewallRuleModel,
    right: &super::model::RobotFirewallRuleModel,
) -> bool {
    rule_models_equal(left, right)
}

pub(super) const fn map_json_error(error: JsonError) -> RobotFirewallDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotFirewallDecodeError::Allocation
    } else {
        RobotFirewallDecodeError::MalformedPayload
    }
}
