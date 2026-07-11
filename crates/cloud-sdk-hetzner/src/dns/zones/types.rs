//! Validated DNS Zone request values.

mod tsig;

pub use tsig::{
    MAX_TSIG_KEY_BYTES, MIN_TSIG_SECRET_BYTES, TsigAlgorithm, TsigCredentials, TsigKey,
};

use core::fmt;
use core::net::IpAddr;
use core::str::FromStr;

use cloud_sdk::buffer;

use crate::cloud::public_ip::{invalid_public_v4, invalid_public_v6};
use crate::cloud::shared::{CloudLabels, CloudRequestError, CloudResourceId};
use crate::request::EndpointPath;

/// Maximum zone-file bytes admitted by this SDK boundary.
pub const MAX_ZONE_FILE_BYTES: usize = 1024 * 1024;
/// Maximum primary nameservers admitted by this SDK boundary.
pub const MAX_PRIMARY_NAMESERVERS: usize = 32;

/// DNS Zone request error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneRequestError {
    /// A shared Cloud request operation failed.
    Cloud(CloudRequestError),
    /// A required request field was omitted.
    MissingRequiredField,
    /// Zone name validation failed.
    InvalidZoneName,
    /// TTL must be between 60 and 2,147,483,647 seconds.
    InvalidTtl,
    /// Zone-file validation failed.
    InvalidZoneFile,
    /// A body output buffer is too small.
    BodyBufferTooSmall,
    /// Primary nameserver address is not an ordinary public IP address.
    InvalidNameserverAddress,
    /// Primary nameserver port must be nonzero.
    InvalidNameserverPort,
    /// TSIG key validation failed.
    InvalidTsigKey,
    /// At least one primary nameserver is required.
    EmptyPrimaryNameservers,
    /// Primary nameserver count exceeds SDK policy.
    TooManyPrimaryNameservers,
    /// Primary nameserver addresses must be unique.
    DuplicatePrimaryNameserver,
    /// A field is incompatible with the selected zone mode.
    InvalidModeConfiguration,
    /// A global-only action filter was used for a resource-local list.
    InvalidActionFilter,
}

impl From<CloudRequestError> for ZoneRequestError {
    fn from(value: CloudRequestError) -> Self {
        Self::Cloud(value)
    }
}

/// Zone numeric identifier.
pub type ZoneId = CloudResourceId;
/// Zone action identifier.
pub type ZoneActionId = crate::actions::ActionId;
/// Zone labels.
pub type ZoneLabels<'a> = CloudLabels<'a>;

/// Lowercase DNS zone name accepted as an ID-or-name path reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneName<'a>(&'a str);

impl<'a> ZoneName<'a> {
    /// Validates a conservative ASCII or ACE-encoded zone name.
    pub fn new(value: &'a str) -> Result<Self, ZoneRequestError> {
        if value.is_empty()
            || value.len() > 253
            || value.starts_with('.')
            || value.ends_with('.')
            || !value.contains('.')
        {
            return Err(ZoneRequestError::InvalidZoneName);
        }
        for label in value.split('.') {
            if label.is_empty()
                || label.len() > 63
                || !label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                || !label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
                || label.bytes().any(|byte| {
                    !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                })
            {
                return Err(ZoneRequestError::InvalidZoneName);
            }
        }
        Ok(Self(value))
    }

    /// Returns the validated zone name.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Zone path selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneReference<'a> {
    /// Select by numeric ID.
    Id(ZoneId),
    /// Select by zone name.
    Name(ZoneName<'a>),
}

/// Zone operating mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneMode {
    /// Hetzner hosts the primary zone.
    Primary,
    /// Hetzner transfers from caller-owned primary nameservers.
    Secondary,
}

impl ZoneMode {
    /// Returns the source-locked API value.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
        }
    }
}

/// Zone default TTL in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneTtl(u32);

impl ZoneTtl {
    /// Creates a TTL in the source-locked API range.
    pub const fn new(value: u32) -> Result<Self, ZoneRequestError> {
        if value < 60 || value > i32::MAX as u32 {
            return Err(ZoneRequestError::InvalidTtl);
        }
        Ok(Self(value))
    }

    /// Returns seconds.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Bounded zone-file body. Debug output is redacted and ordinary equality is
/// intentionally unavailable because zone files can contain secret material.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::dns::zones::ZoneFile;
/// fn insecure_compare(left: ZoneFile<'_>, right: ZoneFile<'_>) -> bool {
///     left == right
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ZoneFile<'a>(&'a str);

impl<'a> ZoneFile<'a> {
    /// Creates a nonempty, NUL-free zone file.
    pub fn new(value: &'a str) -> Result<Self, ZoneRequestError> {
        if value.is_empty() || value.len() > MAX_ZONE_FILE_BYTES || value.as_bytes().contains(&0) {
            return Err(ZoneRequestError::InvalidZoneFile);
        }
        Ok(Self(value))
    }

