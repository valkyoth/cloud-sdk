use core::net::Ipv4Addr;
use core::str::FromStr;

use cloud_sdk::transport::{StatusCode, TransportResponse};

use super::{MAX_METADATA_RESPONSE_BYTES, MetadataPrivateNetworks, MetadataRoute};

/// Strict decoded response for one canonical metadata route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataResponse<'a> {
    /// Complete scalar summary.
    Summary(MetadataSummary<'a>),
    /// Server hostname.
    Hostname(&'a str),
    /// Server numeric identifier.
    InstanceId(u64),
    /// Primary public IPv4 address.
    PublicIpv4(Ipv4Addr),
    /// Attached private networks.
    PrivateNetworks(MetadataPrivateNetworks<'a>),
    /// Availability zone.
    AvailabilityZone(&'a str),
    /// Network region.
    Region(&'a str),
}

/// Scalar summary returned by the root metadata route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataSummary<'a> {
    hostname: &'a str,
    instance_id: u64,
    public_ipv4: Ipv4Addr,
    availability_zone: &'a str,
    region: &'a str,
}

impl<'a> MetadataSummary<'a> {
    /// Returns the server hostname.
    #[must_use]
    pub const fn hostname(self) -> &'a str {
        self.hostname
    }

    /// Returns the server ID.
    #[must_use]
    pub const fn instance_id(self) -> u64 {
        self.instance_id
    }

    /// Returns the primary public IPv4 address.
    #[must_use]
    pub const fn public_ipv4(self) -> Ipv4Addr {
        self.public_ipv4
    }

    /// Returns the availability-zone name.
    #[must_use]
    pub const fn availability_zone(self) -> &'a str {
        self.availability_zone
    }

    /// Returns the network-region name.
    #[must_use]
    pub const fn region(self) -> &'a str {
        self.region
    }
}

/// Metadata status, syntax, bound, or semantic failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataDecodeError {
    /// The response status is not exactly 200.
    UnexpectedStatus(StatusCode),
    /// The response exceeds the provider-owned aggregate bound.
    ResponseTooLarge,
    /// The response is empty.
    Empty,
    /// The response is not UTF-8.
    InvalidUtf8,
    /// A scalar or YAML token contains forbidden syntax.
    InvalidSyntax,
    /// A required summary or network field is absent.
    MissingField,
    /// A summary or network field occurs more than once.
    DuplicateField,
    /// An unrecognized field is present.
    UnknownField,
    /// A scalar exceeds its field-specific length bound.
    FieldTooLong,
    /// A numeric value is invalid or overflows.
    InvalidNumber,
    /// An IPv4 address is invalid or non-canonical.
    InvalidIpv4,
    /// A private-network CIDR is invalid or non-canonical.
    InvalidCidr,
    /// A MAC address is invalid or non-canonical.
    InvalidMac,
    /// Private-network fields contradict one another.
    InconsistentNetwork,
    /// A private-network collection exceeds its fixed count bound.
    TooManyItems,
}

impl_static_error!(MetadataDecodeError,
    Self::UnexpectedStatus(_) => "metadata response status is not 200",
    Self::ResponseTooLarge => "metadata response exceeds the size limit",
    Self::Empty => "metadata response is empty",
    Self::InvalidUtf8 => "metadata response is not valid UTF-8",
    Self::InvalidSyntax => "metadata response syntax is invalid",
    Self::MissingField => "metadata response is missing a required field",
    Self::DuplicateField => "metadata response contains a duplicate field",
    Self::UnknownField => "metadata response contains an unknown field",
    Self::FieldTooLong => "metadata response field exceeds its length limit",
    Self::InvalidNumber => "metadata response number is invalid",
    Self::InvalidIpv4 => "metadata response IPv4 address is invalid",
    Self::InvalidCidr => "metadata response CIDR is invalid",
    Self::InvalidMac => "metadata response MAC address is invalid",
    Self::InconsistentNetwork => "metadata private-network fields are inconsistent",
    Self::TooManyItems => "metadata response contains too many items",
);

/// Checks status and strictly decodes the response selected by `route`.
pub fn decode_metadata_response<'a>(
    route: MetadataRoute,
    response: TransportResponse<'a, '_>,
) -> Result<MetadataResponse<'a>, MetadataDecodeError> {
    if response.status() != StatusCode::OK {
        return Err(MetadataDecodeError::UnexpectedStatus(response.status()));
    }
    decode_metadata_body(route, response.body())
}

