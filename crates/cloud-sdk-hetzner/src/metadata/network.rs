use core::net::Ipv4Addr;

use super::text::{MetadataDecodeError, document, parse_ipv4, parse_u64, scalar};
use super::{MAX_METADATA_ALIAS_IPS, MAX_METADATA_PRIVATE_NETWORKS};

/// Validated private-network collection backed by the response bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataPrivateNetworks<'a> {
    text: &'a str,
    count: usize,
}

impl<'a> MetadataPrivateNetworks<'a> {
    pub(crate) fn new(text: &'a str) -> Result<Self, MetadataDecodeError> {
        let text = document(text)?;
        if text == "[]" {
            return Ok(Self { text, count: 0 });
        }
        let mut count = 0usize;
        let mut aliases = 0usize;
        let mut offset = 0usize;
        while offset < text.len() {
            let (record, next) = parse_record(text, offset)?;
            count = count
                .checked_add(1)
                .ok_or(MetadataDecodeError::TooManyItems)?;
            aliases = aliases
                .checked_add(record.alias_ips.len())
                .ok_or(MetadataDecodeError::TooManyItems)?;
            if count > MAX_METADATA_PRIVATE_NETWORKS || aliases > MAX_METADATA_ALIAS_IPS {
                return Err(MetadataDecodeError::TooManyItems);
            }
            validate_against_prior(text, offset, record)?;
            offset = next;
        }
        Ok(Self { text, count })
    }

    /// Returns the number of attached network records.
    #[must_use]
    pub const fn len(self) -> usize {
        self.count
    }

    /// Reports whether no private network is attached.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.count == 0
    }

    /// Iterates validated records without allocation.
    #[must_use]
    pub const fn iter(self) -> MetadataPrivateNetworksIter<'a> {
        MetadataPrivateNetworksIter {
            text: self.text,
            offset: 0,
        }
    }
}

/// Iterator over strict private-network records.
#[derive(Clone, Debug)]
pub struct MetadataPrivateNetworksIter<'a> {
    text: &'a str,
    offset: usize,
}

impl<'a> Iterator for MetadataPrivateNetworksIter<'a> {
    type Item = MetadataPrivateNetwork<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.text.len() || self.text == "[]" {
            return None;
        }
        let (record, next) = parse_record(self.text, self.offset).ok()?;
        self.offset = next;
        Some(record)
    }
}

/// One attached private-network record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataPrivateNetwork<'a> {
    ip: Ipv4Addr,
    alias_ips: AliasIpv4Addresses<'a>,
    interface_num: u32,
    mac_address: [u8; 6],
    network_id: u64,
    network_name: &'a str,
    network: Ipv4Cidr,
    subnet: Ipv4Cidr,
    gateway: Ipv4Addr,
}

impl<'a> MetadataPrivateNetwork<'a> {
    /// Returns the interface IPv4 address.
    #[must_use]
    pub const fn ip(self) -> Ipv4Addr {
        self.ip
    }
    /// Returns validated alias IPv4 addresses.
    #[must_use]
    pub const fn alias_ips(self) -> AliasIpv4Addresses<'a> {
        self.alias_ips
    }
    /// Returns the metadata interface number.
    #[must_use]
    pub const fn interface_num(self) -> u32 {
        self.interface_num
    }
    /// Returns the six MAC-address octets.
    #[must_use]
    pub const fn mac_address(self) -> [u8; 6] {
        self.mac_address
    }
    /// Returns the Cloud Network identifier.
    #[must_use]
    pub const fn network_id(self) -> u64 {
        self.network_id
    }
    /// Returns the Cloud Network name.
    #[must_use]
    pub const fn network_name(self) -> &'a str {
        self.network_name
    }
    /// Returns the parent network address and prefix length.
    #[must_use]
    pub const fn network(self) -> (Ipv4Addr, u8) {
        (self.network.address, self.network.prefix)
    }
    /// Returns the attached subnet address and prefix length.
    #[must_use]
    pub const fn subnet(self) -> (Ipv4Addr, u8) {
        (self.subnet.address, self.subnet.prefix)
    }
    /// Returns the subnet gateway.
    #[must_use]
    pub const fn gateway(self) -> Ipv4Addr {
        self.gateway
    }
}

/// Validated flow-style YAML list of alias IPv4 addresses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AliasIpv4Addresses<'a> {
    values: &'a str,
    count: usize,
}

impl<'a> AliasIpv4Addresses<'a> {
    /// Returns the alias count.
    #[must_use]
    pub const fn len(self) -> usize {
        self.count
    }
    /// Reports whether no aliases are present.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.count == 0
    }
    /// Iterates canonical alias addresses.
    pub fn iter(self) -> impl Iterator<Item = Ipv4Addr> + 'a {
        self.values
            .split(", ")
            .filter(|value| !value.is_empty())
            .filter_map(|value| parse_ipv4(value).ok())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Ipv4Cidr {
    address: Ipv4Addr,
    prefix: u8,
}

