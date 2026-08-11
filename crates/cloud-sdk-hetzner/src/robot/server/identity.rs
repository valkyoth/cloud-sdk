use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

/// Positive canonical Robot server number.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct RobotServerNumber([u8; 8]);

impl RobotServerNumber {
    /// Creates a positive server number in cleanup-owning storage.
    #[must_use]
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value.to_be_bytes()))
        }
    }

    /// Runs a closure with temporary access to the provider number.
    pub fn with_number<R>(&self, inspect: impl FnOnce(u64) -> R) -> R {
        inspect(self.value())
    }

    pub(crate) const fn value(&self) -> u64 {
        u64::from_be_bytes(self.0)
    }

    #[cfg(feature = "serde")]
    pub(crate) const fn identity_key(&self) -> [u8; 8] {
        self.0
    }
}

impl Drop for RobotServerNumber {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.0);
    }
}

impl fmt::Debug for RobotServerNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerNumber([redacted])")
    }
}
