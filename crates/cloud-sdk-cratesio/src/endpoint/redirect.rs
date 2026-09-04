use core::fmt;

use cloud_sdk::transport::MAX_REQUEST_TARGET_BYTES;

use super::{
    ApiRequestTarget, CRATES_IO_STATIC_DOWNLOAD_BASE_URL, CratesIoTargetError,
    OfficialCratesIoEndpoint, StaticDownloadTarget,
};

/// Maximum absolute static-download redirect accepted before target parsing.
pub const MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES: usize =
    CRATES_IO_STATIC_DOWNLOAD_BASE_URL.len() + MAX_REQUEST_TARGET_BYTES;

/// Authorization behavior for an accepted cross-authority redirect.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RedirectAuthorization {
    /// Follow without copying API authorization to the download request.
    Omit,
}

/// A source-correlated redirect to the official anonymous download authority.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct DownloadRedirect<'a> {
    destination: StaticDownloadTarget<'a>,
}

impl<'a> DownloadRedirect<'a> {
    /// Validates an absolute redirect from a production version-download route.
    ///
    /// Only the exact `https://static.crates.io` authority is admitted. The
    /// archive path must repeat the source crate name and version exactly.
    pub fn new(
        source_endpoint: OfficialCratesIoEndpoint,
        source: ApiRequestTarget<'_>,
        location: &'a str,
    ) -> Result<Self, DownloadRedirectError> {
        if source_endpoint != OfficialCratesIoEndpoint::production_api() {
            return Err(DownloadRedirectError::InvalidSourceEndpoint);
        }
        let (name, version) =
            source_parts(source).ok_or(DownloadRedirectError::InvalidSourcePath)?;
        if location.len() > MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES {
            return Err(DownloadRedirectError::LocationTooLong);
        }
        let target = location
            .strip_prefix(CRATES_IO_STATIC_DOWNLOAD_BASE_URL)
            .ok_or(DownloadRedirectError::DestinationMismatch)?;
        let destination =
            StaticDownloadTarget::new(target).map_err(DownloadRedirectError::InvalidTarget)?;
        let (directory, archive) = destination
            .parts()
            .ok_or(DownloadRedirectError::ArchiveMismatch)?;
        if directory != name || !archive_matches(archive, name, version) {
            return Err(DownloadRedirectError::ArchiveMismatch);
        }
        Ok(Self { destination })
    }

    /// Returns the fixed anonymous download endpoint.
    #[must_use]
    pub const fn endpoint(self) -> OfficialCratesIoEndpoint {
        OfficialCratesIoEndpoint::static_downloads()
    }

    /// Returns the validated static download target.
    #[must_use]
    pub const fn target(self) -> StaticDownloadTarget<'a> {
        self.destination
    }

    /// Requires callers to omit API authorization on the redirected request.
    #[must_use]
    pub const fn authorization(self) -> RedirectAuthorization {
        RedirectAuthorization::Omit
    }
}

impl fmt::Debug for DownloadRedirect<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DownloadRedirect([redacted])")
    }
}

/// crates.io static-download redirect validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DownloadRedirectError {
    /// Only the production API can redirect into the production static origin.
    InvalidSourceEndpoint,
    /// The source is not an exact version-download API target.
    InvalidSourcePath,
    /// The absolute redirect exceeds the bounded location size.
    LocationTooLong,
    /// The redirect does not use the exact official static authority.
    DestinationMismatch,
    /// The redirect target failed canonical target validation.
    InvalidTarget(CratesIoTargetError),
    /// The destination crate name or archive does not match the source route.
    ArchiveMismatch,
}

impl fmt::Display for DownloadRedirectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSourceEndpoint => {
                formatter.write_str("download redirect source endpoint is invalid")
            }
            Self::InvalidSourcePath => {
                formatter.write_str("download redirect source path is invalid")
            }
            Self::LocationTooLong => {
                formatter.write_str("download redirect location exceeds the length limit")
            }
            Self::DestinationMismatch => {
                formatter.write_str("download redirect destination is not official")
            }
            Self::InvalidTarget(error) => write!(formatter, "invalid download redirect: {error}"),
            Self::ArchiveMismatch => {
                formatter.write_str("download redirect archive does not match its source")
            }
        }
    }
}

impl core::error::Error for DownloadRedirectError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidTarget(error) => Some(error),
            Self::InvalidSourceEndpoint
            | Self::InvalidSourcePath
            | Self::LocationTooLong
            | Self::DestinationMismatch
            | Self::ArchiveMismatch => None,
        }
    }
}

fn source_parts(source: ApiRequestTarget<'_>) -> Option<(&str, &str)> {
    if source.as_request_target().query().is_present() {
        return None;
    }
    let remainder = source
        .as_request_target()
        .path()
        .as_str()
        .strip_prefix("/api/v1/crates/")?;
    let mut segments = remainder.split('/');
    let name = segments.next()?;
    let version = segments.next()?;
    let operation = segments.next()?;
    if name.is_empty() || version.is_empty() || operation != "download" || segments.next().is_some()
    {
        return None;
    }
    Some((name, version))
}

fn archive_matches(archive: &str, name: &str, version: &str) -> bool {
    archive
        .strip_suffix(".crate")
        .and_then(|stem| stem.strip_prefix(name))
        .and_then(|suffix| suffix.strip_prefix('-'))
        == Some(version)
}
