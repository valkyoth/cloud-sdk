//! Storage Box request body markers.

use crate::cloud::shared::CloudRequestError;

use super::endpoints::{StorageBoxEndpoint, StorageBoxSnapshotEndpoint};
use super::types::{
    StorageBoxId, StorageBoxLabels, StorageBoxLocation, StorageBoxName, StorageBoxPassword,
    StorageBoxRequestError, StorageBoxSnapshotDescription, StorageBoxSnapshotId, StorageBoxSshKey,
    StorageBoxTypeRef,
};

/// Storage Box create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxCreateRequest<'a> {
    name: StorageBoxName<'a>,
    location: StorageBoxLocation<'a>,
    storage_box_type: StorageBoxTypeRef<'a>,
    password: StorageBoxPassword<'a>,
    labels: Option<StorageBoxLabels<'a>>,
    ssh_keys: Option<&'a [StorageBoxSshKey<'a>]>,
    access_settings: Option<StorageBoxAccessSettingsRequest>,
}

impl<'a> StorageBoxCreateRequest<'a> {
    /// Creates a validated create request.
    pub fn try_new(
        name: Option<StorageBoxName<'a>>,
        location: Option<StorageBoxLocation<'a>>,
        storage_box_type: Option<StorageBoxTypeRef<'a>>,
        password: Option<StorageBoxPassword<'a>>,
    ) -> Result<Self, StorageBoxRequestError> {
        Ok(Self {
            name: name.ok_or(CloudRequestError::MissingRequiredField)?,
            location: location.ok_or(CloudRequestError::MissingRequiredField)?,
            storage_box_type: storage_box_type.ok_or(CloudRequestError::MissingRequiredField)?,
            password: password.ok_or(CloudRequestError::MissingRequiredField)?,
            labels: None,
            ssh_keys: None,
            access_settings: None,
        })
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: StorageBoxLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Sets SSH keys.
    #[must_use]
    pub const fn with_ssh_keys(mut self, ssh_keys: &'a [StorageBoxSshKey<'a>]) -> Self {
        self.ssh_keys = Some(ssh_keys);
        self
    }

    /// Sets access settings.
    #[must_use]
    pub const fn with_access_settings(
        mut self,
        access_settings: StorageBoxAccessSettingsRequest,
    ) -> Self {
        self.access_settings = Some(access_settings);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> StorageBoxEndpoint {
        StorageBoxEndpoint::Create
    }
}

/// Storage Box update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxUpdateRequest<'a> {
    id: StorageBoxId,
    name: Option<StorageBoxName<'a>>,
    labels: Option<StorageBoxLabels<'a>>,
}

impl<'a> StorageBoxUpdateRequest<'a> {
    /// Creates an update request.
    #[must_use]
    pub const fn new(id: StorageBoxId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
        }
    }

    /// Sets replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: StorageBoxName<'a>) -> Self {
        self.name = Some(name);
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
    pub const fn endpoint(self) -> StorageBoxEndpoint {
        StorageBoxEndpoint::Update(self.id)
    }
}

/// Storage Box access settings request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxAccessSettingsRequest {
    reachable_externally: Option<bool>,
    samba_enabled: Option<bool>,
    ssh_enabled: Option<bool>,
    webdav_enabled: Option<bool>,
    zfs_enabled: Option<bool>,
}

impl StorageBoxAccessSettingsRequest {
    /// Creates empty access settings.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reachable_externally: None,
            samba_enabled: None,
            ssh_enabled: None,
            webdav_enabled: None,
            zfs_enabled: None,
        }
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
    /// Sets ZFS snapshot-folder visibility.
    #[must_use]
    pub const fn with_zfs_enabled(mut self, value: bool) -> Self {
        self.zfs_enabled = Some(value);
        self
    }
}

impl Default for StorageBoxAccessSettingsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage Box snapshot create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSnapshotCreateRequest<'a> {
    storage_box: StorageBoxId,
    description: Option<StorageBoxSnapshotDescription<'a>>,
    labels: Option<StorageBoxLabels<'a>>,
}

impl<'a> StorageBoxSnapshotCreateRequest<'a> {
    /// Creates a snapshot create request.
    #[must_use]
    pub const fn new(storage_box: StorageBoxId) -> Self {
        Self {
            storage_box,
            description: None,
            labels: None,
        }
    }

    /// Sets the snapshot description.
    #[must_use]
    pub const fn with_description(
        mut self,
        description: StorageBoxSnapshotDescription<'a>,
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
    pub const fn endpoint(self) -> StorageBoxSnapshotEndpoint {
        StorageBoxSnapshotEndpoint::Create(self.storage_box)
    }
}

/// Storage Box snapshot update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSnapshotUpdateRequest<'a> {
    storage_box: StorageBoxId,
    snapshot: StorageBoxSnapshotId,
    description: Option<StorageBoxSnapshotDescription<'a>>,
    labels: Option<StorageBoxLabels<'a>>,
}

impl<'a> StorageBoxSnapshotUpdateRequest<'a> {
    /// Creates a snapshot update request.
    #[must_use]
    pub const fn new(storage_box: StorageBoxId, snapshot: StorageBoxSnapshotId) -> Self {
        Self {
            storage_box,
            snapshot,
            description: None,
            labels: None,
        }
    }

    /// Sets the snapshot description.
    #[must_use]
    pub const fn with_description(
        mut self,
        description: StorageBoxSnapshotDescription<'a>,
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
    pub const fn endpoint(self) -> StorageBoxSnapshotEndpoint {
        StorageBoxSnapshotEndpoint::Update(self.storage_box, self.snapshot)
    }
}
