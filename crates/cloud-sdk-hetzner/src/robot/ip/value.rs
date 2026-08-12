use core::cmp::Ordering;

use cloud_sdk_sanitization::SecretBoxBytes;

/// Failure while validating a canonical Robot MAC address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotMacAddressError {
    /// The value is not six lowercase hexadecimal octets separated by colons.
    Invalid,
    /// Stable protected storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotMacAddressError,
    Self::Invalid => "Robot MAC address is not canonical",
    Self::Allocation => "Robot MAC address allocation failed",
);

/// Canonical protected EUI-48 address returned by Robot.
pub struct RobotMacAddress(SecretBoxBytes);

impl RobotMacAddress {
    /// Accepts exactly `xx:xx:xx:xx:xx:xx` with lowercase hexadecimal digits.
    pub fn new(value: &str) -> Result<Self, RobotMacAddressError> {
        let bytes = value.as_bytes();
        if bytes.len() != 17
            || bytes.iter().enumerate().any(|(index, byte)| {
                if matches!(index, 2 | 5 | 8 | 11 | 14) {
                    *byte != b':'
                } else {
                    !matches!(*byte, b'0'..=b'9' | b'a'..=b'f')
                }
            })
        {
            return Err(RobotMacAddressError::Invalid);
        }
        SecretBoxBytes::try_from_slice(bytes, 17)
            .map(Self)
            .map_err(|_| RobotMacAddressError::Allocation)
    }

    /// Runs a closure with temporary access to the canonical text.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0
            .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
    }

    #[cfg(test)]
    pub(super) fn equals_text(&self, expected: &str) -> bool {
        self.0.with_secret(|bytes| bytes == expected.as_bytes())
    }
}

impl PartialEq for RobotMacAddress {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .with_secret(|left| other.0.with_secret(|right| left.cmp(right)))
            == Ordering::Equal
    }
}

impl Eq for RobotMacAddress {}

impl core::fmt::Debug for RobotMacAddress {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotMacAddress([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::{RobotMacAddress, RobotMacAddressError};

    #[test]
    fn mac_text_is_exact_and_canonical() {
        let value = RobotMacAddress::new("00:21:85:62:3e:9c")
            .unwrap_or_else(|_| unreachable!("canonical MAC was rejected"));
        assert!(value.equals_text("00:21:85:62:3e:9c"));
        for invalid in [
            "",
            "0:21:85:62:3e:9c",
            "00-21-85-62-3e-9c",
            "00:21:85:62:3E:9c",
            "gg:21:85:62:3e:9c",
            "00:21:85:62:3e:9c:00",
        ] {
            assert_eq!(
                RobotMacAddress::new(invalid).err(),
                Some(RobotMacAddressError::Invalid)
            );
        }
    }
}
