//! Storage Box subaccount request body markers.

use super::action_bodies::StorageBoxResetPasswordRequest;
use super::endpoints::StorageBoxSubaccountEndpoint;
use super::types::{
    StorageBoxHomeDirectory, StorageBoxId, StorageBoxLabels, StorageBoxPassword,
    StorageBoxSubaccountDescription, StorageBoxSubaccountId, StorageBoxSubaccountName,
};

/// Storage Box subaccount access settings request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSubaccountAccessSettingsRequest {
    pub(crate) readonly: Option<bool>,
    pub(crate) reachable_externally: Option<bool>,
    pub(crate) samba_enabled: Option<bool>,
    pub(crate) ssh_enabled: Option<bool>,
    pub(crate) webdav_enabled: Option<bool>,
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
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct StorageBoxSubaccountCreateRequest<'a> {
    storage_box: StorageBoxId,
    pub(crate) home_directory: StorageBoxHomeDirectory<'a>,
    pub(crate) password: StorageBoxPassword<'a>,
    pub(crate) access_settings: Option<StorageBoxSubaccountAccessSettingsRequest>,
    pub(crate) name: Option<StorageBoxSubaccountName<'a>>,
    pub(crate) description: Option<StorageBoxSubaccountDescription<'a>>,
    pub(crate) labels: Option<StorageBoxLabels<'a>>,
}

impl<'a> StorageBoxSubaccountCreateRequest<'a> {
    /// Creates a subaccount create request.
    #[must_use]
    pub const fn new(
        storage_box: StorageBoxId,
        home_directory: StorageBoxHomeDirectory<'a>,
        password: StorageBoxPassword<'a>,
    ) -> Self {
        Self {
            storage_box,
            home_directory,
            password,
            access_settings: None,
            name: None,
            description: None,
            labels: None,
        }
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
    pub(crate) name: Option<StorageBoxSubaccountName<'a>>,
    pub(crate) description: Option<StorageBoxSubaccountDescription<'a>>,
    pub(crate) labels: Option<StorageBoxLabels<'a>>,
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
    #[must_use]
    pub const fn new(home_directory: StorageBoxHomeDirectory<'a>) -> Self {
        Self { home_directory }
    }

    /// Returns the home directory marker.
    #[must_use]
    pub const fn home_directory(self) -> StorageBoxHomeDirectory<'a> {
        self.home_directory
    }
}
