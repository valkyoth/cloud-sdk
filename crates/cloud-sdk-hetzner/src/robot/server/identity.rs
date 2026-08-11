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

impl RobotServerNumber {
    /// Creates a positive server number without retaining inline classified bytes.
    pub fn new(value: u64) -> Result<Self, RobotServerNumberError> {
        if value == 0 {
            return Err(RobotServerNumberError::Zero);
        }
        let bytes = SecretBoxBytes::try_from_fn_bounded(8, 8, |index| {
            let shift = 56_usize.saturating_sub(index.saturating_mul(8));
            Ok::<u8, Infallible>(
                u8::try_from((value >> shift) & 0xff)
                    .unwrap_or_else(|_| unreachable!("masked server-number byte exceeded u8")),
            )
        })
        .map_err(|_| RobotServerNumberError::Allocation)?;
        Ok(Self(bytes))
    }

    /// Runs a closure with temporary access to the provider number.
    pub fn with_number<R>(&self, inspect: impl FnOnce(u64) -> R) -> R {
        inspect(self.value())
    }

    pub(crate) fn value(&self) -> u64 {
        self.0.with_secret(|bytes| {
            bytes
                .iter()
                .fold(0_u64, |value, byte| (value << 8) | u64::from(*byte))
        })
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
        self.0
            .with_secret(|left| other.0.with_secret(|right| left.cmp(right)))
    }
}

impl fmt::Debug for RobotServerNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerNumber([redacted])")
    }
}
