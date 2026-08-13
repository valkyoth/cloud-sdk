use cloud_sdk_sanitization::SecretBoxBytes;

use crate::security::ssh_keys::SshPublicKey;

/// Maximum Robot SSH-key name length admitted by SDK policy.
pub const MAX_ROBOT_SSH_KEY_NAME_BYTES: usize = 128;
/// Maximum Robot SSH public-key input length admitted by SDK policy.
pub const MAX_ROBOT_SSH_KEY_DATA_BYTES: usize = 8_192;
const FINGERPRINT_BYTES: usize = 47;

/// Failure while validating a Robot SSH-key value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotSshKeyValueError {
    /// Text was empty, malformed, noncanonical, or exceeded its bound.
    Invalid,
    /// Protected owned storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotSshKeyValueError,
    Self::Invalid => "Robot SSH-key value is invalid",
    Self::Allocation => "Robot SSH-key protected storage allocation failed",
);

/// Protected printable Robot SSH-key name.
pub struct RobotSshKeyName(SecretBoxBytes);

impl RobotSshKeyName {
    /// Validates and protects a non-empty printable key name.
    pub fn new(value: &str) -> Result<Self, RobotSshKeyValueError> {
        if value.is_empty()
            || value.len() > MAX_ROBOT_SSH_KEY_NAME_BYTES
            || value
                .chars()
                .any(crate::display::is_unsafe_display_character)
        {
            return Err(RobotSshKeyValueError::Invalid);
        }
        protect(value, MAX_ROBOT_SSH_KEY_NAME_BYTES).map(Self)
    }

    /// Runs a closure with temporary access to the exact name.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0
            .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
    }

    pub(super) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        with_text(&self.0, inspect)
    }
}

impl PartialEq for RobotSshKeyName {
    fn eq(&self, other: &Self) -> bool {
        other.0.with_secret(|right| self.0.constant_time_eq(right))
    }
}
impl Eq for RobotSshKeyName {}
impl core::fmt::Debug for RobotSshKeyName {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSshKeyName([redacted])")
    }
}

/// Borrowed OpenSSH or RFC 4716 SSH2 public-key input.
pub struct RobotSshKeyData<'a>(&'a str);

impl<'a> RobotSshKeyData<'a> {
    /// Validates a bounded source-documented public-key representation.
    pub fn new(value: &'a str) -> Result<Self, RobotSshKeyValueError> {
        if value.len() > MAX_ROBOT_SSH_KEY_DATA_BYTES
            || !(SshPublicKey::new(value).is_ok() || valid_ssh2(value))
        {
            return Err(RobotSshKeyValueError::Invalid);
        }
        Ok(Self(value))
    }

    /// Runs a closure with temporary access to the public-key input.
    pub fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        inspect(self.0)
    }
}

impl core::fmt::Debug for RobotSshKeyData<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSshKeyData([redacted])")
    }
}

/// Canonical protected lowercase MD5 fingerprint used by Robot paths.
pub struct RobotSshKeyFingerprint(SecretBoxBytes);

impl RobotSshKeyFingerprint {
    /// Validates exactly 16 lowercase colon-separated octets.
    pub fn new(value: &str) -> Result<Self, RobotSshKeyValueError> {
        if !valid_fingerprint(value.as_bytes()) {
            return Err(RobotSshKeyValueError::Invalid);
        }
        protect(value, FINGERPRINT_BYTES).map(Self)
    }

    /// Runs a closure with temporary access to the canonical fingerprint.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0
            .with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
    }

    pub(super) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        with_text(&self.0, inspect)
    }
}

impl PartialEq for RobotSshKeyFingerprint {
    fn eq(&self, other: &Self) -> bool {
        other.0.with_secret(|right| self.0.constant_time_eq(right))
    }
}
impl Eq for RobotSshKeyFingerprint {}
impl core::fmt::Debug for RobotSshKeyFingerprint {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSshKeyFingerprint([redacted])")
    }
}

fn protect(value: &str, maximum: usize) -> Result<SecretBoxBytes, RobotSshKeyValueError> {
    SecretBoxBytes::try_from_slice(value.as_bytes(), maximum)
        .map_err(|_| RobotSshKeyValueError::Allocation)
}

fn with_text<R>(value: &SecretBoxBytes, inspect: impl FnOnce(&str) -> R) -> R {
    value.with_secret(|bytes| {
        let text = core::str::from_utf8(bytes)
            .unwrap_or_else(|_| unreachable!("protected Robot SSH-key text lost UTF-8"));
        inspect(text)
    })
}

