//! crates.io endpoint, request-target, and redirect trust boundaries.

mod authority;
mod redirect;
mod target;

pub use authority::{
    AcknowledgedCustomApiEndpoint, CRATES_IO_API_BASE_URL, CRATES_IO_STAGING_API_BASE_URL,
    CRATES_IO_STATIC_DOWNLOAD_BASE_URL, CratesIoEndpointError, OfficialCratesIoEndpoint,
    OfficialEndpointPurpose,
};
pub use redirect::{
    DownloadRedirect, DownloadRedirectError, MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES,
    RedirectAuthorization,
};
pub use target::{ApiRequestTarget, CratesIoTargetError, StaticDownloadTarget};

#[cfg(test)]
mod tests;
