use super::{IdentifierError, MAX_VERSION_BYTES, decimal};
use core::fmt;

/// Exact Cargo SemVer version, not a version requirement or an alias.
///
/// Major/minor/patch fit `u64`, matching Cargo's `semver::Version`. Pre-release
/// numeric identifiers reject leading zeros but do not have a numeric-size cap.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Version<'a>(&'a str);

impl<'a> Version<'a> {
    /// Checks the complete ASCII grammar without allocating or normalizing.
    pub fn new(value: &'a str) -> Result<Self, IdentifierError> {
        if value.len() > MAX_VERSION_BYTES {
            return Err(IdentifierError::Limit);
        }
        let core_pre = match value.split_once('+') {
            Some((left, build)) => {
                identifiers(build, false)?;
                left
            }
            None => value,
        };
        let core = match core_pre.split_once('-') {
            Some((left, pre)) => {
                identifiers(pre, true)?;
                left
            }
            None => core_pre,
        };
        let mut numbers = core.split('.');
        for _ in 0..3 {
            decimal(numbers.next().ok_or(IdentifierError::Syntax)?)?;
        }
        if numbers.next().is_some() {
            return Err(IdentifierError::Syntax);
        }
        Ok(Self(value))
    }
    /// Returns the exact version, including build metadata.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

fn identifiers(value: &str, prerelease: bool) -> Result<(), IdentifierError> {
    for part in value.split('.') {
        if part.is_empty()
            || !part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || prerelease
                && part.len() > 1
                && part.starts_with('0')
                && part.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(IdentifierError::Syntax);
        }
    }
    Ok(())
}
impl fmt::Debug for Version<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Version([redacted])")
    }
}