fn valid_fingerprint(value: &[u8]) -> bool {
    value.len() == FINGERPRINT_BYTES
        && value.iter().enumerate().all(|(index, byte)| {
            if index % 3 == 2 {
                *byte == b':'
            } else {
                matches!(*byte, b'0'..=b'9' | b'a'..=b'f')
            }
        })
}

fn valid_ssh2(value: &str) -> bool {
    const BEGIN: &str = "---- BEGIN SSH2 PUBLIC KEY ----";
    const END: &str = "---- END SSH2 PUBLIC KEY ----";
    if value
        .bytes()
        .any(|byte| byte == b'\r' || (byte < 0x20 && byte != b'\n') || byte == 0x7f)
    {
        return false;
    }
    let mut lines = value.lines();
    if lines.next() != Some(BEGIN) {
        return false;
    }
    let mut encoded = 0_usize;
    let mut headers_done = false;
    for line in &mut lines {
        if line == END {
            return encoded != 0 && lines.next().is_none();
        }
        if line.is_empty() || line.len() > 72 {
            return false;
        }
        let base64 = line
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='));
        if base64 {
            headers_done = true;
            encoded = match encoded.checked_add(line.len()) {
                Some(value) => value,
                None => return false,
            };
        } else if headers_done || !valid_ssh2_header(line) {
            return false;
        }
    }
    false
}

fn valid_ssh2_header(line: &str) -> bool {
    let Some((name, value)) = line.split_once(':') else {
        return false;
    };
    !name.is_empty()
        && !value.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && !value
            .chars()
            .any(crate::display::is_unsafe_display_character)
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use super::*;

    #[test]
    fn values_are_bounded_canonical_and_redacted() {
        let name = RobotSshKeyName::new("deploy-key")
            .unwrap_or_else(|_| unreachable!("valid name rejected"));
        let fingerprint =
            RobotSshKeyFingerprint::new("15:28:b0:03:95:f0:77:b3:10:56:15:6b:77:22:a5:bb")
                .unwrap_or_else(|_| unreachable!("valid fingerprint rejected"));
        assert_eq!(format!("{name:?}"), "RobotSshKeyName([redacted])");
        assert_eq!(
            format!("{fingerprint:?}"),
            "RobotSshKeyFingerprint([redacted])"
        );
        assert_eq!(
            RobotSshKeyFingerprint::new("15:28:B0:03:95:f0:77:b3:10:56:15:6b:77:22:a5:bb").err(),
            Some(RobotSshKeyValueError::Invalid)
        );
        for character in [
            '\u{0085}', '\u{061c}', '\u{200b}', '\u{202e}', '\u{2069}', '\u{feff}',
        ] {
            assert_eq!(
                RobotSshKeyName::new(&format!("prod{character}key")).err(),
                Some(RobotSshKeyValueError::Invalid)
            );
            assert_eq!(
                RobotSshKeyData::new(&format!("ssh-ed25519 AAAA prod{character}key")).err(),
                Some(RobotSshKeyValueError::Invalid)
            );
        }
    }

    #[test]
    fn ssh2_armor_requires_one_bounded_payload() {
        let valid = "---- BEGIN SSH2 PUBLIC KEY ----\nComment: test\nAAAAC3NzaC1lZDI1NTE5AAAAIA==\n---- END SSH2 PUBLIC KEY ----";
        assert!(RobotSshKeyData::new(valid).is_ok());
        for invalid in [
            "",
            "---- BEGIN SSH2 PUBLIC KEY ----\n---- END SSH2 PUBLIC KEY ----",
            "---- BEGIN SSH2 PUBLIC KEY ----\nAAAA\r\n---- END SSH2 PUBLIC KEY ----",
            "---- BEGIN SSH2 PUBLIC KEY ----\nAAAA\n---- END SSH2 PUBLIC KEY ----\nextra",
        ] {
            assert_eq!(
                RobotSshKeyData::new(invalid).err(),
                Some(RobotSshKeyValueError::Invalid)
            );
        }
        for character in [
            '\u{0085}', '\u{061c}', '\u{200b}', '\u{202e}', '\u{2069}', '\u{feff}',
        ] {
            let unsafe_header = format!(
                "---- BEGIN SSH2 PUBLIC KEY ----\nComment: prod{character}key\nAAAA\n---- END SSH2 PUBLIC KEY ----"
            );
            assert_eq!(
                RobotSshKeyData::new(&unsafe_header).err(),
                Some(RobotSshKeyValueError::Invalid)
            );
        }
    }
}