#[derive(Default)]
struct RecordBuilder<'a> {
    ip: Option<Ipv4Addr>,
    aliases: Option<AliasIpv4Addresses<'a>>,
    interface: Option<u32>,
    mac: Option<[u8; 6]>,
    network_id: Option<u64>,
    network_name: Option<&'a str>,
    network: Option<Ipv4Cidr>,
    subnet: Option<Ipv4Cidr>,
    gateway: Option<Ipv4Addr>,
}

fn parse_record(
    text: &str,
    start: usize,
) -> Result<(MetadataPrivateNetwork<'_>, usize), MetadataDecodeError> {
    let tail = text
        .get(start..)
        .ok_or(MetadataDecodeError::InvalidSyntax)?;
    if !tail.starts_with("- ") {
        return Err(MetadataDecodeError::InvalidSyntax);
    }
    let boundary = tail.get(2..).and_then(|value| value.find("\n- "));
    let (block_end, next) = if let Some(index) = boundary {
        let block_end = start
            .checked_add(2)
            .and_then(|value| value.checked_add(index))
            .ok_or(MetadataDecodeError::InvalidSyntax)?;
        let next = block_end
            .checked_add(1)
            .ok_or(MetadataDecodeError::InvalidSyntax)?;
        (block_end, next)
    } else {
        (text.len(), text.len())
    };
    let block = text
        .get(start..block_end)
        .ok_or(MetadataDecodeError::InvalidSyntax)?;
    let mut builder = RecordBuilder::default();
    for (index, line) in block.split('\n').enumerate() {
        let line = if index == 0 {
            line.strip_prefix("- ")
        } else {
            line.strip_prefix("  ")
        }
        .ok_or(MetadataDecodeError::InvalidSyntax)?;
        let (key, value) = line
            .split_once(": ")
            .ok_or(MetadataDecodeError::InvalidSyntax)?;
        if value.is_empty() || value.contains(['#', '{', '}']) {
            return Err(MetadataDecodeError::InvalidSyntax);
        }
        parse_field(&mut builder, key, value)?;
    }
    let record = finish(builder)?;
    validate_record(record)?;
    Ok((record, next))
}

fn parse_field<'a>(
    builder: &mut RecordBuilder<'a>,
    key: &str,
    value: &'a str,
) -> Result<(), MetadataDecodeError> {
    match key {
        "ip" => set(&mut builder.ip, parse_ipv4(value)?),
        "alias_ips" => set(&mut builder.aliases, parse_aliases(value)?),
        "interface_num" => set(
            &mut builder.interface,
            u32::try_from(parse_u64(value)?).map_err(|_| MetadataDecodeError::InvalidNumber)?,
        ),
        "mac_address" => set(&mut builder.mac, parse_mac(value)?),
        "network_id" => set(&mut builder.network_id, parse_u64(value)?),
        "network_name" => set(
            &mut builder.network_name,
            scalar(value, 128, network_name_byte)?,
        ),
        "network" => set(&mut builder.network, parse_cidr(value)?),
        "subnet" => set(&mut builder.subnet, parse_cidr(value)?),
        "gateway" => set(&mut builder.gateway, parse_ipv4(value)?),
        _ => Err(MetadataDecodeError::UnknownField),
    }
}

