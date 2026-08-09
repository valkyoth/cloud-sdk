//! Policy-bound checked Hetzner response decoding.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk::operation::{CheckedResponse, PreparedRequest, ResponsePolicyError};
use cloud_sdk::rate_limit::{RateLimit, WallClockTimestamp};
use cloud_sdk::transport::{
    MediaType, ResponseBuffer, ResponseDecodeWorkspace, ResponseWriterError, TransportResponse,
};

use super::binding::{ResponseBinding, ResponseShape, find};
use super::models::{
    CompositeResult, HetznerSuccess, NamedSensitiveText, ResponseModelError, SensitiveText,
    is_cloud_resource_root, is_dns_resource_root, is_security_resource_root, object, parse_action,
    parse_actions, parse_cloud_resource, parse_cloud_resources, parse_dns_resource,
    parse_dns_resources, parse_folders, parse_location, parse_location_page, parse_metrics,
    parse_pagination, parse_pricing, parse_resource, parse_resources, parse_security_resource,
    parse_security_resources, parse_storage_box, parse_storage_box_composite_resource,
    parse_storage_box_page, parse_storage_box_snapshot, parse_storage_box_snapshots,
    parse_storage_box_subaccount, parse_storage_box_subaccounts, parse_storage_box_type,
    parse_storage_box_type_page, parse_zonefile, required, valid_error_code, value_text,
};
use super::strict_json;
use super::strict_json::{Map, Value};
use super::{
    IncrementalJsonDecoder, IncrementalJsonEvent, IncrementalJsonProgress, IncrementalJsonVisitor,
    VisitControl,
};
use super::{MAX_SERDE_RESPONSE_BYTES, ResponseBytes, ResponseSizeError};
use crate::association::ExpectedResponseIdentity;
use crate::identity::HETZNER_PROVIDER_ID;
use crate::rate_limit::{HetznerQuota, HetznerQuotaError};
use crate::response::ApiErrorCode;

mod success;
use success::{decode_checked_success, decode_provider_error};

/// Typed provider error returned by a checked operation response.
#[derive(Eq, PartialEq)]
pub struct HetznerApiError {
    code: ApiErrorCode,
    code_text: String,
    message: SensitiveText,
    quota: HetznerQuota,
}

impl HetznerApiError {
    /// Returns the classified provider error code.
    #[must_use]
    pub const fn code(&self) -> ApiErrorCode {
        self.code
    }

    /// Returns the exact validated provider error code.
    #[must_use]
    pub fn code_text(&self) -> &str {
        &self.code_text
    }

    /// Runs a closure with temporary access to the provider error message.
    pub fn try_with_message<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.message.try_with_secret(inspect)
    }

    /// Returns validated quota and retry metadata from the error response.
    #[must_use]
    pub fn quota(&self) -> &HetznerQuota {
        &self.quota
    }
}

impl fmt::Debug for HetznerApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HetznerApiError")
            .field("code", &self.code)
            .field("code_text", &"[redacted]")
            .field("message", &"[redacted]")
            .field("quota", &self.quota)
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
#[derive(Debug, Eq, PartialEq)]
pub enum HetznerDecodeError {
    /// The prepared request has no provider operation identifier.
    MissingOperationId,
    /// The operation is absent from the source-locked active response table.
    UnknownOperation,
    /// The prepared service does not match the operation's API family.
    ServiceMismatch,
    /// The response failed the prepared success policy.
    ResponsePolicy(ResponsePolicyError),
    /// The response writer was not committed through the admitted buffer.
    ResponseWriter(ResponseWriterError),
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
    /// Provider quota or retry metadata was incomplete or invalid.
    Quota(HetznerQuotaError),
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
            Self::ResponseWriter(_) => "Hetzner response writer state is invalid",
            Self::ResponseSize(_) => "Hetzner response exceeds the parser size limit",
            Self::ErrorContentType => "Hetzner error response content type is invalid",
            Self::MissingErrorBody => "Hetzner error response body is missing",
            Self::MalformedPayload => "Hetzner response JSON is malformed",
            Self::Model(_) => "Hetzner response model validation failed",
            Self::Quota(_) => "Hetzner quota response metadata is invalid",
            Self::Provider(_) => "Hetzner API returned a validated error response",
        })
    }
}

