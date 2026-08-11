use core::cmp::Ordering;
use core::convert::Infallible;
use core::fmt;

use cloud_sdk_sanitization::SecretBoxBytes;

/// Failure while constructing a protected Robot server number.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotServerNumberError {
    /// Robot server numbers must be positive.
    Zero,
    /// Stable protected storage could not be allocated.
    Allocation,
}

impl fmt::Display for RobotServerNumberError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Zero => "Robot server number must be positive",
            Self::Allocation => "Robot server number allocation failed",
        })
    }
}

impl core::error::Error for RobotServerNumberError {}

/// Positive canonical Robot server number in stable protected storage.
pub struct RobotServerNumber(SecretBoxBytes);

#[cfg(feature = "serde")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DecimalServerNumberError {
    Invalid,
    Allocation,
}

impl RobotServerNumber {
    /// Creates a positive server number without retaining inline classified bytes.
    pub fn new(value: u64) -> Result<Self, RobotServerNumberError> {
        if value == 0 {
            return Err(RobotServerNumberError::Zero);
        }
        let len = cloud_sdk::buffer::u64_encoded_len(value);
        let bytes = SecretBoxBytes::try_from_fn_bounded(len, 20, |index| {
            Ok::<u8, Infallible>(decimal_digit(value, len, index))
        })
        .map_err(|_| RobotServerNumberError::Allocation)?;
        Ok(Self(bytes))
    }

    #[cfg(feature = "serde")]
    pub(crate) fn from_decimal_bytes(digits: &[u8]) -> Result<Self, DecimalServerNumberError> {
        if !valid_positive_u64_decimal(digits) {
            return Err(DecimalServerNumberError::Invalid);
        }
        SecretBoxBytes::try_from_slice(digits, 20)
            .map(Self)
            .map_err(|_| DecimalServerNumberError::Allocation)
    }

    /// Runs a closure with temporary access to the provider number.
    pub fn with_number<R>(&self, inspect: impl FnOnce(u64) -> R) -> R {
        inspect(self.value())
    }

    pub(crate) fn value(&self) -> u64 {
        self.0.with_secret(|bytes| {
            bytes.iter().fold(0_u64, |value, byte| {
                value
                    .saturating_mul(10)
                    .saturating_add(u64::from(byte.saturating_sub(b'0')))
            })
        })
    }

    pub(crate) fn with_decimal_bytes<R>(&self, inspect: impl FnOnce(&[u8]) -> R) -> R {
        self.0.with_secret(inspect)
    }
}

impl PartialEq for RobotServerNumber {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for RobotServerNumber {}

impl PartialOrd for RobotServerNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RobotServerNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.with_secret(|left| {
            other
                .0
                .with_secret(|right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)))
        })
    }
}

fn decimal_digit(value: u64, len: usize, index: usize) -> u8 {
    let exponent = len.saturating_sub(index).saturating_sub(1);
    let mut divisor = 1_u64;
    for _ in 0..exponent {
        divisor = divisor.saturating_mul(10);
    }
    let digit = value
        .checked_div(divisor)
        .and_then(|value| value.checked_rem(10))
        .unwrap_or_else(|| unreachable!("nonzero decimal divisor failed"));
    b'0'.saturating_add(
        u8::try_from(digit).unwrap_or_else(|_| unreachable!("decimal digit exceeded u8")),
    )
}

#[cfg(feature = "serde")]
fn valid_positive_u64_decimal(digits: &[u8]) -> bool {
    digits != b"0"
        && !digits.is_empty()
        && digits.first() != Some(&b'0')
        && digits.iter().all(u8::is_ascii_digit)
        && (digits.len() < 20 || (digits.len() == 20 && digits <= b"18446744073709551615"))
}

impl fmt::Debug for RobotServerNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerNumber([redacted])")
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::{DecimalServerNumberError, RobotServerNumber};

    #[test]
    fn protected_decimal_admission_is_canonical_and_bounded() {
        let maximum = RobotServerNumber::from_decimal_bytes(b"18446744073709551615")
            .unwrap_or_else(|_| unreachable!("maximum u64 decimal was rejected"));
        assert_eq!(maximum.with_number(|value| value), u64::MAX);

        for invalid in [b"".as_slice(), b"00", b"01", b"18446744073709551616"] {
            assert!(matches!(
                RobotServerNumber::from_decimal_bytes(invalid),
                Err(DecimalServerNumberError::Invalid)
            ));
        }
    }
}
