use core::fmt;

use cloud_sdk::transport::{RequestTarget, RequestTargetError};

/// A validated crates.io API request target under `/api/v1/`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ApiRequestTarget<'a> {
    target: RequestTarget<'a>,
}

impl<'a> ApiRequestTarget<'a> {
    /// Validates a bounded canonical crates.io API target.
    pub fn new(value: &'a str) -> Result<Self, CratesIoTargetError> {
        let target = RequestTarget::new(value).map_err(CratesIoTargetError::InvalidTarget)?;
        if !target.path().as_str().starts_with("/api/v1/") {
            return Err(CratesIoTargetError::OutsideApiNamespace);
        }
        Ok(Self { target })
    }

    /// Returns the provider-neutral target.
    #[must_use]
    pub const fn as_request_target(self) -> RequestTarget<'a> {
        self.target
    }

    /// Returns the exact validated target text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.target.as_str()
    }
}

impl fmt::Debug for ApiRequestTarget<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiRequestTarget([redacted])")
    }
}

/// A validated anonymous target below `static.crates.io/crates/`.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct StaticDownloadTarget<'a> {
    target: RequestTarget<'a>,
}

impl<'a> StaticDownloadTarget<'a> {
    /// Validates the bounded static package-download path shape.
    pub fn new(value: &'a str) -> Result<Self, CratesIoTargetError> {
        let target = RequestTarget::new(value).map_err(CratesIoTargetError::InvalidTarget)?;
        if target.query().is_present() {
            return Err(CratesIoTargetError::StaticQueryForbidden);
        }
        if static_parts(target.path().as_str()).is_none() {
            return Err(CratesIoTargetError::InvalidStaticDownloadPath);
        }
        Ok(Self { target })
    }

    /// Returns the provider-neutral target.
    #[must_use]
    pub const fn as_request_target(self) -> RequestTarget<'a> {
        self.target
    }

    /// Returns the exact validated target text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.target.as_str()
    }

    pub(crate) fn parts(self) -> Option<(&'a str, &'a str)> {
        static_parts(self.target.path().as_str())
    }
}

impl fmt::Debug for StaticDownloadTarget<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("StaticDownloadTarget([redacted])")
    }
}

/// crates.io request-target validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CratesIoTargetError {
    /// The provider-neutral origin-form target was rejected.
    InvalidTarget(RequestTargetError),
    /// API targets must remain under the public `/api/v1/` namespace.
    OutsideApiNamespace,
    /// Static package downloads never carry a query.
    StaticQueryForbidden,
    /// Static package downloads must use `/crates/{name}/{archive}.crate`.
    InvalidStaticDownloadPath,
}

impl fmt::Display for CratesIoTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTarget(error) => write!(formatter, "invalid crates.io target: {error}"),
            Self::OutsideApiNamespace => {
                formatter.write_str("crates.io target is outside the public API namespace")
            }
            Self::StaticQueryForbidden => {
                formatter.write_str("static crates.io download query is forbidden")
            }
            Self::InvalidStaticDownloadPath => {
                formatter.write_str("static crates.io download path is invalid")
            }
        }
    }
}

impl core::error::Error for CratesIoTargetError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidTarget(error) => Some(error),
            Self::OutsideApiNamespace
            | Self::StaticQueryForbidden
            | Self::InvalidStaticDownloadPath => None,
        }
    }
}

fn static_parts(path: &str) -> Option<(&str, &str)> {
    let remainder = path.strip_prefix("/crates/")?;
    let (name, archive) = remainder.split_once('/')?;
    if name.is_empty()
        || archive.is_empty()
        || archive.contains('/')
        || !archive.ends_with(".crate")
    {
        return None;
    }
    Some((name, archive))
}
