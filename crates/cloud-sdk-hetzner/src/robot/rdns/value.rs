use cloud_sdk_sanitization::SecretBoxBytes;

/// Maximum canonical reverse-DNS name length admitted by the SDK.
pub const MAX_ROBOT_RDNS_NAME_BYTES: usize = 253;

/// Failure while validating a canonical Robot PTR name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotRdnsNameError {
    /// The name is empty, non-ASCII, noncanonical, or violates DNS label bounds.
    Invalid,
    /// Protected owned storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotRdnsNameError,
    Self::Invalid => "Robot reverse-DNS name is invalid",
    Self::Allocation => "Robot reverse-DNS name allocation failed",
);

/// Canonical protected ASCII PTR target without a trailing root dot.
pub struct RobotRdnsName(SecretBoxBytes);

impl RobotRdnsName {
    /// Validates lowercase DNS host syntax and stores the result in protected memory.
    pub fn new(value: &str) -> Result<Self, RobotRdnsNameError> {
        if !valid_name(value.as_bytes()) {
            return Err(RobotRdnsNameError::Invalid);
        }
        SecretBoxBytes::try_from_slice(value.as_bytes(), MAX_ROBOT_RDNS_NAME_BYTES)
            .map(Self)
            .map_err(|_| RobotRdnsNameError::Allocation)
    }

    /// Runs a closure with temporary access to the canonical PTR text.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0
            .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
    }

    pub(crate) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        self.0.with_secret(|bytes| {
            let text = core::str::from_utf8(bytes)
                .unwrap_or_else(|_| unreachable!("protected PTR name lost UTF-8"));
            inspect(text)
        })
    }
}

impl PartialEq for RobotRdnsName {
    fn eq(&self, other: &Self) -> bool {
        other.0.with_secret(|right| self.0.constant_time_eq(right))
    }
}

impl Eq for RobotRdnsName {}

impl core::fmt::Debug for RobotRdnsName {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotRdnsName([redacted])")
    }
}

fn valid_name(value: &[u8]) -> bool {
    if value.is_empty()
        || value.len() > MAX_ROBOT_RDNS_NAME_BYTES
        || value.starts_with(b".")
        || value.ends_with(b".")
    {
        return false;
    }
    value.split(|byte| *byte == b'.').all(valid_label)
}

fn valid_label(label: &[u8]) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with(b"-")
        && !label.ends_with(b"-")
        && label
            .iter()
            .all(|byte| matches!(*byte, b'a'..=b'z' | b'0'..=b'9' | b'-'))
}

#[cfg(test)]
mod tests {
    use alloc::{format, string::String};

    use super::*;

    #[test]
    fn ptr_names_are_bounded_canonical_and_redacted() {
        let name = RobotRdnsName::new("mail-1.example.com")
            .unwrap_or_else(|_| unreachable!("valid PTR name was rejected"));
        assert_eq!(
            name.try_with_text(|value| String::from(value))
                .unwrap_or_else(|_| unreachable!("PTR name lost UTF-8")),
            "mail-1.example.com"
        );
        assert_eq!(format!("{name:?}"), "RobotRdnsName([redacted])");

        for invalid in [
            "",
            ".example.com",
            "example.com.",
            "Example.com",
            "-mail.example.com",
            "mail-.example.com",
            "mail..example.com",
            "mail_example.com",
            "m\u{e5}l.example.com",
        ] {
            assert_eq!(
                RobotRdnsName::new(invalid).err(),
                Some(RobotRdnsNameError::Invalid)
            );
        }
    }

    #[test]
    fn ptr_name_accepts_exact_dns_bounds() {
        let label = "a".repeat(63);
        let maximum = format!("{label}.{label}.{label}.{}", "b".repeat(61));
        assert_eq!(maximum.len(), MAX_ROBOT_RDNS_NAME_BYTES);
        assert!(RobotRdnsName::new(&maximum).is_ok());
        assert_eq!(
            RobotRdnsName::new(&format!("{maximum}x")).err(),
            Some(RobotRdnsNameError::Invalid)
        );
        assert_eq!(
            RobotRdnsName::new(&format!("{}x.example", "a".repeat(63))).err(),
            Some(RobotRdnsNameError::Invalid)
        );
    }
}
