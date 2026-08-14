use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::StatusCode;

use super::decode::RobotFirewallDecodeError;
use super::model::RobotFirewallRuleModel;
use super::prepare::MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES;
use crate::serde::SensitiveText;
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

pub(super) fn rules_equal(left: &RobotFirewallRuleModel, right: &RobotFirewallRuleModel) -> bool {
    left.ip_version == right.ip_version
        && left.protocol == right.protocol
        && left.action == right.action
        && optional_text_eq(left.name.as_ref(), right.name.as_ref())
        && optional_text_eq(left.destination_ip.as_ref(), right.destination_ip.as_ref())
        && optional_text_eq(left.source_ip.as_ref(), right.source_ip.as_ref())
        && optional_text_eq(
            left.destination_port.as_ref(),
            right.destination_port.as_ref(),
        )
        && optional_text_eq(left.source_port.as_ref(), right.source_port.as_ref())
        && optional_text_eq(left.tcp_flags.as_ref(), right.tcp_flags.as_ref())
}

fn optional_text_eq(left: Option<&SensitiveText>, right: Option<&SensitiveText>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left
            .try_with_secret(|left| {
                right
                    .try_with_secret(|right| left == right)
                    .unwrap_or(false)
            })
            .unwrap_or(false),
        _ => false,
    }
}

pub(super) const fn map_json_error(error: JsonError) -> RobotFirewallDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotFirewallDecodeError::Allocation
    } else {
        RobotFirewallDecodeError::MalformedPayload
    }
}
