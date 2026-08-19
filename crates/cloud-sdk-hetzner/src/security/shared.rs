//! Shared security-resource request helpers.

use core::fmt;

use cloud_sdk::buffer::{self, SnapshotEncoder};
use cloud_sdk::transport::MAX_REQUEST_TARGET_BYTES;

use crate::labels::{LabelError, LabelKey, LabelValue};
use crate::request::{EndpointPath, EndpointPathError};

/// Maximum security resource name length admitted by the SDK policy.
pub const MAX_SECURITY_NAME_BYTES: usize = 128;

/// Maximum SSH public key length admitted by the SDK policy.
pub const MAX_SSH_PUBLIC_KEY_BYTES: usize = 8192;

/// Maximum PEM value length admitted by the SDK policy.
pub const MAX_PEM_BYTES: usize = 65_536;

/// Maximum SSH fingerprint length admitted by the SDK policy.
pub const MAX_SSH_FINGERPRINT_BYTES: usize = 64;

/// Error returned while building security request components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityRequestError {
    /// Endpoint paths failed validation.
    InvalidPath(EndpointPathError),
    /// Labels failed validation.
    InvalidLabel(LabelError),
    /// Resource names must not be empty.
    EmptyName,
    /// The resource name exceeds the internal maximum length.
    NameTooLong,
    /// Resource names must not contain control bytes.
    InvalidNameByte,
    /// Caller-provided path buffer is too small.
    PathBufferTooSmall,
    /// Caller-provided query buffer is too small.
    QueryBufferTooSmall,
    /// Caller-provided request-body buffer is too small.
    BodyBufferTooSmall,
    /// Decimal conversion failed.
    NumberEncodingFailed,
    /// Path bytes failed UTF-8 conversion after construction.
    PathEncodingFailed,
    /// SSH public key failed conservative shape validation.
    InvalidSshPublicKey,
    /// SSH fingerprint failed conservative shape validation.
    InvalidSshFingerprint,
    /// PEM body failed conservative shape validation.
    InvalidPem,
    /// Domain name failed conservative shape validation.
    InvalidDomainName,
    /// Managed certificate requests require at least one domain name.
    EmptyDomainNames,
    /// A certificate action query contains too many action IDs.
    TooManyActionIds,
    /// A certificate action query contains too many status filters.
    TooManyActionStatuses,
    /// A certificate action query contains too many sort values.
    TooManyActionSorts,
}

impl_static_error!(SecurityRequestError,
    Self::InvalidPath(_) => "security endpoint path is invalid",
    Self::InvalidLabel(_) => "security resource label is invalid",
    Self::EmptyName => "security resource name is empty",
    Self::NameTooLong => "security resource name exceeds the length limit",
    Self::InvalidNameByte => "security resource name contains an invalid byte",
    Self::PathBufferTooSmall => "security path buffer is too small",
    Self::QueryBufferTooSmall => "security query buffer is too small",
    Self::BodyBufferTooSmall => "security request-body buffer is too small",
    Self::NumberEncodingFailed => "security number encoding failed",
    Self::PathEncodingFailed => "security path encoding failed",
    Self::InvalidSshPublicKey => "SSH public key is invalid",
    Self::InvalidSshFingerprint => "SSH fingerprint is invalid",
    Self::InvalidPem => "PEM body is invalid",
    Self::InvalidDomainName => "certificate domain name is invalid",
    Self::EmptyDomainNames => "managed certificate domain list is empty",
    Self::TooManyActionIds => "certificate action ID filter exceeds the item limit",
    Self::TooManyActionStatuses => "certificate action status filter exceeds the item limit",
    Self::TooManyActionSorts => "certificate action sort list exceeds the item limit",
);

/// Nonzero identifier for security resources.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SecurityId(u64);

impl SecurityId {
    /// Creates a nonzero security resource identifier.
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the raw identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Borrowed, validated security resource name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityName<'a> {
    value: &'a str,
}

impl<'a> SecurityName<'a> {
    /// Creates a bounded printable name.
    pub fn new(value: &'a str) -> Result<Self, SecurityRequestError> {
        if value.is_empty() {
            return Err(SecurityRequestError::EmptyName);
        }
        if value.len() > MAX_SECURITY_NAME_BYTES {
            return Err(SecurityRequestError::NameTooLong);
        }
        if has_control_byte(value) {
            return Err(SecurityRequestError::InvalidNameByte);
        }
        Ok(Self { value })
    }

