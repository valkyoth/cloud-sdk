//! Storage Box endpoint path domains.

use cloud_sdk::{Method, buffer};

use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::cloud::shared::{CloudRequestError, write_id_path, write_static_path};
use crate::request::{ApiBaseUrl, EndpointPath};

use super::types::{
    StorageBoxId, StorageBoxRequestError, StorageBoxSnapshotId, StorageBoxSubaccountId,
    StorageBoxTypeId,
};

/// Resource-local action lookup is deprecated upstream and intentionally absent.
pub const RESOURCE_LOCAL_ACTION_GET_DEFERRED: bool = true;

/// Storage Box CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxEndpoint {
    /// `GET /storage_boxes`.
    List,
    /// `POST /storage_boxes`.
    Create,
    /// `GET /storage_boxes/{id}`.
    Get(StorageBoxId),
    /// `PUT /storage_boxes/{id}`.
    Update(StorageBoxId),
    /// `DELETE /storage_boxes/{id}`.
    Delete(StorageBoxId),
    /// `GET /storage_boxes/{id}/folders`.
    ListFolders(StorageBoxId),
}

impl StorageBoxEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) | Self::ListFolders(_) => Method::Get,
            Self::Create => Method::Post,
            Self::Update(_) => Method::Put,
            Self::Delete(_) => Method::Delete,
        }
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::HetznerV1
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::StorageBoxes
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        match self {
            Self::List | Self::Create => write_static_path(output, "/storage_boxes"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/storage_boxes/", id, "")
            }
            Self::ListFolders(id) => write_id_path(output, "/storage_boxes/", id, "/folders"),
        }
    }
}

/// Storage Box type catalog endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxTypeEndpoint {
    /// `GET /storage_box_types`.
    List,
    /// `GET /storage_box_types/{id}`.
    Get(StorageBoxTypeId),
}

impl StorageBoxTypeEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        Method::Get
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::HetznerV1
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::StorageBoxes
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        match self {
            Self::List => write_static_path(output, "/storage_box_types"),
            Self::Get(id) => write_id_path(output, "/storage_box_types/", id, ""),
        }
    }
}

/// Storage Box action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxActionEndpoint {
    /// `GET /storage_boxes/actions`.
    ListAll,
    /// `GET /storage_boxes/actions/{id}`.
    Get(ActionId),
    /// `GET /storage_boxes/{id}/actions`.
    ListForStorageBox(StorageBoxId),
    /// `POST /storage_boxes/{id}/actions/change_protection`.
    ChangeProtection(StorageBoxId),
    /// `POST /storage_boxes/{id}/actions/change_type`.
    ChangeType(StorageBoxId),
    /// `POST /storage_boxes/{id}/actions/disable_snapshot_plan`.
    DisableSnapshotPlan(StorageBoxId),
    /// `POST /storage_boxes/{id}/actions/enable_snapshot_plan`.
    EnableSnapshotPlan(StorageBoxId),
    /// `POST /storage_boxes/{id}/actions/reset_password`.
    ResetPassword(StorageBoxId),
    /// `POST /storage_boxes/{id}/actions/rollback_snapshot`.
    RollbackSnapshot(StorageBoxId),
    /// `POST /storage_boxes/{id}/actions/update_access_settings`.
    UpdateAccessSettings(StorageBoxId),
}

impl StorageBoxActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForStorageBox(_) => Method::Get,
            Self::ChangeProtection(_)
            | Self::ChangeType(_)
            | Self::DisableSnapshotPlan(_)
            | Self::EnableSnapshotPlan(_)
            | Self::ResetPassword(_)
            | Self::RollbackSnapshot(_)
            | Self::UpdateAccessSettings(_) => Method::Post,
        }
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::HetznerV1
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::StorageBoxActions
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        match self {
            Self::ListAll => write_static_path(output, "/storage_boxes/actions"),
            Self::Get(id) => write_action_path(output, "/storage_boxes/actions/", id, ""),
            Self::ListForStorageBox(id) => write_id_path(output, "/storage_boxes/", id, "/actions"),
            Self::ChangeProtection(id) => {
                write_id_path(output, "/storage_boxes/", id, "/actions/change_protection")
            }
            Self::ChangeType(id) => {
                write_id_path(output, "/storage_boxes/", id, "/actions/change_type")
            }
            Self::DisableSnapshotPlan(id) => write_id_path(
                output,
                "/storage_boxes/",
                id,
                "/actions/disable_snapshot_plan",
            ),
            Self::EnableSnapshotPlan(id) => write_id_path(
                output,
                "/storage_boxes/",
                id,
                "/actions/enable_snapshot_plan",
            ),
            Self::ResetPassword(id) => {
                write_id_path(output, "/storage_boxes/", id, "/actions/reset_password")
            }
            Self::RollbackSnapshot(id) => {
                write_id_path(output, "/storage_boxes/", id, "/actions/rollback_snapshot")
            }
            Self::UpdateAccessSettings(id) => write_id_path(
                output,
                "/storage_boxes/",
                id,
                "/actions/update_access_settings",
            ),
        }
    }
}

