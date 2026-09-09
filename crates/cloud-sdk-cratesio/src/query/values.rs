use super::{MAX_PAGE, MAX_PER_PAGE, MAX_QUERY_VALUE_BYTES, QueryError};
use crate::identifiers::decimal;
use core::fmt;

macro_rules! number {
    ($name:ident, $max:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(u32);
        impl $name {
            /// Checks the nonzero inclusive limit.
            pub fn new(value: u32) -> Result<Self, QueryError> {
                if value == 0 || value > $max {
                    return Err(QueryError::Limit);
                }
                Ok(Self(value))
            }
            /// Parses canonical unsigned decimal input.
            pub fn parse(value: &str) -> Result<Self, QueryError> {
                let value = decimal(value).map_err(|_| QueryError::Syntax)?;
                Self::new(u32::try_from(value).map_err(|_| QueryError::Limit)?)
            }
            /// Returns the validated number.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}
number!(
    Page,
    MAX_PAGE,
    "Numbered page, capped by the conservative SDK policy."
);
number!(
    PerPage,
    MAX_PER_PAGE,
    "Page size, capped by the source-locked server limit."
);

/// Bounded Unicode search text. Reserved URI bytes are encoded, not interpreted.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SearchQuery<'a>(&'a str);
impl<'a> SearchQuery<'a> {
    /// Rejects empty text, controls, backslashes and fragments before encoding.
    pub fn new(value: &'a str) -> Result<Self, QueryError> {
        if value.len() > MAX_QUERY_VALUE_BYTES {
            return Err(QueryError::Limit);
        }
        if value.is_empty()
            || value
                .chars()
                .any(|c| c.is_control() || matches!(c, '#' | '\\'))
        {
            return Err(QueryError::Syntax);
        }
        Ok(Self(value))
    }
    /// Returns the decoded query text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}
impl fmt::Debug for SearchQuery<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SearchQuery([redacted])")
    }
}

/// Provider-issued opaque URL-safe seek token. Never decode or construct its payload.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Seek<'a>(&'a str);
impl<'a> Seek<'a> {
    /// Bounds the opaque token and checks the source's unpadded URL-safe alphabet.
    pub fn new(value: &'a str) -> Result<Self, QueryError> {
        if value.len() > MAX_QUERY_VALUE_BYTES {
            return Err(QueryError::Limit);
        }
        if value.is_empty()
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        {
            return Err(QueryError::Syntax);
        }
        Ok(Self(value))
    }
    /// Lends opaque text to the checked encoder; do not log it.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}
impl fmt::Debug for Seek<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Seek([redacted])")
    }
}

macro_rules! text_enum {
    ($name:ident, $doc:literal, $($variant:ident => $text:literal),+ $(,)?) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name { $(#[doc = $text] $variant),+ }
        impl $name {
            /// Returns the source-owned wire spelling.
            #[must_use]
            pub const fn as_str(self) -> &'static str { match self { $(Self::$variant => $text),+ } }
            /// Rejects unknown or differently spelled values.
            pub fn parse(value: &str) -> Result<Self, QueryError> {
                match value { $($text => Ok(Self::$variant),)+ _ => Err(QueryError::Syntax) }
            }
        }
    }
}
text_enum!(Include, "Known additional response sections; validity is operation-specific.",
    Versions => "versions", Keywords => "keywords", Categories => "categories", Badges => "badges",
    Downloads => "downloads", DefaultVersion => "default_version", Full => "full",
    ReleaseTracks => "release_tracks", LinkedAccounts => "linked_accounts");
text_enum!(Sort, "Known source-owned sort spellings, checked per operation.",
    Alpha => "alpha", Crates => "crates", Alphabetical => "alphabetical", Relevance => "relevance",
    Downloads => "downloads", RecentDownloads => "recent-downloads", RecentUpdates => "recent-updates",
    New => "new", Date => "date", Semver => "semver");

/// Public GET operation families that admit query parameters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryOperation {
    /// Category collection.
    Categories,
    /// Keyword collection.
    Keywords,
    /// Crate search/list.
    Crates,
    /// Crate details.
    Crate,
    /// Crate aggregate downloads.
    Downloads,
    /// Crate versions.
    Versions,
    /// Crate reverse dependencies.
    ReverseDependencies,
    /// Individual version download counts.
    VersionDownloads,
    /// User details.
    User,
    /// GitHub trusted-publishing configurations.
    GithubConfigs,
    /// GitLab trusted-publishing configurations.
    GitlabConfigs,
}
impl QueryOperation {
    /// Whether the source advertises pagination parameters on this operation.
    #[must_use]
    pub const fn paginated(self) -> bool {
        matches!(
            self,
            Self::Categories
                | Self::Keywords
                | Self::Crates
                | Self::Versions
                | Self::ReverseDependencies
                | Self::GithubConfigs
                | Self::GitlabConfigs
        )
    }
    pub(super) fn sort(self, sort: Sort) -> bool {
        match self {
            Self::Categories | Self::Keywords => matches!(sort, Sort::Alpha | Sort::Crates),
            Self::Versions => matches!(sort, Sort::Date | Sort::Semver),
            Self::Crates => matches!(
                sort,
                Sort::Alphabetical
                    | Sort::Relevance
                    | Sort::Downloads
                    | Sort::RecentDownloads
                    | Sort::RecentUpdates
                    | Sort::New
            ),
            _ => false,
        }
    }
    pub(super) fn include(self, value: Include) -> bool {
        match self {
            Self::Crate => matches!(
                value,
                Include::Versions
                    | Include::Keywords
                    | Include::Categories
                    | Include::Badges
                    | Include::Downloads
                    | Include::DefaultVersion
                    | Include::Full
            ),
            Self::Downloads => value == Include::Versions,
            Self::Versions => value == Include::ReleaseTracks,
            Self::User => value == Include::LinkedAccounts,
            _ => false,
        }
    }
}

/// Nonempty, unique additional sections with no silently shadowed selection.
#[derive(Clone, Copy, Debug)]
pub struct IncludeSet<'a>(&'a [Include]);
impl<'a> IncludeSet<'a> {
    /// Checks duplicates and conflicts; operation membership is checked by `Query`.
    pub fn new(values: &'a [Include]) -> Result<Self, QueryError> {
        if values.is_empty() || values.len() > 9 {
            return Err(QueryError::Limit);
        }
        for (i, value) in values.iter().enumerate() {
            if values.get(..i).ok_or(QueryError::Limit)?.contains(value) {
                return Err(QueryError::Duplicate);
            }
        }
        if (values.contains(&Include::Full) && values.len() != 1)
            || (values.contains(&Include::Versions) && values.contains(&Include::DefaultVersion))
        {
            return Err(QueryError::Conflict);
        }
        Ok(Self(values))
    }
    /// Returns the exact selected sections.
    #[must_use]
    pub const fn values(self) -> &'a [Include] {
        self.0
    }
}