    /// Writes the complete JSON string without exposing a raw accessor.
    ///
    /// The caller owns `output` and must securely erase it after transport use.
    pub fn write_json_string(self, output: &mut [u8]) -> Result<usize, ZoneRequestError> {
        let mut len = 0;
        buffer::write_json_string(
            output,
            &mut len,
            self.0,
            ZoneRequestError::BodyBufferTooSmall,
        )?;
        Ok(len)
    }
}

impl fmt::Debug for ZoneFile<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ZoneFile([redacted])")
    }
}

/// Public primary nameserver used by a secondary zone.
///
/// Ordinary equality is unavailable because this value can contain TSIG
/// credentials.
#[derive(Clone, Copy, Debug)]
pub struct PrimaryNameserver<'a> {
    value: &'a str,
    address: IpAddr,
    port: u16,
    tsig: Option<TsigCredentials<'a>>,
}

impl<'a> PrimaryNameserver<'a> {
    /// Creates a nameserver on the default DNS port 53.
    pub fn new(value: &'a str) -> Result<Self, ZoneRequestError> {
        let address =
            IpAddr::from_str(value).map_err(|_| ZoneRequestError::InvalidNameserverAddress)?;
        let invalid = match address {
            IpAddr::V4(address) => invalid_public_v4(address),
            IpAddr::V6(address) => invalid_public_v6(address),
        };
        if invalid {
            return Err(ZoneRequestError::InvalidNameserverAddress);
        }
        Ok(Self {
            value,
            address,
            port: 53,
            tsig: None,
        })
    }

    /// Sets a nonzero transfer port.
    pub const fn with_port(mut self, port: u16) -> Result<Self, ZoneRequestError> {
        if port == 0 {
            return Err(ZoneRequestError::InvalidNameserverPort);
        }
        self.port = port;
        Ok(self)
    }

    /// Sets coherent TSIG credentials.
    #[must_use]
    pub const fn with_tsig(mut self, tsig: TsigCredentials<'a>) -> Self {
        self.tsig = Some(tsig);
        self
    }

    /// Returns the validated address text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
    /// Returns the transfer port.
    #[must_use]
    pub const fn port(self) -> u16 {
        self.port
    }
    /// Returns TSIG credentials when configured.
    #[must_use]
    pub const fn tsig(self) -> Option<TsigCredentials<'a>> {
        self.tsig
    }
}

/// Nonempty, unique primary nameserver list without ordinary equality.
#[derive(Clone, Copy, Debug)]
pub struct PrimaryNameservers<'a>(&'a [PrimaryNameserver<'a>]);

impl<'a> PrimaryNameservers<'a> {
    /// Validates list size and unique addresses.
    pub fn new(values: &'a [PrimaryNameserver<'a>]) -> Result<Self, ZoneRequestError> {
        if values.is_empty() {
            return Err(ZoneRequestError::EmptyPrimaryNameservers);
        }
        if values.len() > MAX_PRIMARY_NAMESERVERS {
            return Err(ZoneRequestError::TooManyPrimaryNameservers);
        }
        for (index, value) in values.iter().enumerate() {
            let previous_values = values
                .get(..index)
                .ok_or(ZoneRequestError::DuplicatePrimaryNameserver)?;
            if previous_values
                .iter()
                .any(|previous| previous.address == value.address)
            {
                return Err(ZoneRequestError::DuplicatePrimaryNameserver);
            }
        }
        Ok(Self(values))
    }

    /// Returns the validated nameservers.
    #[must_use]
    pub const fn entries(self) -> &'a [PrimaryNameserver<'a>] {
        self.0
    }
}

pub(crate) fn write_zone_path(
    output: &mut [u8],
    zone: ZoneReference<'_>,
    suffix: &str,
) -> Result<usize, ZoneRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        "/zones/",
        CloudRequestError::PathBufferTooSmall,
    )?;
    match zone {
        ZoneReference::Id(id) => buffer::write_u64(
            output,
            &mut len,
            id.get(),
            CloudRequestError::PathBufferTooSmall,
        )?,
        ZoneReference::Name(name) => buffer::write_str(
            output,
            &mut len,
            name.as_str(),
            CloudRequestError::PathBufferTooSmall,
        )?,
    }
    buffer::write_str(
        output,
        &mut len,
        suffix,
        CloudRequestError::PathBufferTooSmall,
    )?;
    let path = core::str::from_utf8(
        output
            .get(..len)
            .ok_or(CloudRequestError::PathBufferTooSmall)?,
    )
    .map_err(|_| CloudRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(CloudRequestError::InvalidPath)?;
    Ok(len)
}
