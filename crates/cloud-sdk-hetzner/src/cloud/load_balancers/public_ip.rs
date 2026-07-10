use core::net::{Ipv4Addr, Ipv6Addr};

#[derive(Clone, Copy)]
struct Ipv6Prefix {
    network: u32,
    prefix_len: u8,
}

impl Ipv6Prefix {
    const fn new(first: u16, second: u16, prefix_len: u8) -> Self {
        Self {
            network: ((first as u32) << 16) | second as u32,
            prefix_len,
        }
    }

    fn contains(self, address: Ipv6Addr) -> bool {
        let [first, second, _, _, _, _, _, _] = address.segments();
        let value = (u32::from(first) << 16) | u32::from(second);
        let shift = 32_u32.checked_sub(u32::from(self.prefix_len)).unwrap_or(32);
        let mask = u32::MAX.checked_shl(shift).unwrap_or(0);
        value & mask == self.network & mask
    }
}

// IANA IPv6 Global Unicast Address Space, last updated 2025-10-10. The
// partially allocated 2001::/23 and special-purpose 2002::/16 are deliberately
// excluded. See docs/IANA_IPV6_SOURCE_LOCK.md.
const IANA_ORDINARY_GLOBAL_UNICAST: &[Ipv6Prefix] = &[
    Ipv6Prefix::new(0x2001, 0x0200, 23),
    Ipv6Prefix::new(0x2001, 0x0400, 23),
    Ipv6Prefix::new(0x2001, 0x0600, 23),
    Ipv6Prefix::new(0x2001, 0x0800, 22),
    Ipv6Prefix::new(0x2001, 0x0c00, 23),
    Ipv6Prefix::new(0x2001, 0x0e00, 23),
    Ipv6Prefix::new(0x2001, 0x1200, 23),
    Ipv6Prefix::new(0x2001, 0x1400, 22),
    Ipv6Prefix::new(0x2001, 0x1800, 23),
    Ipv6Prefix::new(0x2001, 0x1a00, 23),
    Ipv6Prefix::new(0x2001, 0x1c00, 22),
    Ipv6Prefix::new(0x2001, 0x2000, 19),
    Ipv6Prefix::new(0x2001, 0x4000, 23),
    Ipv6Prefix::new(0x2001, 0x4200, 23),
    Ipv6Prefix::new(0x2001, 0x4400, 23),
    Ipv6Prefix::new(0x2001, 0x4600, 23),
    Ipv6Prefix::new(0x2001, 0x4800, 23),
    Ipv6Prefix::new(0x2001, 0x4a00, 23),
    Ipv6Prefix::new(0x2001, 0x4c00, 23),
    Ipv6Prefix::new(0x2001, 0x5000, 20),
    Ipv6Prefix::new(0x2001, 0x8000, 19),
    Ipv6Prefix::new(0x2001, 0xa000, 20),
    Ipv6Prefix::new(0x2001, 0xb000, 20),
    Ipv6Prefix::new(0x2003, 0x0000, 18),
    Ipv6Prefix::new(0x2400, 0x0000, 12),
    Ipv6Prefix::new(0x2410, 0x0000, 12),
    Ipv6Prefix::new(0x2600, 0x0000, 12),
    Ipv6Prefix::new(0x2610, 0x0000, 23),
    Ipv6Prefix::new(0x2620, 0x0000, 23),
    Ipv6Prefix::new(0x2630, 0x0000, 12),
    Ipv6Prefix::new(0x2800, 0x0000, 12),
    Ipv6Prefix::new(0x2a00, 0x0000, 12),
    Ipv6Prefix::new(0x2a10, 0x0000, 12),
    Ipv6Prefix::new(0x2c00, 0x0000, 12),
];

pub(super) fn invalid_public_v4(address: Ipv4Addr) -> bool {
    let [first, second, third, _] = address.octets();
    address.is_private()
        || address.is_loopback()
        || address.is_link_local()
        || address.is_multicast()
        || address.is_unspecified()
        || address.is_broadcast()
        || first == 0
        || (first == 100 && (64..=127).contains(&second))
        || (first == 192 && second == 0 && (third == 0 || third == 2))
        || (first == 192 && second == 88 && third == 99)
        || (first == 198 && (second == 18 || second == 19))
        || (first == 198 && second == 51 && third == 100)
        || (first == 203 && second == 0 && third == 113)
        || first >= 240
}

pub(super) fn invalid_public_v6(address: Ipv6Addr) -> bool {
    let [first, second, third, _, _, _, _, _] = address.segments();
    let documentation = first == 0x2001 && second == 0x0db8;
    let as112_service = first == 0x2620 && second == 0x004f && third == 0x8000;

    address.to_ipv4_mapped().is_some()
        || documentation
        || as112_service
        || !IANA_ORDINARY_GLOBAL_UNICAST
            .iter()
            .any(|prefix| prefix.contains(address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_source_locked_allocation_has_an_admitted_address() {
        for prefix in IANA_ORDINARY_GLOBAL_UNICAST {
            let [a, b, c, d] = prefix.network.to_be_bytes();
            let first = u16::from_be_bytes([a, b]);
            let second = u16::from_be_bytes([c, d]);
            let address = Ipv6Addr::new(first, second, 0, 0, 0, 0, 0, 1);
            assert!(!invalid_public_v6(address), "rejected {address}");
        }
    }
}
