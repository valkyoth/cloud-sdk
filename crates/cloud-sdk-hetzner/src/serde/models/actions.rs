//! Owned validated action models.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk::action_polling::ActionUpdate;
use serde_json::Value;

use super::{ResponseModelError, checked_text, object, required};
use crate::actions::{ActionId, ActionStatus};
use crate::cloud::shared::CloudResourceId;
use crate::response::ApiErrorCode;

const MAX_ACTIONS: usize = 1024;
const MAX_ACTION_RESOURCES: usize = 256;

/// Resource referenced by an action result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResultResource {
    id: CloudResourceId,
    resource_type: String,
}

impl ActionResultResource {
    /// Returns the referenced resource identifier.
    #[must_use]
    pub const fn id(&self) -> CloudResourceId {
        self.id
    }

    /// Returns the provider resource type.
    #[must_use]
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }
}

/// Error embedded in a completed action result.
#[derive(Clone, Eq, PartialEq)]
pub struct ActionResultError {
    code: ApiErrorCode,
    message: String,
}

impl ActionResultError {
    /// Returns the classified provider error code.
    #[must_use]
    pub const fn code(&self) -> ApiErrorCode {
        self.code
    }

    /// Returns the provider message through an explicit accessor.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Debug for ActionResultError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActionResultError")
            .field("code", &self.code)
            .field("message", &"[redacted]")
            .finish()
    }
}

/// Validated action returned by a Hetzner operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResult {
    id: ActionId,
    command: String,
    status: ActionStatus,
    progress: u8,
    started: String,
    finished: Option<String>,
    resources: Vec<ActionResultResource>,
    error: Option<ActionResultError>,
}

impl ActionResult {
    /// Returns the action identifier.
    #[must_use]
    pub const fn id(&self) -> ActionId {
        self.id
    }

    /// Returns the action command.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Returns the source-known action status.
    #[must_use]
    pub const fn status(&self) -> ActionStatus {
        self.status
    }

    /// Returns action progress in `0..=100`.
    #[must_use]
    pub const fn progress(&self) -> u8 {
        self.progress
    }

    /// Returns the action start timestamp text.
    #[must_use]
    pub fn started(&self) -> &str {
        &self.started
    }

    /// Returns the optional finish timestamp text.
    #[must_use]
    pub fn finished(&self) -> Option<&str> {
        self.finished.as_deref()
    }

    /// Returns resources referenced by the action.
    #[must_use]
    pub fn resources(&self) -> &[ActionResultResource] {
        &self.resources
    }

    /// Returns the provider action error when supplied.
    #[must_use]
    pub const fn error(&self) -> Option<&ActionResultError> {
        self.error.as_ref()
    }

    /// Converts the action state into the provider-neutral polling update.
    #[must_use]
    pub const fn polling_update(&self) -> ActionUpdate<Option<&ActionResultError>> {
        match self.status {
            ActionStatus::Running => ActionUpdate::Running,
            ActionStatus::Success => ActionUpdate::Success,
            ActionStatus::Error => ActionUpdate::Failed(self.error.as_ref()),
        }
    }
}

pub(crate) fn parse_action(value: &Value) -> Result<ActionResult, ResponseModelError> {
    let fields = object(value)?;
    let id = required(fields, "id")?
        .as_u64()
        .and_then(ActionId::new)
        .ok_or(ResponseModelError::InvalidIdentifier)?;
    let command = text_field(fields, "command", 256)?;
    let status_text = required(fields, "status")?
        .as_str()
        .ok_or(ResponseModelError::WrongType)?;
    let status =
        ActionStatus::from_api_str(status_text).ok_or(ResponseModelError::UnknownEnumValue)?;
    let progress = required(fields, "progress")?
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| *value <= 100)
        .ok_or(ResponseModelError::InvalidNumber)?;
    let started = text_field(fields, "started", 64)?;
    let finished = nullable_text_field(fields, "finished", 64)?;
    let resources = parse_resources(required(fields, "resources")?)?;
    let error = parse_error(required(fields, "error")?)?;
    Ok(ActionResult {
        id,
        command,
        status,
        progress,
        started,
        finished,
        resources,
        error,
    })
}

pub(crate) fn parse_actions(value: &Value) -> Result<Vec<ActionResult>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_ACTIONS {
        return Err(ResponseModelError::TooManyItems);
    }
    values.iter().map(parse_action).collect()
}

fn parse_resources(value: &Value) -> Result<Vec<ActionResultResource>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_ACTION_RESOURCES {
        return Err(ResponseModelError::TooManyItems);
    }
    values
        .iter()
        .map(|value| {
            let fields = object(value)?;
            let id = required(fields, "id")?
                .as_u64()
                .and_then(CloudResourceId::new)
                .ok_or(ResponseModelError::InvalidIdentifier)?;
            let resource_type = text_field(fields, "type", 128)?;
            Ok(ActionResultResource { id, resource_type })
        })
        .collect()
}

fn parse_error(value: &Value) -> Result<Option<ActionResultError>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let fields = object(value)?;
    let code = text_field(fields, "code", 128)?;
    let message = text_field(fields, "message", 16_384)?;
    Ok(Some(ActionResultError {
        code: ApiErrorCode::from_api_str(&code),
        message,
    }))
}

fn text_field(
    fields: &serde_json::Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<String, ResponseModelError> {
    required(fields, key)?
        .as_str()
        .ok_or(ResponseModelError::WrongType)
        .and_then(|value| checked_text(value, max))
}

fn nullable_text_field(
    fields: &serde_json::Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<Option<String>, ResponseModelError> {
    let value = required(fields, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_str()
            .ok_or(ResponseModelError::WrongType)
            .and_then(|value| checked_text(value, max))
            .map(Some)
    }
}
