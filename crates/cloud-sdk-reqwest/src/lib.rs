#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

#[cfg(any(
    feature = "async-rustls",
    feature = "blocking-rustls",
    feature = "blocking-rustls-webpki-roots",
    feature = "blocking-rustls-fips"
))]
mod shared;

#[cfg(any(
    feature = "blocking-rustls",
    feature = "blocking-rustls-webpki-roots",
    feature = "blocking-rustls-fips"
))]
pub mod blocking;

#[cfg(feature = "async-rustls")]
pub mod asynchronous;

#[cfg(all(
    test,
    any(
        feature = "async-rustls",
        feature = "blocking-rustls",
        feature = "blocking-rustls-webpki-roots",
        feature = "blocking-rustls-fips"
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
    /// The blocking rustls adapter can require an explicitly verified FIPS configuration.
    BlockingRustlsFipsAvailable,
    /// The asynchronous rustls adapter is available when its feature is enabled.
    AsyncRustlsAvailable,
}