    /// Returns the name.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Borrowed SSH public key. Debug output is redacted.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SshPublicKey<'a> {
    value: &'a str,
}

impl<'a> SshPublicKey<'a> {
    /// Creates a conservatively validated SSH public key value.
    pub fn new(value: &'a str) -> Result<Self, SecurityRequestError> {
        if value.is_empty()
            || value.len() > MAX_SSH_PUBLIC_KEY_BYTES
            || value
                .chars()
                .any(crate::display::is_unsafe_display_character)
        {
            return Err(SecurityRequestError::InvalidSshPublicKey);
        }
        let mut fields = value.split(' ');
        let algorithm = fields
            .next()
            .ok_or(SecurityRequestError::InvalidSshPublicKey)?;
        let key = fields
            .next()
            .ok_or(SecurityRequestError::InvalidSshPublicKey)?;
        if ssh_algorithm(algorithm).is_none() || key.is_empty() || !is_base64ish(key) {
            return Err(SecurityRequestError::InvalidSshPublicKey);
        }
        Ok(Self { value })
    }

    /// Returns the public key.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

impl fmt::Debug for SshPublicKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SshPublicKey([redacted])")
    }
}

/// Borrowed PEM value. Debug output is redacted.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PemValue<'a> {
    value: &'a str,
}

impl<'a> PemValue<'a> {
    /// Creates a bounded PEM-like value with required markers.
    pub fn new(
        value: &'a str,
        begin_marker: &str,
        end_marker: &str,
    ) -> Result<Self, SecurityRequestError> {
        if value.is_empty() || value.len() > MAX_PEM_BYTES {
            return Err(SecurityRequestError::InvalidPem);
        }
        let begin_at = value
            .find(begin_marker)
            .ok_or(SecurityRequestError::InvalidPem)?;
        let after_begin = begin_at
            .checked_add(begin_marker.len())
            .ok_or(SecurityRequestError::InvalidPem)?;
        let remainder = value
            .get(after_begin..)
            .ok_or(SecurityRequestError::InvalidPem)?;
        let end_at = remainder
            .find(end_marker)
            .and_then(|offset| after_begin.checked_add(offset))
            .ok_or(SecurityRequestError::InvalidPem)?;
        if end_at <= after_begin {
            return Err(SecurityRequestError::InvalidPem);
        }
        Ok(Self { value })
    }

    /// Returns the PEM value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

impl fmt::Debug for PemValue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PemValue([redacted])")
    }
}

/// Borrowed, validated DNS name for managed certificates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateDomainName<'a> {
    value: &'a str,
}

impl<'a> CertificateDomainName<'a> {
    /// Creates a conservative domain name value.
    pub fn new(value: &'a str) -> Result<Self, SecurityRequestError> {
        let domain = value.strip_prefix("*.").unwrap_or(value);
        if domain.is_empty()
            || domain.len() > 253
            || domain.starts_with('.')
            || domain.ends_with('.')
        {
            return Err(SecurityRequestError::InvalidDomainName);
        }
        for label in domain.split('.') {
            if label.is_empty()
                || label.len() > 63
                || !has_alphanumeric_boundaries(label)
                || !label.bytes().all(is_domain_label_byte)
            {
                return Err(SecurityRequestError::InvalidDomainName);
            }
        }
        Ok(Self { value })
    }

    /// Returns the domain name.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Borrowed label entries for a security resource body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecurityLabels<'a> {
    entries: &'a [(LabelKey<'a>, LabelValue<'a>)],
}

impl<'a> SecurityLabels<'a> {
    /// Creates a borrowed label slice after validating deterministic key order.
    pub fn new(
        entries: &'a [(LabelKey<'a>, LabelValue<'a>)],
    ) -> Result<Self, SecurityRequestError> {
        let mut previous: Option<&str> = None;
        for (key, _) in entries {
            if let Some(previous) = previous
                && previous >= key.as_str()
            {
                return Err(SecurityRequestError::InvalidLabel(
                    LabelError::InvalidSelectorSyntax,
                ));
            }
            previous = Some(key.as_str());
        }
        Ok(Self { entries })
    }

    /// Returns the borrowed label entries.
    #[must_use]
    pub const fn entries(self) -> &'a [(LabelKey<'a>, LabelValue<'a>)] {
        self.entries
    }
}

