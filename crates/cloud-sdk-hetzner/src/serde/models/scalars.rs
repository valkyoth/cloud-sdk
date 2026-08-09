//! Exact scalar response models used across Cloud response families.

use alloc::string::String;
use core::fmt;

use cloud_sdk_sanitization::sanitize_string;

use super::cloud_constraints::valid_rfc3339;
use super::{ResponseModelError, checked_text};
use crate::serde::strict_json::Value;

const MAX_TIMESTAMP_BYTES: usize = 64;
const MAX_DECIMAL_BYTES: usize = 128;

/// Calendar-valid RFC 3339 timestamp in Hetzner's canonical UTC form.
#[derive(Eq, PartialEq)]
pub struct UtcTimestamp(String);

impl UtcTimestamp {
    pub(super) fn try_new(value: &str) -> Result<Self, ResponseModelError> {
        if !valid_utc_timestamp(value) {
            return Err(ResponseModelError::InvalidText);
        }
        checked_text(value, MAX_TIMESTAMP_BYTES).map(Self)
    }

    pub(super) fn try_from_string(value: String) -> Result<Self, ResponseModelError> {
        if !valid_utc_timestamp(&value) || value.is_empty() || value.len() > MAX_TIMESTAMP_BYTES {
            return Err(ResponseModelError::InvalidText);
        }
        Ok(Self(value))
    }

    /// Returns the exact source timestamp.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Fallibly copies this bounded timestamp.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        Self::try_new(&self.0)
    }
}

impl fmt::Debug for UtcTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UtcTimestamp([redacted])")
    }
}

impl Drop for UtcTimestamp {
    fn drop(&mut self) {
        sanitize_string(&mut self.0);
    }
}

/// Exact lexical representation of a finite JSON number.
#[derive(Eq, PartialEq)]
pub struct ExactDecimal(String);

impl ExactDecimal {
    pub(super) fn take(value: &mut Value) -> Result<Self, ResponseModelError> {
        let numeric = value.as_f64().ok_or(ResponseModelError::WrongType)?;
        if !numeric.is_finite() {
            return Err(ResponseModelError::InvalidNumber);
        }
        let lexical = value
            .take_number_lexical()
            .ok_or(ResponseModelError::WrongType)?;
        Self::from_lexical(lexical)
    }

    fn from_lexical(lexical: String) -> Result<Self, ResponseModelError> {
        if lexical.is_empty() || lexical.len() > MAX_DECIMAL_BYTES {
            return Err(ResponseModelError::InvalidNumber);
        }
        Ok(Self(lexical))
    }

    /// Returns the exact JSON number text admitted from the provider.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Fallibly copies this bounded exact number.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        let mut lexical = String::new();
        lexical
            .try_reserve_exact(self.0.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        lexical.push_str(&self.0);
        Self::from_lexical(lexical)
    }

    pub(crate) fn is_non_negative(&self) -> bool {
        !self.0.as_bytes().starts_with(b"-")
    }

    pub(crate) fn is_strictly_positive(&self) -> bool {
        self.is_non_negative()
            && self
                .0
                .bytes()
                .take_while(|byte| !matches!(byte, b'e' | b'E'))
                .any(|byte| matches!(byte, b'1'..=b'9'))
    }
}

impl fmt::Debug for ExactDecimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ExactDecimal([redacted])")
    }
}

pub(crate) fn valid_utc_timestamp(value: &str) -> bool {
    value.as_bytes().get(10) == Some(&b'T')
        && value.as_bytes().last() == Some(&b'Z')
        && valid_rfc3339(value)
}

#[cfg(test)]
mod tests {
    use super::{ExactDecimal, UtcTimestamp};
    use crate::serde::strict_json::parse;

    #[test]
    fn utc_timestamp_checks_calendar_and_canonical_source_form() {
        for value in ["2024-02-29T23:59:60Z", "2026-08-08T12:34:56.123Z"] {
            assert!(UtcTimestamp::try_new(value).is_ok());
        }
        for value in [
            "2025-02-29T00:00:00Z",
            "2026-08-08t12:34:56Z",
            "2026-08-08T12:34:56z",
            "2026-08-08T12:34:56+00:00",
        ] {
            assert!(UtcTimestamp::try_new(value).is_err());
        }
    }

    #[test]
    fn exact_decimal_preserves_integer_fraction_exponent_and_negative_zero() {
        for lexical in ["60", "60.000000000000001", "6e1", "-0"] {
            let Ok(mut value) = parse(lexical.as_bytes()) else {
                unreachable!("exact number fixture failed to parse")
            };
            let Ok(decimal) = ExactDecimal::take(&mut value) else {
                unreachable!("exact number fixture failed to convert")
            };
            assert_eq!(decimal.as_str(), lexical);
        }
    }

    #[test]
    fn exact_decimal_classifies_sign_and_zero_without_floating_point() {
        for (lexical, non_negative, positive) in [
            ("-1e-400", false, false),
            ("1e-400", true, true),
            ("-0", false, false),
            ("0e100", true, false),
            ("6e-1", true, true),
            ("6e+1", true, true),
        ] {
            let Ok(mut value) = parse(lexical.as_bytes()) else {
                unreachable!("exact classification fixture failed to parse")
            };
            let Ok(decimal) = ExactDecimal::take(&mut value) else {
                unreachable!("exact classification fixture failed to convert")
            };
            assert_eq!(decimal.is_non_negative(), non_negative, "{lexical}");
            assert_eq!(decimal.is_strictly_positive(), positive, "{lexical}");
        }
    }
}