/// Storage Box snapshot endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxSnapshotEndpoint {
    /// `GET /storage_boxes/{id}/snapshots`.
    List(StorageBoxId),
    /// `POST /storage_boxes/{id}/snapshots`.
    Create(StorageBoxId),
    /// `GET /storage_boxes/{id}/snapshots/{snapshot_id}`.
    Get(StorageBoxId, StorageBoxSnapshotId),
    /// `PUT /storage_boxes/{id}/snapshots/{snapshot_id}`.
    Update(StorageBoxId, StorageBoxSnapshotId),
    /// `DELETE /storage_boxes/{id}/snapshots/{snapshot_id}`.
    Delete(StorageBoxId, StorageBoxSnapshotId),
}

impl StorageBoxSnapshotEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List(_) | Self::Get(_, _) => Method::Get,
            Self::Create(_) => Method::Post,
            Self::Update(_, _) => Method::Put,
            Self::Delete(_, _) => Method::Delete,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::StorageBoxes
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::HetznerV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        match self {
            Self::List(id) | Self::Create(id) => {
                write_id_path(output, "/storage_boxes/", id, "/snapshots")
            }
            Self::Get(id, snapshot_id)
            | Self::Update(id, snapshot_id)
            | Self::Delete(id, snapshot_id) => write_two_id_path(
                output,
                "/storage_boxes/",
                id,
                "/snapshots/",
                snapshot_id,
                "",
            ),
        }
    }
}

/// Storage Box subaccount endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxSubaccountEndpoint {
    /// `GET /storage_boxes/{id}/subaccounts`.
    List(StorageBoxId),
    /// `POST /storage_boxes/{id}/subaccounts`.
    Create(StorageBoxId),
    /// `GET /storage_boxes/{id}/subaccounts/{subaccount_id}`.
    Get(StorageBoxId, StorageBoxSubaccountId),
    /// `PUT /storage_boxes/{id}/subaccounts/{subaccount_id}`.
    Update(StorageBoxId, StorageBoxSubaccountId),
    /// `DELETE /storage_boxes/{id}/subaccounts/{subaccount_id}`.
    Delete(StorageBoxId, StorageBoxSubaccountId),
}

impl StorageBoxSubaccountEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List(_) | Self::Get(_, _) => Method::Get,
            Self::Create(_) => Method::Post,
            Self::Update(_, _) => Method::Put,
            Self::Delete(_, _) => Method::Delete,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::StorageBoxSubaccounts
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::HetznerV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        match self {
            Self::List(id) | Self::Create(id) => {
                write_id_path(output, "/storage_boxes/", id, "/subaccounts")
            }
            Self::Get(id, subaccount_id)
            | Self::Update(id, subaccount_id)
            | Self::Delete(id, subaccount_id) => write_two_id_path(
                output,
                "/storage_boxes/",
                id,
                "/subaccounts/",
                subaccount_id,
                "",
            ),
        }
    }
}

/// Storage Box subaccount action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxSubaccountActionEndpoint {
    /// `POST /storage_boxes/{id}/subaccounts/{subaccount_id}/actions/change_home_directory`.
    ChangeHomeDirectory(StorageBoxId, StorageBoxSubaccountId),
    /// `POST /storage_boxes/{id}/subaccounts/{subaccount_id}/actions/reset_subaccount_password`.
    ResetPassword(StorageBoxId, StorageBoxSubaccountId),
    /// `POST /storage_boxes/{id}/subaccounts/{subaccount_id}/actions/update_access_settings`.
    UpdateAccessSettings(StorageBoxId, StorageBoxSubaccountId),
}

impl StorageBoxSubaccountActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        Method::Post
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::StorageBoxSubaccounts
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::HetznerV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        match self {
            Self::ChangeHomeDirectory(id, subaccount_id) => write_two_id_path(
                output,
                "/storage_boxes/",
                id,
                "/subaccounts/",
                subaccount_id,
                "/actions/change_home_directory",
            ),
            Self::ResetPassword(id, subaccount_id) => write_two_id_path(
                output,
                "/storage_boxes/",
                id,
                "/subaccounts/",
                subaccount_id,
                "/actions/reset_subaccount_password",
            ),
            Self::UpdateAccessSettings(id, subaccount_id) => write_two_id_path(
                output,
                "/storage_boxes/",
                id,
                "/subaccounts/",
                subaccount_id,
                "/actions/update_access_settings",
            ),
        }
    }
}

fn write_action_path(
    output: &mut [u8],
    prefix: &str,
    id: ActionId,
    suffix: &str,
) -> Result<usize, StorageBoxRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        prefix,
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_u64(
        output,
        &mut len,
        id.get(),
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_str(
        output,
        &mut len,
        suffix,
        CloudRequestError::PathBufferTooSmall,
    )?;
    validate_written_path(output, len)?;
    Ok(len)
}

fn write_two_id_path(
    output: &mut [u8],
    prefix: &str,
    first: StorageBoxId,
    middle: &str,
    second: StorageBoxId,
    suffix: &str,
) -> Result<usize, StorageBoxRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        prefix,
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_u64(
        output,
        &mut len,
        first.get(),
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_str(
        output,
        &mut len,
        middle,
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_u64(
        output,
        &mut len,
        second.get(),
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_str(
        output,
        &mut len,
        suffix,
        CloudRequestError::PathBufferTooSmall,
    )?;
    validate_written_path(output, len)?;
    Ok(len)
}

fn validate_written_path(output: &[u8], len: usize) -> Result<(), StorageBoxRequestError> {
    let bytes = output
        .get(..len)
        .ok_or(CloudRequestError::PathBufferTooSmall)?;
    let path = core::str::from_utf8(bytes).map_err(|_| CloudRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(CloudRequestError::InvalidPath)?;
    Ok(())
}
