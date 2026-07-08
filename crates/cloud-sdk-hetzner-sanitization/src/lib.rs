#![no_std]
//! Optional secret-sanitization boundary for token-adjacent helpers.
//!
//! This crate intentionally does not admit a third-party sanitization dependency
//! yet. The dependency will be added only after a dedicated review.

#[cfg(feature = "std")]
extern crate std;

/// Secret-helper readiness state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SanitizationStatus {
    /// Boundary exists, but no secret-handling dependency is admitted.
    DependencyNotAdmitted,
}
