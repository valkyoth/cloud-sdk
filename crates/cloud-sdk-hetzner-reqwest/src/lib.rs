#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

/// Transport adapter readiness state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReqwestAdapterStatus {
    /// Adapter crate boundary exists, but no transport dependency is admitted.
    DependencyNotAdmitted,
}
