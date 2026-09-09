use super::{MAX_QUERY_PARAMETERS, MAX_TARGET_BYTES, QueryError, QueryOperation};
use crate::{endpoint::ApiRequestTarget, identifiers::*};
use cloud_sdk::buffer::{SnapshotEncoder, encode_snapshot_bounded};

macro_rules! fixed_segments {
    ($($name:ident => $text:literal),+ $(,)?) => {
        /// Fixed public route segments, never interpolated caller text.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum FixedSegment { $(#[doc = $text] $name),+ }
        impl FixedSegment {
            /// Exact route spelling.
            #[must_use]
            pub const fn as_str(self) -> &'static str { match self { $(Self::$name => $text),+ } }
        }
    }
}
fixed_segments!(Categories => "categories", Keywords => "keywords", Crates => "crates",
    Users => "users", Teams => "teams", Versions => "versions", Downloads => "downloads",
    Download => "download", ReverseDependencies => "reverse_dependencies", Owners => "owners",
    OwnerUser => "owner_user", OwnerTeam => "owner_team", Follow => "follow", Readme => "readme",
    Authors => "authors", Dependencies => "dependencies", Yank => "yank", Unyank => "unyank",
    New => "new", CategorySlugs => "category_slugs", SiteMetadata => "site_metadata",
    Summary => "summary", TrustedPublishing => "trusted_publishing", GithubConfigs => "github_configs",
    GitlabConfigs => "gitlab_configs", Tokens => "tokens", Current => "current", Me => "me",
    CrateOwnerInvitations => "crate_owner_invitations", EmailNotifications => "email_notifications",
    Stats => "stats", Resend => "resend");

/// One validated public path component. Secret path tokens use the credential API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathSegment<'a> {
    /// Fixed route text.
    Fixed(FixedSegment),
    /// Crate name.
    Crate(CrateName<'a>),
    /// Exact version.
    Version(Version<'a>),
    /// Category hierarchy.
    Category(CategorySlug<'a>),
    /// Keyword.
    Keyword(Keyword<'a>),
    /// User login.
    User(UserLogin<'a>),
    /// Provider-qualified team login.
    Team(TeamLogin<'a>),
    /// Positive API identifier.
    Id(NumericId),
}
impl PathSegment<'_> {
    fn encode(self, encoder: &mut SnapshotEncoder<'_, QueryError>) -> Result<(), QueryError> {
        match self {
            Self::Fixed(v) => encoder.string(v.as_str()),
            Self::Crate(v) => encoder.percent_encoded(v.as_str()),
            Self::Version(v) => encoder.percent_encoded(v.as_str()),
            Self::Category(v) => encoder.percent_encoded(v.as_str()),
            Self::Keyword(v) => encoder.percent_encoded(v.as_str()),
            Self::User(v) => encoder.percent_encoded(v.as_str()),
            Self::Team(v) => encoder.percent_encoded(v.as_str()),
            Self::Id(v) => encoder.u64(u64::from(v.get())),
        }
    }
}

/// Bounded, typed API path assembly; operation authorization remains separate.
#[derive(Clone, Copy, Debug)]
pub struct ApiPath<'a>(&'a [PathSegment<'a>]);
impl<'a> ApiPath<'a> {
    /// Constructs an API path from fixed and validated path segments.
    pub fn new(parts: &'a [PathSegment<'a>]) -> Result<Self, QueryError> {
        if parts.is_empty() || parts.len() > MAX_QUERY_PARAMETERS {
            return Err(QueryError::Limit);
        }
        if !matches!(parts.first(), Some(PathSegment::Fixed(_))) {
            return Err(QueryError::Syntax);
        }
        Ok(Self(parts))
    }
    /// Encodes a complete path atomically, preserving output on capacity failure.
    pub fn write(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError> {
        let len = encode_snapshot_bounded(
            self,
            output,
            MAX_TARGET_BYTES,
            QueryError::Output,
            |path, encoder| path.encode(encoder),
        )?;
        let text = core::str::from_utf8(output.get(..len).ok_or(QueryError::Output)?)
            .map_err(|_| QueryError::Output)?;
        ApiRequestTarget::new(text).map_err(|_| QueryError::Syntax)
    }
    pub(super) fn encode(
        self,
        encoder: &mut SnapshotEncoder<'_, QueryError>,
    ) -> Result<(), QueryError> {
        encoder.string("/api/v1")?;
        for part in self.0 {
            encoder.byte(b'/')?;
            part.encode(encoder)?;
        }
        Ok(())
    }
    pub(crate) fn supports(self, operation: QueryOperation) -> bool {
        use FixedSegment as F;
        use PathSegment as P;
        use QueryOperation as Q;
        matches!(
            (operation, self.0),
            (Q::Categories, [P::Fixed(F::Categories)])
                | (Q::Keywords, [P::Fixed(F::Keywords)])
                | (Q::Crates, [P::Fixed(F::Crates)])
                | (Q::Crate, [P::Fixed(F::Crates), P::Crate(_)])
                | (
                    Q::Downloads,
                    [P::Fixed(F::Crates), P::Crate(_), P::Fixed(F::Downloads)]
                )
                | (
                    Q::Versions,
                    [P::Fixed(F::Crates), P::Crate(_), P::Fixed(F::Versions)]
                )
                | (
                    Q::ReverseDependencies,
                    [
                        P::Fixed(F::Crates),
                        P::Crate(_),
                        P::Fixed(F::ReverseDependencies)
                    ]
                )
                | (
                    Q::VersionDownloads,
                    [
                        P::Fixed(F::Crates),
                        P::Crate(_),
                        P::Version(_),
                        P::Fixed(F::Downloads)
                    ]
                )
                | (Q::User, [P::Fixed(F::Users), P::User(_)])
                | (
                    Q::GithubConfigs,
                    [P::Fixed(F::TrustedPublishing), P::Fixed(F::GithubConfigs)]
                )
                | (
                    Q::GitlabConfigs,
                    [P::Fixed(F::TrustedPublishing), P::Fixed(F::GitlabConfigs)]
                )
        )
    }
}
