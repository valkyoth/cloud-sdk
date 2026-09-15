use crate::{
    endpoint::ApiRequestTarget,
    identifiers::{CrateName, NumericId, TeamLogin, UserLogin},
    query::{
        ApiPath, FixedSegment as F, Parameter, PathSegment as P, Query, QueryError, QueryOperation,
    },
};
use cloud_sdk::operation::{
    OperationId, OperationIdError, OperationMetadata, OperationMetadataError,
};

/// Source-locked anonymous identity or ownership GET.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountOperation {
    /// Public user by login.
    User,
    /// Aggregate owned-crate download count by numeric user ID.
    UserStats,
    /// Public provider-qualified team.
    Team,
    /// Combined user and team owners.
    Owners,
    /// Individual owners only.
    UserOwners,
    /// Team owners only.
    TeamOwners,
}
impl AccountOperation {
    /// Exact upstream operation identifier.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::User => "find_user",
            Self::UserStats => "get_user_stats",
            Self::Team => "find_team",
            Self::Owners => "list_owners",
            Self::UserOwners => "get_user_owners",
            Self::TeamOwners => "get_team_owners",
        }
    }
    /// Checked source identifier.
    pub fn operation_id(self) -> Result<OperationId, OperationIdError> {
        OperationId::new(self.operation_name())
    }
    /// Read-only metadata with no implicit retry.
    pub const fn metadata(self) -> Result<OperationMetadata, OperationMetadataError> {
        crate::discovery::DiscoveryOperation::Summary.metadata()
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Selector<'a> {
    User(UserLogin<'a>, Query<'a>),
    Stats(NumericId),
    Team(TeamLogin<'a>),
    Owners(CrateName<'a>, OwnerFilter),
}
#[derive(Clone, Copy, Debug)]
pub(super) enum OwnerFilter {
    All,
    Users,
    Teams,
}

/// Immutable operation and validated selectors; no credential or permit input.
#[derive(Clone, Copy, Debug)]
pub struct AccountRequest<'a>(pub(super) Selector<'a>);
impl<'a> AccountRequest<'a> {
    /// Public user, optionally including public linked accounts.
    pub fn user(login: UserLogin<'a>, parameters: &'a [Parameter<'a>]) -> Result<Self, QueryError> {
        Ok(Self(Selector::User(
            login,
            Query::new(QueryOperation::User, parameters)?,
        )))
    }
    /// Statistics do not establish whether this ID belongs to an existing user.
    pub const fn user_stats(id: NumericId) -> Self {
        Self(Selector::Stats(id))
    }
    /// Team lookup preserves exact case and percent-encodes the qualified login.
    pub const fn team(login: TeamLogin<'a>) -> Self {
        Self(Selector::Team(login))
    }
    /// All user and team owners, without granting authorization.
    pub const fn owners(name: CrateName<'a>) -> Self {
        Self(Selector::Owners(name, OwnerFilter::All))
    }
    /// Individual owners only.
    pub const fn user_owners(name: CrateName<'a>) -> Self {
        Self(Selector::Owners(name, OwnerFilter::Users))
    }
    /// Team owners only.
    pub const fn team_owners(name: CrateName<'a>) -> Self {
        Self(Selector::Owners(name, OwnerFilter::Teams))
    }
    /// Immutable operation identity.
    pub const fn operation(self) -> AccountOperation {
        match self.0 {
            Selector::User(..) => AccountOperation::User,
            Selector::Stats(_) => AccountOperation::UserStats,
            Selector::Team(_) => AccountOperation::Team,
            Selector::Owners(_, OwnerFilter::All) => AccountOperation::Owners,
            Selector::Owners(_, OwnerFilter::Users) => AccountOperation::UserOwners,
            Selector::Owners(_, OwnerFilter::Teams) => AccountOperation::TeamOwners,
        }
    }
    /// Writes the complete target atomically; a short buffer stays unchanged.
    pub fn write_target(self, output: &mut [u8]) -> Result<ApiRequestTarget<'_>, QueryError> {
        let (parts, len) = match self.0 {
            Selector::User(login, _) => {
                ([P::Fixed(F::Users), P::User(login), P::Fixed(F::Stats)], 2)
            }
            Selector::Stats(id) => ([P::Fixed(F::Users), P::Id(id), P::Fixed(F::Stats)], 3),
            Selector::Team(login) => ([P::Fixed(F::Teams), P::Team(login), P::Fixed(F::Stats)], 2),
            Selector::Owners(name, filter) => (
                [
                    P::Fixed(F::Crates),
                    P::Crate(name),
                    P::Fixed(match filter {
                        OwnerFilter::All => F::Owners,
                        OwnerFilter::Users => F::OwnerUser,
                        OwnerFilter::Teams => F::OwnerTeam,
                    }),
                ],
                3,
            ),
        };
        let path = ApiPath::new(parts.get(..len).ok_or(QueryError::Output)?)?;
        match self.0 {
            Selector::User(_, query) => query.write_target(path, output),
            _ => path.write(output),
        }
    }
}
