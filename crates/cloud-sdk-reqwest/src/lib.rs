#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "blocking-rustls")]
pub mod blocking;

/// Provider-neutral transport adapter readiness state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReqwestAdapterStatus {
    /// Adapter crate boundary exists, but no transport dependency is admitted.
    DependencyNotAdmitted,
    /// The blocking rustls adapter is available when its feature is enabled.
    BlockingRustlsAvailable,
}
