//! Cross-resource action domains and request models.

use cloud_sdk::{Method, buffer};

use crate::EndpointGroup;
use crate::request::{ApiBaseUrl, EndpointPath, EndpointPathError};

/// Largest action identifier admitted by the source-locked API schema.
///
/// Mirrors the upstream OpenAPI maximum for action IDs: JavaScript's largest
/// exactly representable integer (`Number.MAX_SAFE_INTEGER`, or `2^53 - 1`).
pub const MAX_ACTION_ID: u64 = 9_007_199_254_740_991;

/// Maximum number of action IDs admitted by one global list request.
pub const MAX_ACTION_FILTER_IDS: usize = 128;

/// Error returned while building a global action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionRequestError {
    /// Endpoint paths failed validation.
    InvalidPath(EndpointPathError),
    /// The global action list requires at least one action ID.
    EmptyActionIds,
    /// The action-ID filter exceeds [`MAX_ACTION_FILTER_IDS`].
    TooManyActionIds,
    /// Caller-provided path buffer is too small.
    PathBufferTooSmall,
    /// Caller-provided query buffer is too small.
    QueryBufferTooSmall,
    /// Constructed path bytes were unexpectedly not UTF-8.
    PathEncodingFailed,
}

/// Action lifecycle state returned by long-running Hetzner operations.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ActionStatus {
    /// Action is running.
    Running,
    /// Action completed successfully.
    Success,
    /// Action failed.
    Error,
}

impl ActionStatus {
    /// Parses an action status string.
    #[must_use]
    pub const fn from_api_str(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"running" => Some(Self::Running),
            b"success" => Some(Self::Success),
            b"error" => Some(Self::Error),
            _ => None,
        }
    }

    /// Returns the API status string.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Error => "error",
        }
    }

    /// Returns true when polling should stop.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        match self {
            Self::Running => false,
            Self::Success | Self::Error => true,
        }
    }
}

/// Action identifier returned by Hetzner action resources.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActionId(u64);

impl ActionId {
    /// Creates an action identifier within the source-locked API bounds.
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 || value > MAX_ACTION_ID {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the raw identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Global action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionEndpoint {
    /// `GET /actions` with a required repeated action-ID query.
    List,
    /// `GET /actions/{id}`.
    Get(ActionId),
}

impl ActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        Method::Get
    }

    /// Returns the Cloud API base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Returns the source-locked endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::Actions
    }

    /// Returns the static list path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, ActionRequestError>> {
        match self {
            Self::List => {
                Some(EndpointPath::new("/actions").map_err(ActionRequestError::InvalidPath))
            }
            Self::Get(_) => None,
        }
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, ActionRequestError> {
        let mut len = 0;
        buffer::write_str(
            output,
            &mut len,
            "/actions",
            ActionRequestError::PathBufferTooSmall,
        )?;
        if let Self::Get(id) = self {
            buffer::write_byte(
                output,
                &mut len,
                b'/',
                ActionRequestError::PathBufferTooSmall,
            )?;
            buffer::write_u64(
                output,
                &mut len,
                id.get(),
                ActionRequestError::PathBufferTooSmall,
            )?;
        }
        validate_written_path(output, len)?;
        Ok(len)
    }
}

/// Required filters for `GET /actions`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionListRequest<'a> {
    ids: &'a [ActionId],
}

impl<'a> ActionListRequest<'a> {
    /// Creates a bounded nonempty action-ID filter.
    pub fn try_new(ids: &'a [ActionId]) -> Result<Self, ActionRequestError> {
        if ids.is_empty() {
            return Err(ActionRequestError::EmptyActionIds);
        }
        if ids.len() > MAX_ACTION_FILTER_IDS {
            return Err(ActionRequestError::TooManyActionIds);
        }
        Ok(Self { ids })
    }

    /// Returns the global action list endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ActionEndpoint {
        ActionEndpoint::List
    }

    /// Returns the validated action-ID filter.
    #[must_use]
    pub const fn ids(self) -> &'a [ActionId] {
        self.ids
    }

    /// Writes repeated `id` query parameters in caller order.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, ActionRequestError> {
        let mut len = 0;
        let mut first = true;
        for id in self.ids {
            buffer::write_query_u64(
                output,
                &mut len,
                &mut first,
                "id",
                id.get(),
                ActionRequestError::QueryBufferTooSmall,
            )?;
        }
        Ok(len)
    }
}

fn validate_written_path(output: &[u8], len: usize) -> Result<(), ActionRequestError> {
    let bytes = output
        .get(..len)
        .ok_or(ActionRequestError::PathBufferTooSmall)?;
    let path = core::str::from_utf8(bytes).map_err(|_| ActionRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(ActionRequestError::InvalidPath)?;
    Ok(())
}

#[cfg(test)]
mod tests;
