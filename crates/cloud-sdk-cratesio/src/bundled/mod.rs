//! Opt-in official-origin constructors for the neutral rustls HTTP/1 adapter.
//! Unsupported operating systems use the core transport traits instead.

mod artifacts;
#[cfg(test)]
mod tests;
use crate::{endpoint::OfficialCratesIoEndpoint, wire::IdentifyingUserAgent};
pub use artifacts::ArtifactTransport;

/// Payload-free construction failure. No credential is accepted by constructors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BundledBuildError;
impl core::fmt::Display for BundledBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("official crates.io transport construction failed")
    }
}
impl core::error::Error for BundledBuildError {}

#[cfg(all(feature = "async-rustls", not(feature = "blocking-rustls")))]
pub use cloud_sdk_reqwest::asynchronous::RequestTimeouts;
#[cfg(feature = "blocking-rustls")]
pub use cloud_sdk_reqwest::blocking::RequestTimeouts;

#[cfg(feature = "blocking-rustls")]
fn build_blocking(
    endpoint: OfficialCratesIoEndpoint,
    identity: IdentifyingUserAgent<'_>,
    timeouts: RequestTimeouts,
) -> Result<cloud_sdk_reqwest::blocking::RawBlockingClient, BundledBuildError> {
    use cloud_sdk_reqwest::blocking::{HttpsEndpoint, RawBlockingClientBuilder, UserAgent};
    RawBlockingClientBuilder::new(
        HttpsEndpoint::new_with_policy(
            endpoint.base_url(),
            endpoint.policy().map_err(|_| BundledBuildError)?,
        )
        .map_err(|_| BundledBuildError)?,
        UserAgent::new(identity.as_str()).map_err(|_| BundledBuildError)?,
        timeouts,
    )
    .build()
    .map_err(|_| BundledBuildError)
}

/// Fixed production API transport. Use with `RegistryClient::production`.
#[cfg(feature = "blocking-rustls")]
pub fn production_blocking(
    identity: IdentifyingUserAgent<'_>,
    timeouts: RequestTimeouts,
) -> Result<cloud_sdk_reqwest::blocking::RawBlockingClient, BundledBuildError> {
    build_blocking(
        OfficialCratesIoEndpoint::production_api(),
        identity,
        timeouts,
    )
}

/// Fixed staging API transport, never an operator/tenant-supplied destination.
#[cfg(feature = "blocking-rustls")]
pub fn staging_blocking(
    identity: IdentifyingUserAgent<'_>,
    timeouts: RequestTimeouts,
) -> Result<cloud_sdk_reqwest::blocking::RawBlockingClient, BundledBuildError> {
    build_blocking(OfficialCratesIoEndpoint::staging_api(), identity, timeouts)
}

#[cfg(feature = "async-rustls")]
fn build_async(
    endpoint: OfficialCratesIoEndpoint,
    identity: IdentifyingUserAgent<'_>,
    timeouts: RequestTimeouts,
) -> Result<cloud_sdk_reqwest::asynchronous::RawAsyncClient, BundledBuildError> {
    use cloud_sdk_reqwest::asynchronous::{HttpsEndpoint, RawAsyncClientBuilder, UserAgent};
    RawAsyncClientBuilder::new(
        HttpsEndpoint::new_with_policy(
            endpoint.base_url(),
            endpoint.policy().map_err(|_| BundledBuildError)?,
        )
        .map_err(|_| BundledBuildError)?,
        UserAgent::new(identity.as_str()).map_err(|_| BundledBuildError)?,
        timeouts,
    )
    .build()
    .map_err(|_| BundledBuildError)
}

/// Fixed production API async transport; polling requires a Tokio executor.
#[cfg(feature = "async-rustls")]
pub fn production_async(
    identity: IdentifyingUserAgent<'_>,
    timeouts: RequestTimeouts,
) -> Result<cloud_sdk_reqwest::asynchronous::RawAsyncClient, BundledBuildError> {
    build_async(
        OfficialCratesIoEndpoint::production_api(),
        identity,
        timeouts,
    )
}

/// Fixed staging API async transport with no credential installed implicitly.
#[cfg(feature = "async-rustls")]
pub fn staging_async(
    identity: IdentifyingUserAgent<'_>,
    timeouts: RequestTimeouts,
) -> Result<cloud_sdk_reqwest::asynchronous::RawAsyncClient, BundledBuildError> {
    build_async(OfficialCratesIoEndpoint::staging_api(), identity, timeouts)
}