/// Writes a static endpoint path.
pub fn static_path(value: &'static str) -> Result<EndpointPath<'static>, SecurityRequestError> {
    EndpointPath::new(value).map_err(SecurityRequestError::InvalidPath)
}

/// Writes `{prefix}{id}{suffix}` into a caller-owned path buffer.
pub fn write_id_path(
    output: &mut [u8],
    prefix: &str,
    id: SecurityId,
    suffix: &str,
) -> Result<usize, SecurityRequestError> {
    let len = buffer::encode_snapshot_bounded(
        (prefix, id, suffix),
        output,
        crate::request::MAX_ENDPOINT_PATH_BYTES,
        SecurityRequestError::PathBufferTooSmall,
        |(prefix, id, suffix), encoder| {
            encoder.string(prefix)?;
            encoder.u64(id.get())?;
            encoder.string(suffix)
        },
    )?;
    if let Err(error) = validate_written_path(output, len) {
        if let Some(target) = output.get_mut(..len) {
            cloud_sdk_sanitization::sanitize_bytes(target);
        }
        return Err(error);
    }
    Ok(len)
}

/// Atomically encodes one immutable security query snapshot.
pub(crate) fn encode_security_query<F>(
    output: &mut [u8],
    encode: F,
) -> Result<usize, SecurityRequestError>
where
    F: Copy
        + for<'encoder> Fn(
            &mut SnapshotEncoder<'encoder, SecurityRequestError>,
            &mut bool,
        ) -> Result<(), SecurityRequestError>,
{
    buffer::encode_snapshot_bounded(
        encode,
        output,
        MAX_REQUEST_TARGET_BYTES,
        SecurityRequestError::QueryBufferTooSmall,
        |encode, encoder| {
            let mut first = true;
            encode(encoder, &mut first)
        },
    )
}

fn validate_written_path(output: &[u8], len: usize) -> Result<(), SecurityRequestError> {
    let bytes = output
        .get(..len)
        .ok_or(SecurityRequestError::PathBufferTooSmall)?;
    let path = core::str::from_utf8(bytes).map_err(|_| SecurityRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(SecurityRequestError::InvalidPath)?;
    Ok(())
}

fn has_control_byte(value: &str) -> bool {
    value.bytes().any(|byte| byte < 0x20 || byte == 0x7f)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SshAlgorithm {
    Ed25519,
    Rsa,
    EcdsaNistP256,
    EcdsaNistP384,
    EcdsaNistP521,
    SkEd25519,
    SkEcdsaNistP256,
}

impl SshAlgorithm {
    #[cfg(feature = "serde")]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ed25519 => "ssh-ed25519",
            Self::Rsa => "ssh-rsa",
            Self::EcdsaNistP256 => "ecdsa-sha2-nistp256",
            Self::EcdsaNistP384 => "ecdsa-sha2-nistp384",
            Self::EcdsaNistP521 => "ecdsa-sha2-nistp521",
            Self::SkEd25519 => "sk-ssh-ed25519@openssh.com",
            Self::SkEcdsaNistP256 => "sk-ecdsa-sha2-nistp256@openssh.com",
        }
    }
}

pub(crate) fn ssh_algorithm(value: &str) -> Option<SshAlgorithm> {
    match value {
        "ssh-ed25519" => Some(SshAlgorithm::Ed25519),
        "ssh-rsa" => Some(SshAlgorithm::Rsa),
        "ecdsa-sha2-nistp256" => Some(SshAlgorithm::EcdsaNistP256),
        "ecdsa-sha2-nistp384" => Some(SshAlgorithm::EcdsaNistP384),
        "ecdsa-sha2-nistp521" => Some(SshAlgorithm::EcdsaNistP521),
        "sk-ssh-ed25519@openssh.com" => Some(SshAlgorithm::SkEd25519),
        "sk-ecdsa-sha2-nistp256@openssh.com" => Some(SshAlgorithm::SkEcdsaNistP256),
        _ => None,
    }
}

fn is_base64ish(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
}

fn has_alphanumeric_boundaries(value: &str) -> bool {
    match (value.as_bytes().first(), value.as_bytes().last()) {
        (Some(first), Some(last)) => first.is_ascii_alphanumeric() && last.is_ascii_alphanumeric(),
        _ => false,
    }
}

fn is_domain_label_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-'
}