fn finish(builder: RecordBuilder<'_>) -> Result<MetadataPrivateNetwork<'_>, MetadataDecodeError> {
    Ok(MetadataPrivateNetwork {
        ip: builder.ip.ok_or(MetadataDecodeError::MissingField)?,
        alias_ips: builder.aliases.ok_or(MetadataDecodeError::MissingField)?,
        interface_num: builder.interface.ok_or(MetadataDecodeError::MissingField)?,
        mac_address: builder.mac.ok_or(MetadataDecodeError::MissingField)?,
        network_id: builder
            .network_id
            .ok_or(MetadataDecodeError::MissingField)?,
        network_name: builder
            .network_name
            .ok_or(MetadataDecodeError::MissingField)?,
        network: builder.network.ok_or(MetadataDecodeError::MissingField)?,
        subnet: builder.subnet.ok_or(MetadataDecodeError::MissingField)?,
        gateway: builder.gateway.ok_or(MetadataDecodeError::MissingField)?,
    })
}

fn validate_record(record: MetadataPrivateNetwork<'_>) -> Result<(), MetadataDecodeError> {
    if record.interface_num == 0
        || record.network_id == 0
        || !private(record.ip)
        || !private(record.gateway)
    {
        return Err(MetadataDecodeError::InconsistentNetwork);
    }
    let subnet_last = last_address(record.subnet);
    if record.network.prefix > 24
        || record.subnet.prefix > 30
        || !contains(record.network, record.subnet.address)
        || !contains(record.network, subnet_last)
        || !contains(record.subnet, record.ip)
        || !contains(record.subnet, record.gateway)
        || record.ip == record.gateway
        || record.ip == record.subnet.address
        || record.ip == subnet_last
        || record.gateway == record.subnet.address
        || record.gateway == subnet_last
        || record.alias_ips.iter().any(|ip| {
            !private(ip)
                || !contains(record.subnet, ip)
                || ip == record.ip
                || ip == record.gateway
                || ip == record.subnet.address
                || ip == subnet_last
        })
    {
        return Err(MetadataDecodeError::InconsistentNetwork);
    }
    Ok(())
}

fn validate_against_prior(
    text: &str,
    current_start: usize,
    current: MetadataPrivateNetwork<'_>,
) -> Result<(), MetadataDecodeError> {
    let mut offset = 0;
    while offset < current_start {
        let (prior, next) = parse_record(text, offset)?;
        if prior.interface_num == current.interface_num || prior.network_id == current.network_id {
            return Err(MetadataDecodeError::InconsistentNetwork);
        }
        offset = next;
    }
    Ok(())
}

fn parse_aliases(value: &str) -> Result<AliasIpv4Addresses<'_>, MetadataDecodeError> {
    let values = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or(MetadataDecodeError::InvalidSyntax)?;
    let mut count = 0_usize;
    if !values.is_empty() {
        for (index, value) in values.split(", ").enumerate() {
            if value.is_empty() || parse_ipv4(value).is_err() {
                return Err(MetadataDecodeError::InvalidIpv4);
            }
            if values.split(", ").take(index).any(|prior| prior == value) {
                return Err(MetadataDecodeError::InconsistentNetwork);
            }
            count = count
                .checked_add(1)
                .ok_or(MetadataDecodeError::TooManyItems)?;
        }
        if values.contains(',') && !values.contains(", ") {
            return Err(MetadataDecodeError::InvalidSyntax);
        }
    }
    Ok(AliasIpv4Addresses { values, count })
}

fn parse_cidr(value: &str) -> Result<Ipv4Cidr, MetadataDecodeError> {
    let (address, prefix) = value
        .split_once('/')
        .ok_or(MetadataDecodeError::InvalidCidr)?;
    let address = parse_ipv4(address).map_err(|_| MetadataDecodeError::InvalidCidr)?;
    let prefix = u8::try_from(parse_u64(prefix).map_err(|_| MetadataDecodeError::InvalidCidr)?)
        .map_err(|_| MetadataDecodeError::InvalidCidr)?;
    if prefix > 32 || !private(address) || network_address(address, prefix) != address {
        return Err(MetadataDecodeError::InvalidCidr);
    }
    Ok(Ipv4Cidr { address, prefix })
}

fn parse_mac(value: &str) -> Result<[u8; 6], MetadataDecodeError> {
    if value.len() != 17 {
        return Err(MetadataDecodeError::InvalidMac);
    }
    let mut output = [0; 6];
    let mut count = 0_usize;
    for (index, pair) in value.split(':').enumerate() {
        if index >= 6
            || pair.len() != 2
            || !pair
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(MetadataDecodeError::InvalidMac);
        }
        let slot = output
            .get_mut(index)
            .ok_or(MetadataDecodeError::InvalidMac)?;
        *slot = u8::from_str_radix(pair, 16).map_err(|_| MetadataDecodeError::InvalidMac)?;
        count = count
            .checked_add(1)
            .ok_or(MetadataDecodeError::InvalidMac)?;
    }
    if count == output.len() {
        Ok(output)
    } else {
        Err(MetadataDecodeError::InvalidMac)
    }
}

fn set<T>(slot: &mut Option<T>, value: T) -> Result<(), MetadataDecodeError> {
    if slot.replace(value).is_some() {
        Err(MetadataDecodeError::DuplicateField)
    } else {
        Ok(())
    }
}

fn private(address: Ipv4Addr) -> bool {
    address.is_private()
}
fn mask(prefix: u8) -> u32 {
    let shift = 32_u32.checked_sub(u32::from(prefix)).unwrap_or(32);
    u32::MAX.checked_shl(shift).unwrap_or(0)
}
fn network_address(address: Ipv4Addr, prefix: u8) -> Ipv4Addr {
    Ipv4Addr::from(u32::from(address) & mask(prefix))
}
fn last_address(cidr: Ipv4Cidr) -> Ipv4Addr {
    Ipv4Addr::from(u32::from(cidr.address) | !mask(cidr.prefix))
}
fn contains(cidr: Ipv4Cidr, address: Ipv4Addr) -> bool {
    network_address(address, cidr.prefix) == cidr.address
}
fn network_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_')
}
