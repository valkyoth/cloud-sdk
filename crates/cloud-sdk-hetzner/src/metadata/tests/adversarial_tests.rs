use alloc::format;
use alloc::string::String;
use alloc::vec;
use core::fmt::Write as _;

use super::super::{
    MAX_METADATA_ALIAS_IPS, MAX_METADATA_PRIVATE_NETWORKS, MAX_METADATA_RESPONSE_BYTES,
    MetadataDecodeError, MetadataRoute, decode_metadata_body,
};
use super::SUMMARY;

const NETWORK: &str = "- ip: 10.0.0.2\n  alias_ips: [10.0.0.3]\n  interface_num: 1\n  mac_address: 86:00:00:2a:7d:e0\n  network_id: 1234\n  network_name: private\n  network: 10.0.0.0/8\n  subnet: 10.0.0.0/24\n  gateway: 10.0.0.1\n";

#[test]
fn every_route_rejects_the_aggregate_response_overrun() {
    let oversized = vec![b'a'; MAX_METADATA_RESPONSE_BYTES + 1];
    for route in [
        MetadataRoute::Summary,
        MetadataRoute::Hostname,
        MetadataRoute::InstanceId,
        MetadataRoute::PublicIpv4,
        MetadataRoute::PrivateNetworks,
        MetadataRoute::AvailabilityZone,
        MetadataRoute::Region,
    ] {
        assert_eq!(
            decode_metadata_body(route, &oversized),
            Err(MetadataDecodeError::ResponseTooLarge)
        );
    }
}

#[test]
fn scalar_decoders_reject_controls_noncanonical_values_and_extra_lines() {
    let cases = [
        (MetadataRoute::Hostname, b"host\r\n".as_slice()),
        (MetadataRoute::Hostname, b"host\nother".as_slice()),
        (MetadataRoute::Hostname, b"host\0name".as_slice()),
        (MetadataRoute::Hostname, b"h\xc3\xb6st".as_slice()),
        (MetadataRoute::InstanceId, b"0".as_slice()),
        (MetadataRoute::InstanceId, b"+42".as_slice()),
        (
            MetadataRoute::InstanceId,
            b"18446744073709551616".as_slice(),
        ),
        (MetadataRoute::PublicIpv4, b"127.1".as_slice()),
        (MetadataRoute::PublicIpv4, b"01.2.3.4".as_slice()),
        (MetadataRoute::AvailabilityZone, b"hel1 dc2".as_slice()),
        (MetadataRoute::Region, b"eu/central".as_slice()),
    ];
    for (route, body) in cases {
        assert!(
            decode_metadata_body(route, body).is_err(),
            "unexpectedly accepted {route:?}: {body:?}"
        );
    }
}

#[test]
fn public_ipv4_accepts_only_the_shared_single_lf_normalization() {
    assert!(decode_metadata_body(MetadataRoute::PublicIpv4, b"1.2.3.4\n").is_ok());
    assert_eq!(
        decode_metadata_body(MetadataRoute::PublicIpv4, b"1.2.3.4\n\n"),
        Err(MetadataDecodeError::InvalidSyntax)
    );
    assert_eq!(
        decode_metadata_body(MetadataRoute::PublicIpv4, b"1.2.3.4\r\n"),
        Err(MetadataDecodeError::InvalidSyntax)
    );
}

#[test]
fn metadata_identity_fields_enforce_server_and_public_address_semantics() {
    let invalid_hostnames = ["-", ".", "a..b", "-host", "host-", "host_name"];
    for hostname in invalid_hostnames {
        assert_eq!(
            decode_metadata_body(MetadataRoute::Hostname, hostname.as_bytes()),
            Err(MetadataDecodeError::InvalidSyntax)
        );
        let summary = core::str::from_utf8(SUMMARY)
            .unwrap_or_else(|_| unreachable!("static UTF-8 summary"))
            .replace("my-server", hostname);
        assert_eq!(
            decode_metadata_body(MetadataRoute::Summary, summary.as_bytes()),
            Err(MetadataDecodeError::InvalidSyntax)
        );
    }
    for hostname in ["a".repeat(64), "-a".repeat(32)] {
        assert!(decode_metadata_body(MetadataRoute::Hostname, hostname.as_bytes()).is_err());
    }
    assert!(decode_metadata_body(MetadataRoute::Hostname, "a".repeat(63).as_bytes()).is_ok());

    let non_public = [
        "0.0.0.0",
        "10.0.0.1",
        "100.64.0.1",
        "127.0.0.1",
        "169.254.1.1",
        "192.0.0.1",
        "192.0.2.1",
        "192.88.99.1",
        "198.18.0.1",
        "198.51.100.1",
        "203.0.113.1",
        "224.0.0.1",
        "240.0.0.1",
        "255.255.255.255",
    ];
    for address in non_public {
        assert_eq!(
            decode_metadata_body(MetadataRoute::PublicIpv4, address.as_bytes()),
            Err(MetadataDecodeError::InvalidIpv4)
        );
        let summary = core::str::from_utf8(SUMMARY)
            .unwrap_or_else(|_| unreachable!("static UTF-8 summary"))
            .replace("1.2.3.4", address);
        assert_eq!(
            decode_metadata_body(MetadataRoute::Summary, summary.as_bytes()),
            Err(MetadataDecodeError::InvalidIpv4)
        );
    }
}

