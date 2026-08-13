use alloc::vec::Vec;
use core::fmt;
use core::net::{Ipv4Addr, Ipv6Addr};

use crate::robot::{RobotIpAddress, RobotServerNumber};
use crate::serde::SensitiveText;

/// Boot configuration family selected by a Robot operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotBootFamily {
    /// Temporary Rescue System.
    Rescue,
    /// Automated Linux installation.
    Linux,
    /// VNC-based installation.
    Vnc,
    /// Automated Windows installation.
    Windows,
}

/// Protected provider text such as a password, key, or selector.
pub struct RobotBootSecret(pub(super) SensitiveText);

impl RobotBootSecret {
    /// Runs a closure with temporary access to the protected text.
    pub fn try_with_secret<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }
}

impl fmt::Debug for RobotBootSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotBootSecret([redacted])")
    }
}

/// Available choices or the exact currently selected choice.
pub enum RobotBootChoice {
    /// Bounded provider-advertised options.
    Available(Vec<RobotBootSecret>),
    /// Exact selected value from an activation or last-operation response.
    Selected(RobotBootSecret),
}

impl RobotBootChoice {
    /// Returns the number of advertised values, or one for a selection.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Available(values) => values.len(),
            Self::Selected(_) => 1,
        }
    }

    /// Reports whether the available list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Available(values) if values.is_empty())
    }

    /// Returns one protected value by index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&RobotBootSecret> {
        match self {
            Self::Available(values) => values.get(index),
            Self::Selected(value) if index == 0 => Some(value),
            Self::Selected(_) => None,
        }
    }

    /// Reports whether this is one exact selected value.
    #[must_use]
    pub const fn is_selected(&self) -> bool {
        matches!(self, Self::Selected(_))
    }
}

impl fmt::Debug for RobotBootChoice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotBootChoice")
            .field("values", &self.len())
            .field("selected", &self.is_selected())
            .finish()
    }
}

/// Strict state returned for one boot family.
pub struct RobotBootEntry {
    pub(super) family: RobotBootFamily,
    pub(super) server_ipv4: RobotIpAddress,
    pub(super) server_ipv6_network: RobotIpAddress,
    pub(super) number: RobotServerNumber,
    pub(super) primary: RobotBootChoice,
    pub(super) languages: Option<RobotBootChoice>,
    pub(super) active: bool,
    pub(super) password: Option<RobotBootSecret>,
    pub(super) authorized_keys: Vec<RobotBootSecret>,
    pub(super) host_keys: Vec<RobotBootSecret>,
}

impl RobotBootEntry {
    /// Returns this entry's source-locked family.
    #[must_use]
    pub const fn family(&self) -> RobotBootFamily {
        self.family
    }

    /// Returns the canonical server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.number
    }

    /// Runs a closure with the canonical server IPv4 address.
    pub fn with_server_ipv4<R>(&self, inspect: impl FnOnce(Ipv4Addr) -> R) -> R {
        self.server_ipv4.with_addr(|address| match address {
            core::net::IpAddr::V4(address) => inspect(address),
            core::net::IpAddr::V6(_) => unreachable!("validated Robot boot IPv4 changed family"),
        })
    }

    /// Runs a closure with the canonical server IPv6 network address.
    pub fn with_server_ipv6_network<R>(&self, inspect: impl FnOnce(Ipv6Addr) -> R) -> R {
        self.server_ipv6_network.with_addr(|address| match address {
            core::net::IpAddr::V6(address) => inspect(address),
            core::net::IpAddr::V4(_) => unreachable!("validated Robot boot IPv6 changed family"),
        })
    }

    /// Returns OS choices for Rescue/Windows or distribution choices otherwise.
    #[must_use]
    pub const fn primary_choice(&self) -> &RobotBootChoice {
        &self.primary
    }

    /// Returns language choices for Linux, VNC, and Windows.
    #[must_use]
    pub const fn languages(&self) -> Option<&RobotBootChoice> {
        self.languages.as_ref()
    }

    /// Reports whether this configuration is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Returns a protected generated password when the provider supplied one.
    #[must_use]
    pub const fn password(&self) -> Option<&RobotBootSecret> {
        self.password.as_ref()
    }

    /// Returns protected authorized SSH keys or fingerprints.
    #[must_use]
    pub fn authorized_keys(&self) -> &[RobotBootSecret] {
        &self.authorized_keys
    }

    /// Returns protected host keys.
    #[must_use]
    pub fn host_keys(&self) -> &[RobotBootSecret] {
        &self.host_keys
    }
}

impl fmt::Debug for RobotBootEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotBootEntry")
            .field("family", &self.family)
            .field("identity", &"[redacted]")
            .field("active", &self.active)
            .field("password", &self.password.is_some())
            .field("authorized_keys", &self.authorized_keys.len())
            .field("host_keys", &self.host_keys.len())
            .finish()
    }
}

/// Complete four-family boot overview for one server.
pub struct RobotBoot {
    pub(super) rescue: RobotBootEntry,
    pub(super) linux: RobotBootEntry,
    pub(super) vnc: RobotBootEntry,
    pub(super) windows: RobotBootEntry,
}

impl RobotBoot {
    /// Returns Rescue boot state.
    #[must_use]
    pub const fn rescue(&self) -> &RobotBootEntry {
        &self.rescue
    }
    /// Returns Linux installation state.
    #[must_use]
    pub const fn linux(&self) -> &RobotBootEntry {
        &self.linux
    }
    /// Returns VNC installation state.
    #[must_use]
    pub const fn vnc(&self) -> &RobotBootEntry {
        &self.vnc
    }
    /// Returns Windows installation state.
    #[must_use]
    pub const fn windows(&self) -> &RobotBootEntry {
        &self.windows
    }
}

impl fmt::Debug for RobotBoot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotBoot([redacted])")
    }
}
