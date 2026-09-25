//! Cargo-compatible publish, yank, and unyank model ownership.
#[cfg(feature = "alloc")]
mod yank;
#[cfg(feature = "alloc")]
pub use yank::*;
