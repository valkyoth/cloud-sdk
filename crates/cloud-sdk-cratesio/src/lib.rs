#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

pub mod accounts;
pub mod catalog;
#[cfg(feature = "alloc")]
pub mod credentials;
pub mod discovery;
pub mod downloads;
pub mod endpoint;
pub mod identifiers;
pub mod identity;
pub mod ownership;
pub mod pagination;
pub mod publishing;
pub mod query;
#[cfg(feature = "alloc")]
pub mod settings;
pub mod trusted_publishing;
pub mod versions;
pub mod wire;

pub use identity::{CRATES_IO_PROVIDER_ID, CratesIo, REGISTRY_SERVICE_ID, RegistryService};