#[test]
fn private_network_yaml_rejects_unknown_syntax_and_field_contradictions() {
    let cases = [
        NETWORK.replace(
            "  gateway: 10.0.0.1\n",
            "  secret: token\n  gateway: 10.0.0.1\n",
        ),
        NETWORK.replace("86:00:00:2a:7d:e0", "86:00:00:2A:7D:E0"),
        NETWORK.replace("10.0.0.0/24", "10.0.0.1/24"),
        NETWORK.replace("10.0.0.1\n", "10.0.0.255\n"),
        NETWORK.replace("10.0.0.2", "10.0.0.1"),
        NETWORK.replace("[10.0.0.3]", "[10.0.0.3, 10.0.0.3]"),
        NETWORK.replace("alias_ips: [10.0.0.3]", "alias_ips:\n    - 10.0.0.3"),
        NETWORK.replace("network_name: private", "network_name: private # comment"),
    ];
    for body in cases {
        assert!(decode_metadata_body(MetadataRoute::PrivateNetworks, body.as_bytes()).is_err());
    }
}

#[test]
fn private_network_and_alias_collection_limits_fail_closed() {
    let mut networks = String::new();
    let count = MAX_METADATA_PRIVATE_NETWORKS
        .checked_add(1)
        .unwrap_or_else(|| unreachable!("fixed limit"));
    for index in 0..count {
        let identity = index
            .checked_add(1)
            .unwrap_or_else(|| unreachable!("bounded test identity"));
        assert!(
            write!(
                networks,
                "- ip: 10.0.0.2\n  alias_ips: []\n  interface_num: {identity}\n  mac_address: 86:00:00:2a:7d:e0\n  network_id: {identity}\n  network_name: private-{identity}\n  network: 10.0.0.0/8\n  subnet: 10.0.0.0/24\n  gateway: 10.0.0.1\n"
            )
            .is_ok()
        );
    }
    assert_eq!(
        decode_metadata_body(MetadataRoute::PrivateNetworks, networks.as_bytes()),
        Err(MetadataDecodeError::TooManyItems)
    );

    let aliases = alias_values(MAX_METADATA_ALIAS_IPS);
    let body = alias_network(&aliases);
    assert!(decode_metadata_body(MetadataRoute::PrivateNetworks, body.as_bytes()).is_ok());

    let oversized = format!("{aliases}, not-an-address");
    let body = alias_network(&oversized);
    assert_eq!(
        decode_metadata_body(MetadataRoute::PrivateNetworks, body.as_bytes()),
        Err(MetadataDecodeError::TooManyItems)
    );
}

fn alias_values(count: usize) -> String {
    let mut aliases = String::new();
    for index in 0..count {
        if index != 0 {
            aliases.push_str(", ");
        }
        let host = index
            .checked_add(2)
            .unwrap_or_else(|| unreachable!("bounded alias host"));
        assert!(write!(aliases, "10.1.{}.{}", host / 256, host % 256).is_ok());
    }
    aliases
}

fn alias_network(aliases: &str) -> String {
    format!(
        "- ip: 10.1.255.254\n  alias_ips: [{aliases}]\n  interface_num: 1\n  mac_address: 86:00:00:2a:7d:e0\n  network_id: 1\n  network_name: private\n  network: 10.0.0.0/8\n  subnet: 10.1.0.0/16\n  gateway: 10.1.255.253\n"
    )
}
