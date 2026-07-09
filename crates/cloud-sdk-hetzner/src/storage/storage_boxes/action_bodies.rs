//! Storage Box action request body markers.

use crate::cloud::shared::CloudRequestError;

use super::bodies::StorageBoxAccessSettingsRequest;
use super::types::{
    SnapshotPlanDayOfMonth, SnapshotPlanDayOfWeek, SnapshotPlanHour, SnapshotPlanMaxSnapshots,
    SnapshotPlanMinute, StorageBoxPassword, StorageBoxRequestError, StorageBoxSnapshotRef,
    StorageBoxTypeRef,
};

/// Storage Box update-access-settings action request.
pub type StorageBoxUpdateAccessSettingsRequest = StorageBoxAccessSettingsRequest;

/// Storage Box change-protection action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxProtectionRequest {
    delete: bool,
}

impl StorageBoxProtectionRequest {
    /// Creates a protection request.
    #[must_use]
    pub const fn new(delete: bool) -> Self {
        Self { delete }
    }

    /// Returns the delete protection value.
    #[must_use]
    pub const fn delete(self) -> bool {
        self.delete
    }
}

/// Storage Box change-type action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxChangeTypeRequest<'a> {
    storage_box_type: StorageBoxTypeRef<'a>,
}

impl<'a> StorageBoxChangeTypeRequest<'a> {
    /// Creates a change-type request.
    pub fn try_new(
        storage_box_type: Option<StorageBoxTypeRef<'a>>,
    ) -> Result<Self, StorageBoxRequestError> {
        Ok(Self {
            storage_box_type: storage_box_type.ok_or(CloudRequestError::MissingRequiredField)?,
        })
    }

    /// Returns the requested type marker.
    #[must_use]
    pub const fn storage_box_type(self) -> StorageBoxTypeRef<'a> {
        self.storage_box_type
    }
}

/// Storage Box reset-password action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxResetPasswordRequest<'a> {
    password: StorageBoxPassword<'a>,
}

impl<'a> StorageBoxResetPasswordRequest<'a> {
    /// Creates a reset-password request.
    pub fn try_new(
        password: Option<StorageBoxPassword<'a>>,
    ) -> Result<Self, StorageBoxRequestError> {
        Ok(Self {
            password: password.ok_or(CloudRequestError::MissingRequiredField)?,
        })
    }

    /// Returns the redacted password marker.
    #[must_use]
    pub const fn password(self) -> StorageBoxPassword<'a> {
        self.password
    }
}

/// Storage Box snapshot rollback action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxRollbackSnapshotRequest<'a> {
    snapshot: StorageBoxSnapshotRef<'a>,
}

impl<'a> StorageBoxRollbackSnapshotRequest<'a> {
    /// Creates a rollback request.
    pub fn try_new(
        snapshot: Option<StorageBoxSnapshotRef<'a>>,
    ) -> Result<Self, StorageBoxRequestError> {
        Ok(Self {
            snapshot: snapshot.ok_or(CloudRequestError::MissingRequiredField)?,
        })
    }

    /// Returns the snapshot ID-or-name marker.
    #[must_use]
    pub const fn snapshot(self) -> StorageBoxSnapshotRef<'a> {
        self.snapshot
    }
}

/// Storage Box enable-snapshot-plan action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSnapshotPlanRequest {
    max_snapshots: SnapshotPlanMaxSnapshots,
    minute: SnapshotPlanMinute,
    hour: SnapshotPlanHour,
    day_of_week: Option<SnapshotPlanDayOfWeek>,
    day_of_month: Option<SnapshotPlanDayOfMonth>,
}

impl StorageBoxSnapshotPlanRequest {
    /// Creates a snapshot-plan request.
    pub fn try_new(
        max_snapshots: Option<SnapshotPlanMaxSnapshots>,
        minute: Option<SnapshotPlanMinute>,
        hour: Option<SnapshotPlanHour>,
    ) -> Result<Self, StorageBoxRequestError> {
        Ok(Self {
            max_snapshots: max_snapshots.ok_or(CloudRequestError::MissingRequiredField)?,
            minute: minute.ok_or(CloudRequestError::MissingRequiredField)?,
            hour: hour.ok_or(CloudRequestError::MissingRequiredField)?,
            day_of_week: None,
            day_of_month: None,
        })
    }

    /// Sets the optional day of week. `None` means every day.
    #[must_use]
    pub const fn with_day_of_week(mut self, value: Option<SnapshotPlanDayOfWeek>) -> Self {
        self.day_of_week = value;
        self
    }

    /// Sets the optional day of month. `None` means every day.
    #[must_use]
    pub const fn with_day_of_month(mut self, value: Option<SnapshotPlanDayOfMonth>) -> Self {
        self.day_of_month = value;
        self
    }

    /// Returns the max snapshot count.
    #[must_use]
    pub const fn max_snapshots(self) -> SnapshotPlanMaxSnapshots {
        self.max_snapshots
    }
}
