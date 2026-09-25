//! Public owner snapshots and explicit single-attempt ownership mutations.
//! Read snapshots do not grant mutation authority.
#[cfg(feature = "alloc")]
pub use crate::accounts::{AccountKind, AccountRecord, OwnerList};
#[cfg(feature = "alloc")]
mod changes;
#[cfg(feature = "alloc")]
pub use changes::*;
