#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(all(
    any(
        feature = "async-rustls",
        feature = "blocking-rustls",
        feature = "blocking-rustls-webpki-roots"
    ),
    not(any(
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    ))
))]
compile_error!(
    "cloud-sdk-reqwest transport features are unsupported on this target; use a target-native implementation of the cloud-sdk transport traits"
);

#[cfg(all(
    feature = "std",
    any(
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )
))]
extern crate std;

#[cfg(all(
    any(
        feature = "async-rustls",
        feature = "blocking-rustls",
        feature = "blocking-rustls-webpki-roots"
    ),
    any(
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )
))]
mod shared;

#[cfg(all(
    feature = "fuzzing",
    any(
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )
))]
#[doc(hidden)]
pub use shared::{fuzz_raw_http1_wire, fuzz_raw_response_parser};

#[cfg(all(
    any(feature = "blocking-rustls", feature = "blocking-rustls-webpki-roots"),
    any(
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )
))]
pub mod blocking;

#[cfg(all(
    feature = "async-rustls",
    any(
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )
))]
pub mod asynchronous;

#[cfg(all(
    test,
    any(
        feature = "async-rustls",
        feature = "blocking-rustls",
        feature = "blocking-rustls-webpki-roots"
    ),
    any(
        target_os = "freebsd",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )
))]
mod test_server;

/// Provider-neutral transport adapter readiness state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ReqwestAdapterStatus {
    /// The default crate graph remains no_std and transport-free.
    TransportFreeByDefault,
    /// The blocking rustls adapter is available when its feature is enabled.
    BlockingRustlsAvailable,
    /// The blocking adapter can use a deterministic Mozilla trust-root snapshot.
    BlockingRustlsWebPkiRootsAvailable,
    /// The asynchronous rustls adapter is available when its feature is enabled.
    AsyncRustlsAvailable,
}
