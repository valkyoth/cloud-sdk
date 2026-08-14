use alloc::format;

use super::*;

#[test]
fn vlan_boundaries_reject_every_value_outside_the_usable_range() {
    assert!(RobotVlanId::new(0).is_err());
    assert!(RobotVlanId::new(1).is_ok());
    assert!(RobotVlanId::new(4094).is_ok());
    assert!(RobotVlanId::new(4095).is_err());
    assert!(RobotVlanId::new(4096).is_err());
    assert!(RobotVlanId::new(u16::MAX).is_err());
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
