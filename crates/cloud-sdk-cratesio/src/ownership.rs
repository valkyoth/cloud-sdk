//! Public owner models and authenticated ownership-operation ownership.
//! Read snapshots do not grant mutation authority.
#[cfg(feature = "alloc")]
pub use crate::accounts::{AccountKind, AccountRecord, OwnerList};
