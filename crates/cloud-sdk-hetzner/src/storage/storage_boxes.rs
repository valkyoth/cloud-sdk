//! Storage Box endpoint domains.

mod action_bodies;
mod bodies;
mod endpoints;
mod requests;
mod subaccount_bodies;
mod types;

pub use action_bodies::{
    StorageBoxChangeTypeRequest, StorageBoxProtectionRequest, StorageBoxResetPasswordRequest,
    StorageBoxRollbackSnapshotRequest, StorageBoxSnapshotPlanRequest,
    StorageBoxUpdateAccessSettingsRequest,
};
pub use bodies::{
    StorageBoxAccessSettingsRequest, StorageBoxCreateRequest, StorageBoxSnapshotCreateRequest,
    StorageBoxSnapshotUpdateRequest, StorageBoxUpdateRequest,
};
pub use endpoints::{
    RESOURCE_LOCAL_ACTION_GET_DEFERRED, StorageBoxActionEndpoint, StorageBoxEndpoint,
    StorageBoxSnapshotEndpoint, StorageBoxSubaccountActionEndpoint, StorageBoxSubaccountEndpoint,
    StorageBoxTypeEndpoint,
};
pub use requests::{
    StorageBoxActionListRequest, StorageBoxListRequest, StorageBoxSnapshotListRequest,
    StorageBoxSubaccountListRequest, StorageBoxTypeListRequest,
};
pub use subaccount_bodies::{
    StorageBoxChangeHomeDirectoryRequest, StorageBoxSubaccountAccessSettingsRequest,
    StorageBoxSubaccountCreateRequest, StorageBoxSubaccountResetPasswordRequest,
    StorageBoxSubaccountUpdateAccessSettingsRequest, StorageBoxSubaccountUpdateRequest,
};
pub use types::{
    SnapshotPlanDayOfMonth, SnapshotPlanDayOfWeek, SnapshotPlanHour, SnapshotPlanMaxSnapshots,
    SnapshotPlanMinute, StorageBoxActionSortField, StorageBoxDescription, StorageBoxHomeDirectory,
    StorageBoxId, StorageBoxLabels, StorageBoxLocation, StorageBoxName, StorageBoxPassword,
    StorageBoxRequestError, StorageBoxSnapshotDescription, StorageBoxSnapshotId,
    StorageBoxSnapshotName, StorageBoxSnapshotRef, StorageBoxSnapshotSortField,
    StorageBoxSortField, StorageBoxSshKey, StorageBoxSubaccountDescription, StorageBoxSubaccountId,
    StorageBoxSubaccountName, StorageBoxSubaccountSortField, StorageBoxSubaccountUsername,
    StorageBoxText, StorageBoxTypeId, StorageBoxTypeName, StorageBoxTypeRef,
    StorageBoxTypeSortField,
};

use crate::EndpointGroup;

/// Storage Box endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::StorageBoxes,
    EndpointGroup::StorageBoxActions,
    EndpointGroup::StorageBoxSubaccounts,
];

#[cfg(test)]
mod tests;