impl core::error::Error for HetznerDecodeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ResponsePolicy(error) => Some(error),
            Self::ResponseWriter(error) => Some(error),
            Self::ResponseSize(error) => Some(error),
            Self::Model(error) => Some(error),
            Self::Quota(error) => Some(error),
            Self::Provider(error) => Some(error),
            _ => None,
        }
    }
}

/// Successful checked response plus validated rate-limit metadata.
///
/// Ordinary equality is unavailable because a successful DNS response can
/// contain TSIG material.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::CheckedHetznerResponse;
/// fn compare(left: CheckedHetznerResponse, right: CheckedHetznerResponse) -> bool {
///     left == right
/// }
/// ```
#[derive(Debug)]
pub struct CheckedHetznerResponse {
    success: HetznerSuccess,
    quota: HetznerQuota,
}

impl CheckedHetznerResponse {
    /// Returns the typed operation success value.
    #[must_use]
    pub const fn success(&self) -> &HetznerSuccess {
        &self.success
    }

    /// Returns validated provider-owned quota and retry metadata.
    #[must_use]
    pub fn quota(&self) -> &HetznerQuota {
        &self.quota
    }

    /// Returns the legacy single-bucket rate-limit compatibility view.
    #[must_use]
    pub fn rate_limit(&self) -> Option<RateLimit> {
        self.quota.rate_limit()
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
    response: ResponseBuffer<'_>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    decode_response_with_clock(prepared, response, None)
}

/// Decodes a response while preserving the compile-time operation association
/// until the checked provider boundary.
pub fn decode_associated_response<O: crate::association::HetznerOperation>(
    prepared: crate::association::Prepared<'_, O>,
    response: ResponseBuffer<'_>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    let (prepared, expected_identity) = prepared.into_parts();
    let decoded = decode_response(prepared, response)?;
    validate_expected_identity(decoded.success(), expected_identity)?;
    Ok(decoded)
}

/// Decodes one successful typed execution result without reopening raw bytes.
pub fn decode_associated_checked_response<O: crate::association::HetznerOperation>(
    checked: crate::association::AssociatedCheckedResponse<'_, O>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    let operation = O::DESCRIPTOR.operation_id();
    let binding = find(operation.as_str()).ok_or(HetznerDecodeError::UnknownOperation)?;
    if O::DESCRIPTOR.service_id() != binding.service_id {
        return Err(HetznerDecodeError::ServiceMismatch);
    }
    let (checked, expected_identity) = checked.into_parts();
    let decoded = checked.decode_owned_with_workspace(|checked, workspace| {
        let quota = HetznerQuota::decode_without_clock(checked.headers())
            .map_err(HetznerDecodeError::Quota)?;
        decode_checked_success(operation.as_str(), binding, checked, workspace, quota)
    })?;
    validate_expected_identity(decoded.success(), expected_identity)?;
    Ok(decoded)
}

fn validate_expected_identity(
    success: &HetznerSuccess,
    expected: ExpectedResponseIdentity,
) -> Result<(), HetznerDecodeError> {
    let matches = match (expected, success) {
        (ExpectedResponseIdentity::None, _) => true,
        (ExpectedResponseIdentity::StorageBox(expected), HetznerSuccess::StorageBox(value)) => {
            value.id() == expected
        }
        (
            ExpectedResponseIdentity::StorageBoxType(expected),
            HetznerSuccess::StorageBoxType(value),
        ) => value.id() == expected,
        (
            ExpectedResponseIdentity::StorageBoxSnapshot {
                storage_box,
                snapshot,
            },
            HetznerSuccess::StorageBoxSnapshot(value),
        ) => value.storage_box() == storage_box && snapshot == Some(value.id()),
        (
            ExpectedResponseIdentity::StorageBoxSnapshot {
                storage_box,
                snapshot: None,
            },
            HetznerSuccess::StorageBoxSnapshots(values),
        ) => values
            .iter()
            .all(|value| value.storage_box() == storage_box),
        (
            ExpectedResponseIdentity::StorageBoxSubaccount {
                storage_box,
                subaccount,
            },
            HetznerSuccess::StorageBoxSubaccount(value),
        ) => value.storage_box() == storage_box && subaccount == Some(value.id()),
        (
            ExpectedResponseIdentity::StorageBoxSubaccount {
                storage_box,
                subaccount: None,
            },
            HetznerSuccess::StorageBoxSubaccounts(values),
        ) => values
            .iter()
            .all(|value| value.storage_box() == storage_box),
        (
            ExpectedResponseIdentity::StorageBoxSnapshot {
                storage_box,
                snapshot: None,
            },
            HetznerSuccess::Composite(value),
        ) => matches!(
            value.storage_box_resource(),
            Some(super::models::StorageBoxResource::SnapshotReference(reference))
                if reference.storage_box() == storage_box
        ),
        (
            ExpectedResponseIdentity::StorageBoxSubaccount {
                storage_box,
                subaccount: None,
            },
            HetznerSuccess::Composite(value),
        ) => matches!(
            value.storage_box_resource(),
            Some(super::models::StorageBoxResource::SubaccountReference(reference))
                if reference.storage_box() == storage_box
        ),
        _ => false,
    };
    if matches {
        Ok(())
    } else {
        Err(HetznerDecodeError::Model(
            ResponseModelError::ResponseIdentityMismatch,
        ))
    }
}

/// Decodes one response with caller-supplied wall time for obsolete HTTP dates.
pub fn decode_response_at(
    prepared: PreparedRequest<'_>,
    response: ResponseBuffer<'_>,
    now: WallClockTimestamp,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    decode_response_with_clock(prepared, response, Some(now))
}

#[allow(
    clippy::large_types_passed_by_value,
    reason = "the public checked decoder intentionally consumes one complete prepared request"
)]
fn decode_response_with_clock(
    prepared: PreparedRequest<'_>,
    mut response: ResponseBuffer<'_>,
    now: Option<WallClockTimestamp>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    let operation = prepared
        .operation_id()
        .ok_or(HetznerDecodeError::MissingOperationId)?;
    let binding = find(operation.as_str()).ok_or(HetznerDecodeError::UnknownOperation)?;
    let service = prepared.service();
    if service.provider_id() != HETZNER_PROVIDER_ID || service.service_id() != binding.service_id {
        return Err(HetznerDecodeError::ServiceMismatch);
    }
    let quota = response
        .with_response(|view| match now {
            Some(now) => HetznerQuota::decode(view.headers(), now),
            None => HetznerQuota::decode_without_clock(view.headers()),
        })
        .map_err(HetznerDecodeError::ResponseWriter)?
        .map_err(HetznerDecodeError::Quota)?;
    let status = response
        .with_response(|view| view.status())
        .map_err(HetznerDecodeError::ResponseWriter)?;
    if status.is_error() {
        prepared
            .apply_response_metadata_policy(&mut response)
            .map_err(HetznerDecodeError::ResponsePolicy)?;
        let mut workspace = ResponseDecodeWorkspace::new_for_provider();
        let decoded = response
            .with_response(|response| decode_provider_error(response, &mut workspace, quota))
            .map_err(HetznerDecodeError::ResponseWriter)?;
        drop(response);
        return match decoded {
            Ok(error) => Err(HetznerDecodeError::Provider(error)),
            Err(error) => Err(error),
        };
    }
    let checked = prepared
        .validate_response(response)
        .map_err(HetznerDecodeError::ResponsePolicy)?;
    checked.decode_owned_with_workspace(|checked, workspace| {
        decode_checked_success(operation.as_str(), binding, checked, workspace, quota)
    })
}