/// Strictly decodes already status-checked body bytes for one route.
pub fn decode_metadata_body(
    route: MetadataRoute,
    body: &[u8],
) -> Result<MetadataResponse<'_>, MetadataDecodeError> {
    if body.len() > MAX_METADATA_RESPONSE_BYTES {
        return Err(MetadataDecodeError::ResponseTooLarge);
    }
    let text = core::str::from_utf8(body).map_err(|_| MetadataDecodeError::InvalidUtf8)?;
    match route {
        MetadataRoute::Summary => parse_summary(text).map(MetadataResponse::Summary),
        MetadataRoute::Hostname => {
            scalar(text, 253, valid_hostname).map(MetadataResponse::Hostname)
        }
        MetadataRoute::InstanceId => {
            parse_instance_id(scalar(text, 20, ascii_digits)?).map(MetadataResponse::InstanceId)
        }
        MetadataRoute::PublicIpv4 => {
            parse_ipv4(scalar(text, 15, ipv4_chars)?).map(MetadataResponse::PublicIpv4)
        }
        MetadataRoute::PrivateNetworks => {
            MetadataPrivateNetworks::new(text).map(MetadataResponse::PrivateNetworks)
        }
        MetadataRoute::AvailabilityZone => {
            scalar(text, 64, name_chars).map(MetadataResponse::AvailabilityZone)
        }
        MetadataRoute::Region => scalar(text, 64, name_chars).map(MetadataResponse::Region),
    }
}

fn parse_summary(text: &str) -> Result<MetadataSummary<'_>, MetadataDecodeError> {
    let text = document(text)?;
    let mut hostname = None;
    let mut instance_id = None;
    let mut public_ipv4 = None;
    let mut availability_zone = None;
    let mut region = None;
    for line in text.split('\n') {
        let (key, value) = line
            .split_once(": ")
            .ok_or(MetadataDecodeError::InvalidSyntax)?;
        if value.is_empty() || value.contains([':', '#', '[', ']', '{', '}']) {
            return Err(MetadataDecodeError::InvalidSyntax);
        }
        match key {
            "hostname" => set(&mut hostname, scalar(value, 253, valid_hostname)?),
            "instance-id" => set(&mut instance_id, parse_instance_id(value)?),
            "public-ipv4" => set(&mut public_ipv4, parse_ipv4(value)?),
            "availability-zone" => set(&mut availability_zone, scalar(value, 64, name_chars)?),
            "region" => set(&mut region, scalar(value, 64, name_chars)?),
            _ => return Err(MetadataDecodeError::UnknownField),
        }?;
    }
    Ok(MetadataSummary {
        hostname: hostname.ok_or(MetadataDecodeError::MissingField)?,
        instance_id: instance_id.ok_or(MetadataDecodeError::MissingField)?,
        public_ipv4: public_ipv4.ok_or(MetadataDecodeError::MissingField)?,
        availability_zone: availability_zone.ok_or(MetadataDecodeError::MissingField)?,
        region: region.ok_or(MetadataDecodeError::MissingField)?,
    })
}

pub(super) fn document(text: &str) -> Result<&str, MetadataDecodeError> {
    let text = text.strip_suffix('\n').unwrap_or(text);
    if text.is_empty() {
        return Err(MetadataDecodeError::Empty);
    }
    if text.contains('\r') || text.ends_with('\n') || text.bytes().any(|byte| byte == 0) {
        return Err(MetadataDecodeError::InvalidSyntax);
    }
    Ok(text)
}

pub(super) fn scalar(
    text: &str,
    max: usize,
    valid: fn(u8) -> bool,
) -> Result<&str, MetadataDecodeError> {
    let text = document(text)?;
    if text.len() > max {
        return Err(MetadataDecodeError::FieldTooLong);
    }
    if !text.is_ascii() || !text.bytes().all(valid) {
        return Err(MetadataDecodeError::InvalidSyntax);
    }
    Ok(text)
}

pub(super) fn parse_u64(value: &str) -> Result<u64, MetadataDecodeError> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || value.len() > 1 && value.starts_with('0')
    {
        return Err(MetadataDecodeError::InvalidNumber);
    }
    value
        .parse()
        .map_err(|_| MetadataDecodeError::InvalidNumber)
}

fn parse_instance_id(value: &str) -> Result<u64, MetadataDecodeError> {
    let value = parse_u64(value)?;
    if value == 0 {
        return Err(MetadataDecodeError::InvalidNumber);
    }
    Ok(value)
}

pub(super) fn parse_ipv4(value: &str) -> Result<Ipv4Addr, MetadataDecodeError> {
    let address = Ipv4Addr::from_str(value).map_err(|_| MetadataDecodeError::InvalidIpv4)?;
    let octets = address.octets();
    let canonical = canonical_ipv4_len(octets) == value.len();
    if !canonical {
        return Err(MetadataDecodeError::InvalidIpv4);
    }
    Ok(address)
}

fn canonical_ipv4_len(octets: [u8; 4]) -> usize {
    octets
        .into_iter()
        .map(decimal_len)
        .sum::<usize>()
        .saturating_add(3)
}

fn decimal_len(value: u8) -> usize {
    if value >= 100 {
        3
    } else if value >= 10 {
        2
    } else {
        1
    }
}

fn set<T>(slot: &mut Option<T>, value: T) -> Result<(), MetadataDecodeError> {
    if slot.replace(value).is_some() {
        Err(MetadataDecodeError::DuplicateField)
    } else {
        Ok(())
    }
}

fn valid_hostname(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_')
}

fn name_chars(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-'
}

fn ascii_digits(byte: u8) -> bool {
    byte.is_ascii_digit()
}
fn ipv4_chars(byte: u8) -> bool {
    byte.is_ascii_digit() || byte == b'.'
}
