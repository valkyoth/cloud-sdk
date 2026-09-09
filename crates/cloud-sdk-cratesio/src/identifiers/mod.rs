//! Bounded borrowed crates.io identifiers. No normalization or allocation.

use core::fmt;

mod date;
mod version;
pub use date::Date;
pub use version::Version;

/// Upstream crate-name ceiling.
pub const MAX_CRATE_NAME_BYTES: usize = 64;
/// Upstream version ceiling (the admitted grammar is ASCII).
pub const MAX_VERSION_BYTES: usize = 150;
/// Upstream keyword publishing ceiling.
pub const MAX_KEYWORD_BYTES: usize = 20;
/// SDK bound for category hierarchy text and team login text.
pub const MAX_IDENTIFIER_BYTES: usize = 256;
/// SDK bound for a user login; not a claim about account availability.
pub const MAX_LOGIN_BYTES: usize = 100;

/// Payload-free identifier validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierError {
    /// Empty or syntactically invalid input.
    Syntax,
    /// A documented length or numeric limit was exceeded.
    Limit,
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Syntax => "invalid crates.io identifier syntax",
            Self::Limit => "crates.io identifier exceeds its bound",
        })
    }
}
impl core::error::Error for IdentifierError {}

macro_rules! borrowed_identifier {
    ($name:ident, $doc:literal, $max:expr, $check:expr) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Eq, PartialEq)]
        pub struct $name<'a>(&'a str);
        impl<'a> $name<'a> {
            /// Checks syntax and byte bounds without rewriting the input.
            pub fn new(value: &'a str) -> Result<Self, IdentifierError> {
                if value.len() > $max {
                    return Err(IdentifierError::Limit);
                }
                if !($check)(value) {
                    return Err(IdentifierError::Syntax);
                }
                Ok(Self(value))
            }
            /// Returns the exact validated identifier.
            #[must_use]
            pub const fn as_str(self) -> &'a str {
                self.0
            }
        }
        impl fmt::Debug for $name<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

borrowed_identifier!(
    CrateName,
    "ASCII crate name, preserving case and hyphen/underscore identity.",
    MAX_CRATE_NAME_BYTES,
    |s: &str| s.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
);
borrowed_identifier!(
    Keyword,
    "ASCII registry keyword; `+` is an allowed keyword byte.",
    MAX_KEYWORD_BYTES,
    |s: &str| s.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'+'))
);
borrowed_identifier!(
    CategorySlug,
    "Category hierarchy such as `development-tools::cargo-plugins`.",
    MAX_IDENTIFIER_BYTES,
    |s: &str| s.split("::").all(slug)
);
borrowed_identifier!(
    UserLogin,
    "Bounded ASCII account login; does not prove account existence.",
    MAX_LOGIN_BYTES,
    |s: &str| !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
);
borrowed_identifier!(
    TeamLogin,
    "GitHub team owner syntax `github:organization:team`.",
    MAX_IDENTIFIER_BYTES,
    valid_team
);

fn slug(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

fn valid_team(s: &str) -> bool {
    let Some(s) = s.strip_prefix("github:") else {
        return false;
    };
    let Some((org, team)) = s.split_once(':') else {
        return false;
    };
    UserLogin::new(org).is_ok()
        && !team.is_empty()
        && team
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

/// Explicit distinction between a user owner and a provider-qualified team.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Owner<'a> {
    /// Individual registry account.
    User(UserLogin<'a>),
    /// Provider-qualified organization team.
    Team(TeamLogin<'a>),
}
impl<'a> Owner<'a> {
    /// Parses the source-owned team prefix, or an individual login.
    pub fn new(value: &'a str) -> Result<Self, IdentifierError> {
        if value.contains(':') {
            TeamLogin::new(value).map(Self::Team)
        } else {
            UserLogin::new(value).map(Self::User)
        }
    }
    /// Returns exact owner text for a checked body encoder.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        match self {
            Self::User(v) => v.as_str(),
            Self::Team(v) => v.as_str(),
        }
    }
}

/// Positive identifier in the OpenAPI signed 32-bit domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumericId(u32);
impl NumericId {
    /// Rejects zero, negative textual forms, and values above `i32::MAX`.
    pub fn new(value: u64) -> Result<Self, IdentifierError> {
        if value == 0 {
            return Err(IdentifierError::Syntax);
        }
        if value > 2_147_483_647 {
            return Err(IdentifierError::Limit);
        }
        Ok(Self(
            u32::try_from(value).map_err(|_| IdentifierError::Limit)?,
        ))
    }
    /// Parses canonical decimal digits, without leading zeroes or signs.
    pub fn parse(value: &str) -> Result<Self, IdentifierError> {
        Self::new(decimal(value)?)
    }
    /// Returns the validated number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

pub(crate) fn decimal(value: &str) -> Result<u64, IdentifierError> {
    if value.is_empty()
        || value.len() > 20
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(IdentifierError::Syntax);
    }
    value.parse().map_err(|_| IdentifierError::Limit)
}

#[cfg(test)]
mod tests;
