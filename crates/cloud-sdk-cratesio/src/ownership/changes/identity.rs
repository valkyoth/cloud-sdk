use crate::identifiers::{IdentifierError, TeamLogin, UserLogin};

/// Explicit registry/GitHub namespaces. Unprefixed identities are resolved by
/// the server and may be rejected as ambiguous; they are never silently rewritten.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerSelector<'a> {
    /// Cargo-compatible unprefixed spelling; prefer RegistryUser when known.
    Unqualified(UserLogin<'a>),
    /// Registry username, not the user's linked GitHub login.
    RegistryUser(UserLogin<'a>),
    /// GitHub login; the server resolves linked account identity.
    GithubUser(UserLogin<'a>),
    /// Provider-qualified team.
    Team(TeamLogin<'a>),
}
impl<'a> OwnerSelector<'a> {
    /// Exact supported namespace grammar; rejects extra/missing separators.
    pub fn new(value: &'a str) -> Result<Self, IdentifierError> {
        if let Some(user) = value.strip_prefix("crates.io:") {
            UserLogin::new(user).map(Self::RegistryUser)
        } else if let Some(user) = value.strip_prefix("github:") {
            if user.contains(':') {
                TeamLogin::new(value).map(Self::Team)
            } else {
                UserLogin::new(user).map(Self::GithubUser)
            }
        } else {
            UserLogin::new(value).map(Self::Unqualified)
        }
    }
    pub(super) fn parts(self) -> (&'static str, &'a str) {
        match self {
            Self::Unqualified(u) => ("", u.as_str()),
            Self::RegistryUser(u) => ("crates.io:", u.as_str()),
            Self::GithubUser(u) => ("github:", u.as_str()),
            Self::Team(t) => ("", t.as_str()),
        }
    }
    pub(super) fn duplicate(self, other: Self) -> bool {
        match (self, other) {
            (Self::Team(a), Self::Team(b)) => a.as_str().eq_ignore_ascii_case(b.as_str()),
            (Self::Team(_), _) | (_, Self::Team(_)) => false,
            _ => same_user(self.parts().1, other.parts().1),
        }
    }
}
pub(super) fn same_user(a: &str, b: &str) -> bool {
    let canon = |b: u8| {
        if b == b'-' {
            b'_'
        } else {
            b.to_ascii_lowercase()
        }
    };
    a.bytes().map(canon).eq(b.bytes().map(canon))
}
