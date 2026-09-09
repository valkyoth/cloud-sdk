use super::*;
use alloc::{format, string::ToString};

#[test]
fn public_identifier_profiles_reject_traversal_and_encoded_separators() {
    for value in [
        "", ".", "..", "a/b", "a\\b", "a%2Fb", "a%252Fb", "a?b", "a#b", "a\n", "\u{e9}",
    ] {
        assert!(CrateName::new(value).is_err());
        assert!(Keyword::new(value).is_err());
        assert!(CategorySlug::new(value).is_err());
        assert!(UserLogin::new(value).is_err());
        assert!(TeamLogin::new(value).is_err());
    }
    for value in ["serde", "Hello_World", "a-0", "a"] {
        assert!(CrateName::new(value).is_ok());
    }
    for value in ["_a", "0abc", "-abc"] {
        assert!(CrateName::new(value).is_err());
    }
    assert!(Keyword::new("c++").is_ok());
    assert!(CategorySlug::new("development-tools::cargo-plugins").is_ok());
    for value in ["a:::b", "::a", "a::", "a:-b", "UPPER", "-a", "a-"] {
        assert!(CategorySlug::new(value).is_err());
    }
    assert!(Owner::new("github:rust-lang:crates-io").is_ok());
    for value in [
        "gitlab:a:b",
        "github:a",
        "github::b",
        "github:a:",
        "github:a:b:c",
    ] {
        assert!(Owner::new(value).is_err());
    }
}

#[test]
fn identifier_bounds_are_exact_and_diagnostics_redacted() {
    for (length, accepted) in [(63, true), (64, true), (65, false)] {
        assert_eq!(CrateName::new(&"a".repeat(length)).is_ok(), accepted);
    }
    for (length, accepted) in [(20, true), (21, false)] {
        assert_eq!(Keyword::new(&"a".repeat(length)).is_ok(), accepted);
    }
    for (length, accepted) in [(100, true), (101, false)] {
        assert_eq!(UserLogin::new(&"a".repeat(length)).is_ok(), accepted);
    }
    for (length, accepted) in [(256, true), (257, false)] {
        assert_eq!(CategorySlug::new(&"a".repeat(length)).is_ok(), accepted);
        let team = format!("github:a:{}", "b".repeat(length.saturating_sub(9)));
        assert_eq!(TeamLogin::new(&team).is_ok(), accepted);
    }
    let name = CrateName::new("private-project").unwrap_or_else(|_| unreachable!("name fixture"));
    assert!(!format!("{name:?}").contains("private-project"));
    assert!(NumericId::new(2_147_483_647).is_ok());
    for n in [0, 2_147_483_648, u64::MAX] {
        assert!(NumericId::new(n).is_err());
    }
    for s in [
        "",
        "01",
        "+1",
        "-1",
        "0",
        "4294967296",
        "18446744073709551616",
    ] {
        assert!(NumericId::parse(s).is_err());
    }
}

#[test]
fn semver_acceptance_matches_cargo_oracle_and_preserves_metadata() {
    let fixtures = [
        "0.0.0",
        "1.2.3",
        "1.2.3-alpha.1+build.001",
        "1.2.3+001",
        "1.2.3--",
        "1.2.3-999999999999999999999999999999",
        "1.2.3-0a",
        "18446744073709551615.0.0",
        "1",
        "1.2",
        "v1.2.3",
        "1.2.3.4",
        "1.2.3-01",
        "1.02.3",
        "01.2.3",
        "1.2.03",
        "1.2.3-",
        "1.2.3+",
        "1.2.3+a+b",
        "1.2.3+a..b",
        "1.2.3-a..b",
        "latest",
        "*",
        "^1.2.3",
        " 1.2.3",
        "1.2.3\n",
        "1.2.3-\u{e9}",
        "18446744073709551616.0.0",
    ];
    for value in fixtures {
        compare_version(value);
    }
    for (length, accepted) in [(150_usize, true), (151, false)] {
        let v = format!("1.2.3+{}", "x".repeat(length.saturating_sub(6)));
        assert_eq!(Version::new(&v).is_ok(), accepted);
    }
}

fn compare_version(value: &str) {
    let reference = semver::Version::parse(value);
    let actual = Version::new(value);
    assert_eq!(
        actual.is_ok(),
        reference.is_ok() && value.len() <= MAX_VERSION_BYTES,
        "oracle mismatch: {value:?}"
    );
    match actual {
        Ok(parsed) => assert_eq!(parsed.as_str(), value),
        Err(_) => assert!(reference.is_err() || value.len() > MAX_VERSION_BYTES),
    }
    match reference {
        Ok(parsed) => assert_eq!(parsed.to_string(), value),
        Err(_) => assert!(actual.is_err()),
    }
}

#[test]
fn semver_byte_mutations_and_generated_components_match_independent_parser() {
    for base in ["1.2.3", "1.2.3-a.0+build.01", "0.0.0-999999999999999999999"] {
        for pos in 0..base.len() {
            for byte in 0_u8..=127 {
                let mut value = base.as_bytes().to_vec();
                *value
                    .get_mut(pos)
                    .unwrap_or_else(|| unreachable!("mutation index")) = byte;
                let text =
                    core::str::from_utf8(&value).unwrap_or_else(|_| unreachable!("ASCII mutation"));
                compare_version(text);
            }
        }
    }
    for core in ["0", "1", "01", "-1", "18446744073709551616"] {
        for pre in ["0", "01", "a", "a.0", "a..b", "-", "", "a_1"] {
            for build in ["0", "01", "a+b", "a.1", ""] {
                compare_version(&format!("{core}.2.3-{pre}+{build}"));
            }
        }
    }
}

#[test]
fn calendar_dates_reject_impossible_days() {
    for v in ["2024-02-29", "2000-02-29", "0001-01-01", "9999-12-31"] {
        assert!(Date::new(v).is_ok());
    }
    for v in [
        "1900-02-29",
        "2025-02-29",
        "2024-04-31",
        "2024-00-01",
        "2024-01-00",
        "0000-01-01",
        "2024-1-01",
        "2024-01-1",
        "2024-01-01x",
        "2024-01-01-",
        "\u{e9}",
    ] {
        assert!(Date::new(v).is_err());
    }
}
