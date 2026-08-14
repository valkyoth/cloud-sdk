use alloc::format;

use super::tests::{
    CREATED, DETAIL, SUMMARY, decode_create, decode_get, decode_list, id, name, vlan,
};
use super::*;

#[test]
fn vlan_boundaries_reject_every_value_outside_the_usable_range() {
    for value in 0..=u16::MAX {
        assert_eq!(
            RobotVlanId::new(value).is_ok(),
            (4000..=4091).contains(&value),
            "unexpected VLAN admission for {value}",
        );
    }
}

#[test]
fn names_admit_only_the_documented_ascii_profile() {
    assert!(RobotVSwitchName::new("A-z_09.private-fabric").is_ok());
    let exact = "a".repeat(MAX_ROBOT_VSWITCH_NAME_BYTES);
    assert!(RobotVSwitchName::new(&exact).is_ok());
    assert!(RobotVSwitchName::new(&format!("{exact}a")).is_err());

    for byte in 0_u8..=127 {
        let value = [b'a', byte, b'b'];
        let text = core::str::from_utf8(&value)
            .unwrap_or_else(|_| unreachable!("ASCII fixture lost UTF-8"));
        let allowed = byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'-' | b'_' | b'.');
        assert_eq!(RobotVSwitchName::new(text).is_ok(), allowed);
    }

    for invalid in [
        "",
        " leading",
        "trailing ",
        "soft\u{00ad}hyphen",
        "grapheme\u{034f}joiner",
        "variation\u{fe0f}selector",
        "tag\u{e0061}character",
        "mixed-\u{0430}",
    ] {
        assert!(RobotVSwitchName::new(invalid).is_err());
    }
}

#[test]
fn server_ip_canonicalization_accepts_exact_display_only() {
    for valid in ["192.0.2.10", "2001:db8::1"] {
        assert!(RobotVSwitchServerIdentifier::new(valid).is_ok());
    }
    for invalid in [
        "192.000.2.10",
        "2001:0db8::1",
        "2001:DB8::1",
        "0:0:0:0:0:ffff:c000:020a",
    ] {
        assert!(RobotVSwitchServerIdentifier::new(invalid).is_err());
    }
}

#[test]
fn provider_names_are_high_assurance_or_explicitly_quarantined() {
    let ordinary =
        decode_get(id(), DETAIL).unwrap_or_else(|_| unreachable!("ordinary provider name failed"));
    assert!(ordinary.name().is_high_assurance());
    assert!(!ordinary.name().is_quarantined());
    assert!(ordinary.name().as_high_assurance().is_some());
    let detail =
        core::str::from_utf8(DETAIL).unwrap_or_else(|_| unreachable!("detail fixture lost UTF-8"));

    for (wire, expected) in [
        ("prod/eu", "prod/eu"),
        ("mixed-\\u0430", "mixed-\u{0430}"),
        ("line\\nbreak", "line\nbreak"),
    ] {
        let response = detail.replace("my vSwitch", wire);
        let observed = decode_get(id(), response.as_bytes())
            .unwrap_or_else(|_| unreachable!("bounded provider name was not quarantined"));
        assert!(!observed.name().is_high_assurance());
        assert!(observed.name().is_quarantined());
        assert!(observed.name().as_high_assurance().is_none());
        assert_eq!(
            observed.name().try_with_text(|value| value == expected),
            Ok(true)
        );
        assert_eq!(
            format!("{:?}", observed.name()),
            "RobotVSwitchObservedName([redacted])"
        );
    }

    let empty = detail.replace("my vSwitch", "");
    assert_eq!(
        decode_get(id(), empty.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidVSwitch)
    );
    let oversized = "a".repeat(MAX_ROBOT_VSWITCH_NAME_BYTES + 1);
    let response = detail.replace("my vSwitch", &oversized);
    assert_eq!(
        decode_get(id(), response.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidVSwitch)
    );
}

#[test]
fn one_quarantined_name_does_not_invalidate_the_complete_inventory() {
    let second = SUMMARY
        .replace("\"id\":4321", "\"id\":4322")
        .replace("my vSwitch", "prod/eu")
        .replace("\"vlan\":4000", "\"vlan\":4001");
    let response = format!("[{SUMMARY},{second}]");
    let inventory = decode_list(response.as_bytes())
        .unwrap_or_else(|_| unreachable!("mixed-assurance inventory failed"));
    assert_eq!(inventory.len(), 2);
    assert!(inventory.as_slice()[0].name().is_high_assurance());
    assert!(inventory.as_slice()[1].name().is_quarantined());
}

#[test]
fn creation_reconciliation_rejects_a_quarantined_provider_name() {
    let request = RobotVSwitchCreateRequest::new(name("my vSwitch"), vlan(4000));
    let response = core::str::from_utf8(CREATED)
        .unwrap_or_else(|_| unreachable!("creation fixture lost UTF-8"))
        .replace("my vSwitch", "my/vSwitch");
    assert_eq!(
        decode_create(&request, response.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::MutationOutcomeMismatch)
    );
}
