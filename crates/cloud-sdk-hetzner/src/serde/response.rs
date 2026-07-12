//! Validated borrowed response envelopes.

use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::fmt;

use ::serde::Deserialize;
use ::serde::de::Error as _;

use crate::actions::{ActionId, ActionStatus};
use crate::cloud::shared::CloudResourceId;
use crate::response::ApiErrorCode;

mod wire;

use wire::{ActionEnvelopeWire, ActionResourceWire, ApiErrorEnvelopeWire, ApiErrorWire};

/// Maximum resources admitted in one action response.
pub const MAX_ACTION_RESPONSE_RESOURCES: usize = 256;
/// Maximum API error message bytes admitted after parsing.
pub const MAX_API_ERROR_MESSAGE_BYTES: usize = 16_384;
/// Maximum raw response bytes admitted before format deserialization.
pub const MAX_SERDE_RESPONSE_BYTES: usize = 8_388_608;

const MAX_API_ERROR_CODE_BYTES: usize = 128;
const MAX_ACTION_COMMAND_BYTES: usize = 256;
const MAX_ACTION_TIMESTAMP_BYTES: usize = 64;
const MAX_ACTION_RESOURCE_TYPE_BYTES: usize = 128;

/// Error returned when raw response input exceeds the Serde boundary policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseSizeError {
    /// The response exceeds [`MAX_SERDE_RESPONSE_BYTES`].
    TooLarge,
}

/// Size-checked raw response input for a caller-selected Serde format parser.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ResponseBytes<'a>(&'a [u8]);

impl<'a> ResponseBytes<'a> {
    /// Checks raw response size before parser allocation or deserialization.
    pub const fn new(value: &'a [u8]) -> Result<Self, ResponseSizeError> {
        if value.len() > MAX_SERDE_RESPONSE_BYTES {
            return Err(ResponseSizeError::TooLarge);
        }
        Ok(Self(value))
    }

    /// Returns the admitted parser input.
    #[must_use]
    pub const fn as_slice(self) -> &'a [u8] {
        self.0
    }
}

impl fmt::Debug for ResponseBytes<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseBytes")
            .field("body", &"[redacted]")
            .field("len", &self.0.len())
            .finish()
    }
}

/// Classified API error body with borrowed-or-owned message text.
#[derive(Clone, Eq, PartialEq)]
pub struct ApiErrorResponse<'a> {
    code: ApiErrorCode,
    message: Cow<'a, str>,
}

impl fmt::Debug for ApiErrorResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApiErrorResponse")
            .field("code", &self.code)
            .field("message", &"[redacted]")
            .finish()
    }
}

impl ApiErrorResponse<'_> {
    /// Returns the classified API error code.
    #[must_use]
    pub const fn code(&self) -> ApiErrorCode {
        self.code
    }

    /// Returns the human-facing error message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_ref()
    }
}

/// Borrowed API error envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiErrorEnvelope<'a> {
    error: ApiErrorResponse<'a>,
}

impl<'a> ApiErrorEnvelope<'a> {
    /// Returns the validated error body.
    #[must_use]
    pub const fn error(&self) -> &ApiErrorResponse<'a> {
        &self.error
    }
}

impl<'de> Deserialize<'de> for ApiErrorEnvelope<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let wire = ApiErrorEnvelopeWire::deserialize(deserializer)?;
        Ok(Self {
            error: wire.error.try_into_response::<D::Error>()?,
        })
    }
}

/// Resource referenced by an action response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResource<'a> {
    id: CloudResourceId,
    resource_type: Cow<'a, str>,
}

impl<'a> ActionResource<'a> {
    /// Returns the nonzero resource identifier.
    #[must_use]
    pub const fn id(&self) -> CloudResourceId {
        self.id
    }

    /// Returns the provider resource type.
    #[must_use]
    pub fn resource_type(&self) -> &str {
        self.resource_type.as_ref()
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for ActionResource<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let wire = ActionResourceWire::deserialize(deserializer)?;
        let id = CloudResourceId::new(wire.id)
            .ok_or_else(|| D::Error::custom("action resource ID must be nonzero"))?;
        validate_text::<D::Error>(
            wire.resource_type.as_ref(),
            MAX_ACTION_RESOURCE_TYPE_BYTES,
            "action resource type is invalid",
        )?;
        Ok(Self {
            id,
            resource_type: wire.resource_type,
        })
    }
}

