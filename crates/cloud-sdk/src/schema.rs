//! Explicit provider schema-version validation contracts.

use core::fmt;

use cloud_sdk_sanitization::SecretBuffer;

use crate::buffer::write_u64;
use crate::transport::{HeaderName, RequestHeader};

/// Schema-version validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaVersionError {
    /// Major schema versions must be nonzero.
    ZeroMajor,
    /// A schema version was not canonical `major.minor` decimal text.
    InvalidVersion,
    /// The selected version differs from the reviewed major version.
    UnreviewedMajor,
    /// The validation-only header name is invalid.
    InvalidHeader,
    /// Caller scratch cannot hold the complete encoded version.
    OutputTooSmall,
}

impl_static_error!(SchemaVersionError,
    Self::ZeroMajor => "schema major version must be nonzero",
    Self::InvalidVersion => "schema version is not canonical major.minor text",
    Self::UnreviewedMajor => "schema version differs from the reviewed major",
    Self::InvalidHeader => "schema validation header is invalid",
    Self::OutputTooSmall => "schema version output is too small",
);

/// Canonical provider schema version with explicit major and minor parts.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion {
    major: u16,
    minor: u16,
}

impl SchemaVersion {
    /// Creates a schema version with a nonzero major component.
    pub const fn new(major: u16, minor: u16) -> Result<Self, SchemaVersionError> {
        if major == 0 {
            return Err(SchemaVersionError::ZeroMajor);
        }
        Ok(Self { major, minor })
    }

    /// Parses strict canonical `major.minor` ASCII decimal text.
    pub fn parse(value: &[u8]) -> Result<Self, SchemaVersionError> {
        let dot = value
            .iter()
            .position(|byte| *byte == b'.')
            .ok_or(SchemaVersionError::InvalidVersion)?;
        if value
            .get(dot.saturating_add(1)..)
            .is_none_or(|part| part.is_empty())
            || value.get(..dot).is_none_or(|part| part.is_empty())
            || value
                .get(dot.saturating_add(1)..)
                .is_some_and(|part| part.contains(&b'.'))
        {
            return Err(SchemaVersionError::InvalidVersion);
        }
        let major = parse_component(value.get(..dot).ok_or(SchemaVersionError::InvalidVersion)?)?;
        let minor = parse_component(
            value
                .get(dot.saturating_add(1)..)
                .ok_or(SchemaVersionError::InvalidVersion)?,
        )?;
        Self::new(major, minor)
    }

    /// Returns the major version selected by the account or validation probe.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the minor schema version.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }
}

/// Source-reviewed major version and immutable evidence digest.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ReviewedSchemaMajor {
    major: u16,
    source_sha256: [u8; 32],
}

impl ReviewedSchemaMajor {
    /// Binds one nonzero major to the exact reviewed source digest.
    pub const fn new(major: u16, source_sha256: [u8; 32]) -> Result<Self, SchemaVersionError> {
        if major == 0 {
            return Err(SchemaVersionError::ZeroMajor);
        }
        Ok(Self {
            major,
            source_sha256,
        })
    }

    /// Returns the reviewed major version.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the exact SHA-256 source-lock evidence.
    #[must_use]
    pub const fn source_sha256(self) -> [u8; 32] {
        self.source_sha256
    }

    /// Rejects a version from any unreviewed major line.
    pub const fn validate(self, version: SchemaVersion) -> Result<(), SchemaVersionError> {
        if version.major != self.major {
            return Err(SchemaVersionError::UnreviewedMajor);
        }
        Ok(())
    }
}

impl fmt::Debug for ReviewedSchemaMajor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReviewedSchemaMajor")
            .field("major", &self.major)
            .field("source_sha256", &"[source-locked]")
            .finish()
    }
}

/// Explicit validation-only schema override header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationSchemaHeader<'a> {
    name: HeaderName<'a>,
    version: SchemaVersion,
    evidence: ReviewedSchemaMajor,
}

impl<'a> ValidationSchemaHeader<'a> {
    /// Creates an override only when its major matches reviewed evidence.
    pub fn new(
        name: &'a str,
        version: SchemaVersion,
        evidence: ReviewedSchemaMajor,
    ) -> Result<Self, SchemaVersionError> {
        evidence.validate(version)?;
        let name = HeaderName::new(name).map_err(|_| SchemaVersionError::InvalidHeader)?;
        RequestHeader::new(name.as_str(), "1.0").map_err(|_| SchemaVersionError::InvalidHeader)?;
        Ok(Self {
            name,
            version,
            evidence,
        })
    }

    /// Returns the validation-only header name.
    #[must_use]
    pub const fn name(self) -> HeaderName<'a> {
        self.name
    }

    /// Returns the exact reviewed schema version.
    #[must_use]
    pub const fn version(self) -> SchemaVersion {
        self.version
    }

    /// Returns the evidence binding used to admit this override.
    #[must_use]
    pub const fn evidence(self) -> ReviewedSchemaMajor {
        self.evidence
    }

    /// Builds the public validation header only for the duration of `inspect`.
    ///
    /// This deliberately named method keeps the override out of default
    /// production request construction.
    pub fn with_validation_header<R>(
        self,
        scratch: &mut [u8],
        inspect: impl FnOnce(RequestHeader<'_>) -> R,
    ) -> Result<R, SchemaVersionError> {
        let mut scratch = SecretBuffer::new(scratch);
        let mut len = 0_usize;
        write_u64(
            scratch.as_mut_slice(),
            &mut len,
            u64::from(self.version.major),
            SchemaVersionError::OutputTooSmall,
        )?;
        crate::buffer::write_byte(
            scratch.as_mut_slice(),
            &mut len,
            b'.',
            SchemaVersionError::OutputTooSmall,
        )?;
        write_u64(
            scratch.as_mut_slice(),
            &mut len,
            u64::from(self.version.minor),
            SchemaVersionError::OutputTooSmall,
        )?;
        let value = core::str::from_utf8(
            scratch
                .as_slice()
                .get(..len)
                .ok_or(SchemaVersionError::OutputTooSmall)?,
        )
        .map_err(|_| SchemaVersionError::InvalidVersion)?;
        let header = RequestHeader::new(self.name.as_str(), value)
            .map_err(|_| SchemaVersionError::InvalidHeader)?;
        Ok(inspect(header))
    }
}

fn parse_component(value: &[u8]) -> Result<u16, SchemaVersionError> {
    if value.is_empty()
        || (value.len() > 1 && value.first() == Some(&b'0'))
        || !value.iter().all(u8::is_ascii_digit)
    {
        return Err(SchemaVersionError::InvalidVersion);
    }
    let mut parsed = 0_u16;
    for byte in value {
        let digit = byte
            .checked_sub(b'0')
            .ok_or(SchemaVersionError::InvalidVersion)?;
        parsed = parsed
            .checked_mul(10)
            .and_then(|current| current.checked_add(u16::from(digit)))
            .ok_or(SchemaVersionError::InvalidVersion)?;
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests;
