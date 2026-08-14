use alloc::string::ToString;
use core::cmp::Ordering;
use core::net::IpAddr;
use core::str::FromStr;

use cloud_sdk_sanitization::SecretBoxBytes;

/// Maximum UTF-8 bytes admitted in a Robot vSwitch name.
pub const MAX_ROBOT_VSWITCH_NAME_BYTES: usize = 128;
/// Maximum server selectors admitted in one membership mutation.
pub const MAX_ROBOT_VSWITCH_SERVERS_PER_REQUEST: usize = 256;

/// Failure while validating a Robot vSwitch value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotVSwitchValueError {
    /// A provider identifier was zero.
    InvalidId,
    /// The VLAN identifier was outside the standard usable range.
    InvalidVlan,
    /// A name was empty, oversized, or unsafe for diagnostics.
    InvalidName,
    /// A server selector was not a canonical positive number or IP address.
    InvalidServer,
    /// A membership request had no selectors or exceeded its local bound.
    InvalidServerCount,
    /// A membership request repeated the same canonical selector.
    DuplicateServer,
    /// Protected storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotVSwitchValueError,
    Self::InvalidId => "Robot vSwitch identifier is invalid",
    Self::InvalidVlan => "Robot vSwitch VLAN identifier is invalid",
    Self::InvalidName => "Robot vSwitch name is invalid",
    Self::InvalidServer => "Robot vSwitch server selector is invalid",
    Self::InvalidServerCount => "Robot vSwitch server selector count is invalid",
    Self::DuplicateServer => "Robot vSwitch server selector is duplicated",
    Self::Allocation => "Robot vSwitch protected allocation failed",
);

/// Non-zero Robot vSwitch identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotVSwitchId(u64);

impl RobotVSwitchId {
    /// Creates a non-zero provider identifier.
    pub const fn new(value: u64) -> Result<Self, RobotVSwitchValueError> {
        if value == 0 {
            Err(RobotVSwitchValueError::InvalidId)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the provider identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Standard usable IEEE 802.1Q VLAN identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotVlanId(u16);

impl RobotVlanId {
    /// Admits VLAN IDs `1..=4094`; Robot may enforce a narrower account policy.
    pub const fn new(value: u16) -> Result<Self, RobotVSwitchValueError> {
        if value == 0 || value == 4095 {
            Err(RobotVSwitchValueError::InvalidVlan)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the VLAN identifier.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Protected validated Robot vSwitch name.
pub struct RobotVSwitchName(SecretBoxBytes);

impl RobotVSwitchName {
    /// Copies bounded display-safe text into protected owned storage.
    pub fn new(value: &str) -> Result<Self, RobotVSwitchValueError> {
        if value.is_empty()
            || value.len() > MAX_ROBOT_VSWITCH_NAME_BYTES
            || value.starts_with(char::is_whitespace)
            || value.ends_with(char::is_whitespace)
            || value
                .chars()
                .any(crate::display::is_unsafe_display_character)
        {
            return Err(RobotVSwitchValueError::InvalidName);
        }
        SecretBoxBytes::try_from_slice(value.as_bytes(), MAX_ROBOT_VSWITCH_NAME_BYTES)
            .map(Self)
            .map_err(|_| RobotVSwitchValueError::Allocation)
    }

    /// Runs a closure with temporary access to the exact name.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0
            .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
    }

    pub(super) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        self.0.with_secret(|bytes| {
            let text = core::str::from_utf8(bytes)
                .unwrap_or_else(|_| unreachable!("protected vSwitch name lost UTF-8"));
            inspect(text)
        })
    }
}

impl PartialEq for RobotVSwitchName {
    fn eq(&self, other: &Self) -> bool {
        other.0.with_secret(|right| self.0.constant_time_eq(right))
    }
}
impl Eq for RobotVSwitchName {}
impl core::fmt::Debug for RobotVSwitchName {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotVSwitchName([redacted])")
    }
}

/// Canonical borrowed Robot server number or IP selector.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotVSwitchServerIdentifier<'a>(&'a str);

impl<'a> RobotVSwitchServerIdentifier<'a> {
    /// Validates a canonical positive decimal server number or canonical IP.
    pub fn new(value: &'a str) -> Result<Self, RobotVSwitchValueError> {
        if valid_server_number(value) || valid_ip(value) {
            Ok(Self(value))
        } else {
            Err(RobotVSwitchValueError::InvalidServer)
        }
    }

    pub(super) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl Ord for RobotVSwitchServerIdentifier<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}
impl PartialOrd for RobotVSwitchServerIdentifier<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl core::fmt::Debug for RobotVSwitchServerIdentifier<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotVSwitchServerIdentifier([redacted])")
    }
}

/// Non-empty bounded membership selector snapshot.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotVSwitchServers<'a>(&'a [RobotVSwitchServerIdentifier<'a>]);

impl<'a> RobotVSwitchServers<'a> {
    /// Validates selector count and exact canonical uniqueness.
    pub fn new(
        values: &'a [RobotVSwitchServerIdentifier<'a>],
    ) -> Result<Self, RobotVSwitchValueError> {
        if values.is_empty() || values.len() > MAX_ROBOT_VSWITCH_SERVERS_PER_REQUEST {
            return Err(RobotVSwitchValueError::InvalidServerCount);
        }
        for (index, value) in values.iter().enumerate() {
            if values
                .get(..index)
                .is_some_and(|prior| prior.contains(value))
            {
                return Err(RobotVSwitchValueError::DuplicateServer);
            }
        }
        Ok(Self(values))
    }

    /// Returns the validated selector count.
    #[must_use]
    pub const fn len(self) -> usize {
        self.0.len()
    }

    /// Reports whether the selector set is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    pub(super) const fn as_slice(self) -> &'a [RobotVSwitchServerIdentifier<'a>] {
        self.0
    }
}

impl core::fmt::Debug for RobotVSwitchServers<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotVSwitchServers")
            .field("len", &self.0.len())
            .field("values", &"[redacted]")
            .finish()
    }
}

fn valid_server_number(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value.as_bytes().first() != Some(&b'0')
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok_and(|number| number != 0)
}

fn valid_ip(value: &str) -> bool {
    IpAddr::from_str(value).is_ok_and(|address| address.to_string() == value)
}