/// Validated action response body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResponse<'a> {
    id: ActionId,
    command: Cow<'a, str>,
    status: ActionStatus,
    progress: u8,
    started: Cow<'a, str>,
    finished: Option<Cow<'a, str>>,
    resources: Vec<ActionResource<'a>>,
    error: Option<ApiErrorResponse<'a>>,
}

impl<'a> ActionResponse<'a> {
    /// Returns the nonzero action identifier.
    #[must_use]
    pub const fn id(&self) -> ActionId {
        self.id
    }

    /// Returns the action command.
    #[must_use]
    pub fn command(&self) -> &str {
        self.command.as_ref()
    }

    /// Returns the validated action status.
    #[must_use]
    pub const fn status(&self) -> ActionStatus {
        self.status
    }

    /// Returns progress in the inclusive range `0..=100`.
    #[must_use]
    pub const fn progress(&self) -> u8 {
        self.progress
    }

    /// Returns the provider timestamp at which the action started.
    #[must_use]
    pub fn started(&self) -> &str {
        self.started.as_ref()
    }

    /// Returns the provider completion timestamp when present.
    #[must_use]
    pub fn finished(&self) -> Option<&str> {
        self.finished.as_deref()
    }

    /// Returns resources referenced by the action.
    #[must_use]
    pub fn resources(&self) -> &[ActionResource<'a>] {
        &self.resources
    }

    /// Returns the action error when present.
    #[must_use]
    pub const fn error(&self) -> Option<&ApiErrorResponse<'a>> {
        self.error.as_ref()
    }
}

/// Borrowed action response envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionEnvelope<'a> {
    action: ActionResponse<'a>,
}

impl<'a> ActionEnvelope<'a> {
    /// Returns the validated action body.
    #[must_use]
    pub const fn action(&self) -> &ActionResponse<'a> {
        &self.action
    }
}

impl<'de> Deserialize<'de> for ActionEnvelope<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let wire = ActionEnvelopeWire::deserialize(deserializer)?;
        let id = ActionId::new(wire.action.id)
            .ok_or_else(|| D::Error::custom("action ID must be nonzero"))?;
        let status = ActionStatus::from_api_str(wire.action.status.as_ref())
            .ok_or_else(|| D::Error::custom("unknown action status"))?;
        if wire.action.progress > 100 {
            return Err(D::Error::custom("action progress exceeds 100"));
        }
        validate_text::<D::Error>(
            wire.action.command.as_ref(),
            MAX_ACTION_COMMAND_BYTES,
            "action command is invalid",
        )?;
        validate_text::<D::Error>(
            wire.action.started.as_ref(),
            MAX_ACTION_TIMESTAMP_BYTES,
            "action start timestamp is invalid",
        )?;
        if let Some(finished) = wire.action.finished.0.as_deref() {
            validate_text::<D::Error>(
                finished,
                MAX_ACTION_TIMESTAMP_BYTES,
                "action finish timestamp is invalid",
            )?;
        }
        let error = wire
            .action
            .error
            .0
            .map(ApiErrorWire::try_into_response::<D::Error>)
            .transpose()?;
        Ok(Self {
            action: ActionResponse {
                id,
                command: wire.action.command,
                status,
                progress: wire.action.progress,
                started: wire.action.started,
                finished: wire.action.finished.0,
                resources: wire.action.resources.0,
                error,
            },
        })
    }
}

fn validate_text<E>(value: &str, max: usize, field: &str) -> Result<(), E>
where
    E: ::serde::de::Error,
{
    if value.is_empty() || value.len() > max || value.bytes().any(invalid_text_byte) {
        return Err(E::custom(field));
    }
    Ok(())
}

const fn invalid_text_byte(byte: u8) -> bool {
    byte < 0x20 || byte == 0x7f
}
