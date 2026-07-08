#![no_std]
//! Optional reqwest transport adapter boundary.
//!
//! This crate intentionally does not admit `reqwest` yet. The dependency will
//! be added only after transport policy, TLS policy, and tests are source-locked.

#[cfg(feature = "std")]
extern crate std;

/// Transport adapter readiness state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReqwestAdapterStatus {
    /// Adapter crate boundary exists, but no transport dependency is admitted.
    DependencyNotAdmitted,
}
