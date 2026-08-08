//! Source-complete SSH-key response model.

use alloc::string::String;
use core::fmt;

use cloud_sdk_sanitization::{sanitize_bytes, sanitize_string};

use super::cloud_schema::validate_model;
use super::ssh_wire::validate_key_identity;
use super::wipe_string::WipeString;
use super::{
    Labels, ResponseModelError, SensitiveText, UtcTimestamp, parse_labels, required, value_text,
};
use crate::security::ssh_keys::MAX_SSH_PUBLIC_KEY_BYTES;
use crate::serde::strict_json::Value;

const MAX_LABELS: usize = 64;
const MAX_NAME_BYTES: usize = 256;
const MD5_FINGERPRINT_BYTES: usize = 47;

/// Source-complete SSH key returned by the checked decoder.
///
/// The public key is protected and available only through closure-scoped
/// access. Ordinary equality and infallible cloning are intentionally absent.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::SshKey;
/// fn compare(left: SshKey, right: SshKey) -> bool { left == right }
/// ```
#[non_exhaustive]
pub struct SshKey {
    id: u64,
    name: String,
    fingerprint: String,
    sha256_fingerprint: [u8; 32],
    public_key: SensitiveText,
    labels: Labels,
    created: UtcTimestamp,
}

impl SshKey {
    /// Returns the provider identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the resource name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source-documented MD5 fingerprint.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Returns the SDK-computed SHA-256 fingerprint bytes.
    ///
    /// This fingerprint is derived from the RFC 4253 public-key encoding and
    /// is suitable for identity comparisons. [`Self::fingerprint`] exposes
    /// Hetzner's legacy MD5 compatibility field.
    #[must_use]
    pub const fn sha256_fingerprint(&self) -> &[u8; 32] {
        &self.sha256_fingerprint
    }

    /// Inspects the protected OpenSSH public key without returning a borrow.
    pub fn try_with_public_key<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.public_key.try_with_secret(inspect)
    }

    /// Returns user-defined labels.
    #[must_use]
    pub const fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub const fn created(&self) -> &UtcTimestamp {
        &self.created
    }
}

impl fmt::Debug for SshKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SshKey")
            .field("id", &"[redacted]")
            .field("fingerprint", &"[redacted]")
            .field("public_key", &"[redacted]")
            .field("fields", &"[redacted]")
            .finish()
    }
}

impl Drop for SshKey {
    fn drop(&mut self) {
        sanitize_string(&mut self.name);
        sanitize_string(&mut self.fingerprint);
        sanitize_bytes(&mut self.sha256_fingerprint);
    }
}

pub(crate) fn parse_ssh_key(value: &mut Value) -> Result<SshKey, ResponseModelError> {
    validate_model("ssh_key", value)?;
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let id = required(fields, "id")?
        .as_u64()
        .filter(|value| *value != 0 && *value <= 9_007_199_254_740_991)
        .ok_or(ResponseModelError::InvalidIdentifier)?;
    let name = WipeString::new(value_text(required(fields, "name")?, MAX_NAME_BYTES)?);
    let fingerprint = WipeString::new(value_text(
        required(fields, "fingerprint")?,
        MD5_FINGERPRINT_BYTES,
    )?);
    let supplied_fingerprint = parse_md5_fingerprint(fingerprint.as_str())?;
    let labels = parse_labels(required(fields, "labels")?, MAX_LABELS)?;
    let created = required(fields, "created")?
        .try_with_str(UtcTimestamp::try_new)
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)??;
    let public_key = fields
        .get_mut("public_key")
        .ok_or(ResponseModelError::MissingField)?
        .take_string()
        .map(SensitiveText::new)
        .ok_or(ResponseModelError::WrongType)?;
    public_key.validate(MAX_SSH_PUBLIC_KEY_BYTES)?;
    let sha256_fingerprint = public_key
        .try_with_secret(|value| validate_key_identity(value, supplied_fingerprint))
        .map_err(|_| ResponseModelError::InvalidText)??;
    Ok(SshKey {
        id,
        name: name.into_inner(),
        fingerprint: fingerprint.into_inner(),
        sha256_fingerprint,
        public_key,
        labels,
        created,
    })
}

fn parse_md5_fingerprint(value: &str) -> Result<[u8; 16], ResponseModelError> {
    if value.len() != MD5_FINGERPRINT_BYTES {
        return Err(ResponseModelError::InvalidText);
    }
    let mut output = [0_u8; 16];
    let mut octets = value.split(':');
    for byte in &mut output {
        let octet = octets.next().ok_or(ResponseModelError::InvalidText)?;
        if octet.len() != 2 {
            return Err(ResponseModelError::InvalidText);
        }
        *byte = u8::from_str_radix(octet, 16).map_err(|_| ResponseModelError::InvalidText)?;
    }
    if octets.next().is_some() {
        return Err(ResponseModelError::InvalidText);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::parse_md5_fingerprint;

    #[test]
    fn fingerprint_requires_exactly_sixteen_colon_separated_octets() {
        assert_eq!(
            parse_md5_fingerprint("00:11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff"),
            Ok([
                0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
                0xee, 0xff,
            ])
        );
        for invalid in [
            "00:11",
            "00112233445566778899aabbccddeeff",
            "00:11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:gg",
        ] {
            assert_eq!(
                parse_md5_fingerprint(invalid),
                Err(super::ResponseModelError::InvalidText)
            );
        }
    }
}
