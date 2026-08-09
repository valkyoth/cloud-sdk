//! Source-complete Hetzner Console Storage Box response models.

mod box_model;
mod common;
mod parse;
mod resource;
mod snapshot;
mod subaccount;
mod type_page;

pub use box_model::{StorageBox, StorageBoxPage, StorageBoxStats, StorageBoxStatus};
pub use common::{
    AccessSettings, Deprecation, Money, Price, Protection, SnapshotPlan, StorageBoxType,
};
pub use resource::{
    StorageBoxResource, StorageBoxSnapshotReference, StorageBoxSubaccountReference,
};
pub use snapshot::{StorageBoxSnapshot, StorageBoxSnapshotStats};
pub use subaccount::{StorageBoxSubaccount, StorageBoxSubaccountAccessSettings};
pub use type_page::StorageBoxTypePage;

pub(crate) use box_model::{parse_storage_box, parse_storage_box_page};
pub(crate) use resource::parse_storage_box_composite_resource;
pub(crate) use snapshot::{parse_storage_box_snapshot, parse_storage_box_snapshots};
pub(crate) use subaccount::{parse_storage_box_subaccount, parse_storage_box_subaccounts};
pub(crate) use type_page::{parse_storage_box_type, parse_storage_box_type_page};
