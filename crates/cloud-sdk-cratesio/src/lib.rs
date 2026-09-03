#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

pub mod accounts;
pub mod catalog;
pub mod identity;
pub mod ownership;
pub mod publishing;
pub mod trusted_publishing;

pub use identity::{CRATES_IO_PROVIDER_ID, CratesIo, REGISTRY_SERVICE_ID, RegistryService};
