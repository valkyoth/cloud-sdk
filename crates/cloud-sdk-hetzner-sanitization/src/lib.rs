#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

/// Secret-helper readiness state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SanitizationStatus {
    /// Boundary exists, but no secret-handling dependency is admitted.
    DependencyNotAdmitted,
}
