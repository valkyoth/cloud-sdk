//! Storage Box action request body markers.

use super::bodies::StorageBoxAccessSettingsRequest;
use super::types::{
    SnapshotPlanDayOfMonth, SnapshotPlanDayOfWeek, SnapshotPlanHour, SnapshotPlanMaxSnapshots,
    SnapshotPlanMinute, StorageBoxPassword, StorageBoxSnapshotRef, StorageBoxTypeRef,
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
    #[must_use]
    pub const fn new(storage_box_type: StorageBoxTypeRef<'a>) -> Self {
        Self { storage_box_type }
    }

    /// Returns the requested type marker.
    #[must_use]
    pub const fn storage_box_type(self) -> StorageBoxTypeRef<'a> {
        self.storage_box_type
    }
}

/// Storage Box reset-password action request.
#[derive(Clone, Copy, Debug)]
pub struct StorageBoxResetPasswordRequest<'a> {
    password: StorageBoxPassword<'a>,
}

impl<'a> StorageBoxResetPasswordRequest<'a> {
    /// Creates a reset-password request.
    #[must_use]
    pub const fn new(password: StorageBoxPassword<'a>) -> Self {
        Self { password }
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
    #[must_use]
    pub const fn new(snapshot: StorageBoxSnapshotRef<'a>) -> Self {
        Self { snapshot }
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
    pub(crate) max_snapshots: SnapshotPlanMaxSnapshots,
    pub(crate) minute: SnapshotPlanMinute,
    pub(crate) hour: SnapshotPlanHour,
    pub(crate) day_of_week: Option<SnapshotPlanDayOfWeek>,
    pub(crate) day_of_month: Option<SnapshotPlanDayOfMonth>,
}

impl StorageBoxSnapshotPlanRequest {
    /// Creates a snapshot-plan request.
    #[must_use]
    pub const fn new(
        max_snapshots: SnapshotPlanMaxSnapshots,
        minute: SnapshotPlanMinute,
        hour: SnapshotPlanHour,
    ) -> Self {
        Self {
            max_snapshots,
            minute,
            hour,
            day_of_week: None,
            day_of_month: None,
        }
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
