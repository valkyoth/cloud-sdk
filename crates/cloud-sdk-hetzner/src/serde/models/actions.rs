//! Owned validated action models.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk::action_polling::ActionUpdate;

use super::{
    ResponseModelError, SensitiveText, UtcTimestamp, object, required, valid_error_code, value_text,
};
use crate::actions::{ActionId, ActionStatus, MAX_ACTION_ID};
use crate::cloud::shared::CloudResourceId;
use crate::response::ApiErrorCode;
use crate::serde::strict_json::{Map, Value};

const MAX_ACTIONS: usize = 1024;
const MAX_ACTION_RESOURCES: usize = 256;

/// Resource referenced by an action result.
#[derive(Clone, Eq, PartialEq)]
pub struct ActionResultResource {
    id: CloudResourceId,
    resource_type: String,
}

impl fmt::Debug for ActionResultResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActionResultResource")
            .field("id", &self.id)
            .field("resource_type", &"[redacted]")
            .finish()
    }
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
#[derive(Eq, PartialEq)]
pub struct ActionResultError {
    code: ApiErrorCode,
    code_text: String,
    message: SensitiveText,
}

impl ActionResultError {
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

    /// Runs a closure with temporary access to the provider action error message.
    pub fn try_with_message<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.message.try_with_secret(inspect)
    }
}

impl fmt::Debug for ActionResultError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActionResultError")
            .field("code", &self.code)
            .field("code_text", &"[redacted]")
            .field("message", &"[redacted]")
            .finish()
    }
}

/// Validated action returned by a Hetzner operation.
#[derive(Eq, PartialEq)]
pub struct ActionResult {
    id: ActionId,
    command: String,
    status: ActionStatus,
    progress: u8,
    started: UtcTimestamp,
    finished: Option<UtcTimestamp>,
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
        self.started.as_str()
    }

    /// Returns the optional finish timestamp text.
    #[must_use]
    pub fn finished(&self) -> Option<&str> {
        self.finished.as_ref().map(UtcTimestamp::as_str)
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

impl fmt::Debug for ActionResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActionResult")
            .field("id", &self.id)
            .field("command", &"[redacted]")
            .field("status", &self.status)
            .field("progress", &self.progress)
            .field("timestamps", &"[redacted]")
            .field("resource_count", &self.resources.len())
            .field("error", &self.error)
            .finish()
    }
}

pub(crate) fn parse_action(value: &mut Value) -> Result<ActionResult, ResponseModelError> {
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let id = required(fields, "id")?
        .as_u64()
        .and_then(ActionId::new)
        .ok_or(ResponseModelError::InvalidIdentifier)?;
    let command = text_field(fields, "command", 256)?;
    let status = required(fields, "status")?
        .try_with_str(ActionStatus::from_api_str)
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)?
        .ok_or(ResponseModelError::UnknownEnumValue)?;
    let progress = required(fields, "progress")?
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .filter(|value| *value <= 100)
        .ok_or(ResponseModelError::InvalidNumber)?;
    let started = timestamp_field(fields, "started")?;
    let finished = nullable_timestamp_field(fields, "finished")?;
    let resources = parse_resources(required(fields, "resources")?)?;
    let error = parse_error(
        fields
            .get_mut("error")
            .ok_or(ResponseModelError::MissingField)?,
    )?;
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

pub(crate) fn parse_actions(value: &mut Value) -> Result<Vec<ActionResult>, ResponseModelError> {
    let values = match value {
        Value::Array(values) => values,
        _ => return Err(ResponseModelError::WrongType),
    };
    if values.len() > MAX_ACTIONS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut actions = Vec::new();
    actions
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        actions.push(parse_action(value)?);
    }
    Ok(actions)
}

fn parse_resources(value: &Value) -> Result<Vec<ActionResultResource>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_ACTION_RESOURCES {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut resources = Vec::new();
    resources
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        let fields = object(value)?;
        let id = required(fields, "id")?
            .as_u64()
            .filter(|id| *id <= MAX_ACTION_ID)
            .and_then(CloudResourceId::new)
            .ok_or(ResponseModelError::InvalidIdentifier)?;
        let resource_type = text_field(fields, "type", 128)?;
        resources.push(ActionResultResource { id, resource_type });
    }
    Ok(resources)
}

fn parse_error(value: &mut Value) -> Result<Option<ActionResultError>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let code = text_field(fields, "code", 128)?;
    if !valid_error_code(&code, 128) {
        return Err(ResponseModelError::InvalidText);
    }
    let message = fields
        .get_mut("message")
        .ok_or(ResponseModelError::MissingField)?
        .take_string()
        .map(SensitiveText::new)
        .ok_or(ResponseModelError::WrongType)?;
    message.validate(16_384)?;
    Ok(Some(ActionResultError {
        code: ApiErrorCode::from_api_str(&code),
        code_text: code,
        message,
    }))
}

fn text_field(fields: &Map, key: &str, max: usize) -> Result<String, ResponseModelError> {
    value_text(required(fields, key)?, max)
}

fn timestamp_field(fields: &Map, key: &str) -> Result<UtcTimestamp, ResponseModelError> {
    let value = text_field(fields, key, 64)?;
    UtcTimestamp::try_new(&value)
}

fn nullable_timestamp_field(
    fields: &Map,
    key: &str,
) -> Result<Option<UtcTimestamp>, ResponseModelError> {
    let value = required(fields, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        let value = value_text(value, 64)?;
        UtcTimestamp::try_new(&value).map(Some)
    }
}
