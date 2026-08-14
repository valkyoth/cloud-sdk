use core::cmp::Ordering;
use core::net::IpAddr;
use core::str::FromStr;

use cloud_sdk_sanitization::SecretBoxBytes;

use crate::robot::canonical::display_matches;
use crate::robot::server::protected_parse::{self, AddressFamily, ProtectedValueError};

/// Failure while constructing a protected Robot cancellation value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotCancellationValueError {
    /// The value is malformed or not in canonical provider form.
    Invalid,
    /// Stable protected storage could not be allocated.
    Allocation,
}

impl core::fmt::Display for RobotCancellationValueError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Invalid => "Robot cancellation value is invalid",
            Self::Allocation => "Robot cancellation value allocation failed",
        })
    }
}

impl core::error::Error for RobotCancellationValueError {}

fn map_error(error: ProtectedValueError) -> RobotCancellationValueError {
    match error {
        ProtectedValueError::Invalid => RobotCancellationValueError::Invalid,
        ProtectedValueError::Allocation => RobotCancellationValueError::Allocation,
    }
}

fn protected_text(
    value: &str,
    maximum: usize,
) -> Result<SecretBoxBytes, RobotCancellationValueError> {
    SecretBoxBytes::try_from_slice(value.as_bytes(), maximum)
        .map_err(|_| RobotCancellationValueError::Allocation)
}

fn protected_cmp(left: &SecretBoxBytes, right: &SecretBoxBytes) -> Ordering {
    left.with_secret(|left| right.with_secret(|right| left.cmp(right)))
}

/// Canonical protected IP identity accepted by Robot cancellation paths.
pub struct RobotIpAddress(SecretBoxBytes);

impl RobotIpAddress {
    /// Parses an address and requires Rust's canonical display spelling.
    pub fn new(value: &str) -> Result<Self, RobotCancellationValueError> {
        let parsed = protected_parse::address(value, AddressFamily::Any).map_err(map_error)?;
        let canonical =
            IpAddr::from_str(value).map_err(|_| RobotCancellationValueError::Invalid)?;
        if !display_matches(value, canonical) {
            return Err(RobotCancellationValueError::Invalid);
        }
        drop(parsed);
        protected_text(value, 39).map(Self)
    }

    /// Runs a closure with temporary access to the canonical address.
    pub fn with_addr<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.0.with_secret(|bytes| {
            let text = core::str::from_utf8(bytes)
                .unwrap_or_else(|_| unreachable!("protected Robot address lost UTF-8"));
            let value = IpAddr::from_str(text)
                .unwrap_or_else(|_| unreachable!("protected Robot address became invalid"));
            inspect(value)
        })
    }

    pub(crate) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        self.0.with_secret(|bytes| {
            let text = core::str::from_utf8(bytes)
                .unwrap_or_else(|_| unreachable!("protected Robot address lost UTF-8"));
            inspect(text)
        })
    }
}

impl PartialEq for RobotIpAddress {
    fn eq(&self, other: &Self) -> bool {
        protected_cmp(&self.0, &other.0) == Ordering::Equal
    }
}
impl Eq for RobotIpAddress {}
impl core::fmt::Debug for RobotIpAddress {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotIpAddress([redacted])")
    }
}

/// Canonical protected subnet route identity accepted by Robot paths.
pub struct RobotSubnetAddress(RobotIpAddress);

impl RobotSubnetAddress {
    /// Creates a subnet-path identity from its canonical address spelling.
    pub fn new(value: &str) -> Result<Self, RobotCancellationValueError> {
        RobotIpAddress::new(value).map(Self)
    }

    /// Runs a closure with temporary access to the canonical route identity.
    pub fn with_addr<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.0.with_addr(inspect)
    }

    pub(crate) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        self.0.with_text(inspect)
    }
}

impl PartialEq for RobotSubnetAddress {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for RobotSubnetAddress {}
impl core::fmt::Debug for RobotSubnetAddress {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSubnetAddress([redacted])")
    }
}

/// Calendar-valid protected Robot cancellation date.
pub struct RobotCancellationDate(SecretBoxBytes);

impl RobotCancellationDate {
    /// Parses an exact `YYYY-MM-DD` date.
    pub fn new(value: &str) -> Result<Self, RobotCancellationValueError> {
        let parsed = protected_parse::date(value).map_err(map_error)?;
        drop(parsed);
        protected_text(value, 10).map(Self)
    }

    /// Runs a closure with temporary access to the exact date text.
    pub fn try_with_date<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0
            .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
    }

    pub(crate) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        self.0.with_secret(|bytes| {
            let text = core::str::from_utf8(bytes)
                .unwrap_or_else(|_| unreachable!("protected Robot date lost UTF-8"));
            inspect(text)
        })
    }
}

impl PartialEq for RobotCancellationDate {
    fn eq(&self, other: &Self) -> bool {
        protected_cmp(&self.0, &other.0) == Ordering::Equal
    }
}
impl Eq for RobotCancellationDate {}
impl PartialOrd for RobotCancellationDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RobotCancellationDate {
    fn cmp(&self, other: &Self) -> Ordering {
        protected_cmp(&self.0, &other.0)
    }
}
impl core::fmt::Debug for RobotCancellationDate {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotCancellationDate([redacted])")
    }
}
