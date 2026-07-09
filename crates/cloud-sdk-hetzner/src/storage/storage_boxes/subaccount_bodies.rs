//! Storage Box subaccount request body markers.

use crate::cloud::shared::CloudRequestError;

use super::action_bodies::StorageBoxResetPasswordRequest;
use super::endpoints::StorageBoxSubaccountEndpoint;
use super::types::{
    StorageBoxHomeDirectory, StorageBoxId, StorageBoxLabels, StorageBoxPassword,
    StorageBoxRequestError, StorageBoxSubaccountDescription, StorageBoxSubaccountId,
    StorageBoxSubaccountName,
};

/// Storage Box subaccount access settings request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSubaccountAccessSettingsRequest {
    readonly: Option<bool>,
    reachable_externally: Option<bool>,
    samba_enabled: Option<bool>,
    ssh_enabled: Option<bool>,
    webdav_enabled: Option<bool>,
}

impl StorageBoxSubaccountAccessSettingsRequest {
    /// Creates empty subaccount access settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            readonly: None,
            reachable_externally: None,
            samba_enabled: None,
            ssh_enabled: None,
            webdav_enabled: None,
        }
    }

    /// Sets read-only mode.
    #[must_use]
    pub const fn with_readonly(mut self, value: bool) -> Self {
        self.readonly = Some(value);
        self
    }

    /// Sets external reachability.
    #[must_use]
    pub const fn with_reachable_externally(mut self, value: bool) -> Self {
        self.reachable_externally = Some(value);
        self
    }

    /// Sets Samba access.
    #[must_use]
    pub const fn with_samba_enabled(mut self, value: bool) -> Self {
        self.samba_enabled = Some(value);
        self
    }

    /// Sets SSH access.
    #[must_use]
    pub const fn with_ssh_enabled(mut self, value: bool) -> Self {
        self.ssh_enabled = Some(value);
        self
    }

    /// Sets WebDAV access.
    #[must_use]
    pub const fn with_webdav_enabled(mut self, value: bool) -> Self {
        self.webdav_enabled = Some(value);
        self
    }
}

impl Default for StorageBoxSubaccountAccessSettingsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage Box subaccount update-access-settings action request.
pub type StorageBoxSubaccountUpdateAccessSettingsRequest =
    StorageBoxSubaccountAccessSettingsRequest;

/// Storage Box subaccount reset-password action request.
pub type StorageBoxSubaccountResetPasswordRequest<'a> = StorageBoxResetPasswordRequest<'a>;

/// Storage Box subaccount create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSubaccountCreateRequest<'a> {
    storage_box: StorageBoxId,
    home_directory: StorageBoxHomeDirectory<'a>,
    password: StorageBoxPassword<'a>,
    access_settings: Option<StorageBoxSubaccountAccessSettingsRequest>,
    name: Option<StorageBoxSubaccountName<'a>>,
    description: Option<StorageBoxSubaccountDescription<'a>>,
    labels: Option<StorageBoxLabels<'a>>,
}

impl<'a> StorageBoxSubaccountCreateRequest<'a> {
    /// Creates a subaccount create request.
    pub fn try_new(
        storage_box: StorageBoxId,
        home_directory: Option<StorageBoxHomeDirectory<'a>>,
        password: Option<StorageBoxPassword<'a>>,
    ) -> Result<Self, StorageBoxRequestError> {
        Ok(Self {
            storage_box,
            home_directory: home_directory.ok_or(CloudRequestError::MissingRequiredField)?,
            password: password.ok_or(CloudRequestError::MissingRequiredField)?,
            access_settings: None,
            name: None,
            description: None,
            labels: None,
        })
    }

    /// Sets access settings.
    #[must_use]
    pub const fn with_access_settings(
        mut self,
        access_settings: StorageBoxSubaccountAccessSettingsRequest,
    ) -> Self {
        self.access_settings = Some(access_settings);
        self
    }

    /// Sets display name.
    #[must_use]
    pub const fn with_name(mut self, name: StorageBoxSubaccountName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets description.
    #[must_use]
    pub const fn with_description(
        mut self,
        description: StorageBoxSubaccountDescription<'a>,
    ) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: StorageBoxLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> StorageBoxSubaccountEndpoint {
        StorageBoxSubaccountEndpoint::Create(self.storage_box)
    }
}

/// Storage Box subaccount update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSubaccountUpdateRequest<'a> {
    storage_box: StorageBoxId,
    subaccount: StorageBoxSubaccountId,
    name: Option<StorageBoxSubaccountName<'a>>,
    description: Option<StorageBoxSubaccountDescription<'a>>,
    labels: Option<StorageBoxLabels<'a>>,
}

impl<'a> StorageBoxSubaccountUpdateRequest<'a> {
    /// Creates a subaccount update request.
    #[must_use]
    pub const fn new(storage_box: StorageBoxId, subaccount: StorageBoxSubaccountId) -> Self {
        Self {
            storage_box,
            subaccount,
            name: None,
            description: None,
            labels: None,
        }
    }

    /// Sets display name.
    #[must_use]
    pub const fn with_name(mut self, name: StorageBoxSubaccountName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets description.
    #[must_use]
    pub const fn with_description(
        mut self,
        description: StorageBoxSubaccountDescription<'a>,
    ) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: StorageBoxLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> StorageBoxSubaccountEndpoint {
        StorageBoxSubaccountEndpoint::Update(self.storage_box, self.subaccount)
    }
}

/// Storage Box subaccount change-home-directory action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxChangeHomeDirectoryRequest<'a> {
    home_directory: StorageBoxHomeDirectory<'a>,
}

impl<'a> StorageBoxChangeHomeDirectoryRequest<'a> {
    /// Creates a change-home-directory request.
    pub fn try_new(
        home_directory: Option<StorageBoxHomeDirectory<'a>>,
    ) -> Result<Self, StorageBoxRequestError> {
        Ok(Self {
            home_directory: home_directory.ok_or(CloudRequestError::MissingRequiredField)?,
        })
    }

    /// Returns the home directory marker.
    #[must_use]
    pub const fn home_directory(self) -> StorageBoxHomeDirectory<'a> {
        self.home_directory
    }
}
