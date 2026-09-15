use crate::{
    discovery::{DiscoveryError, DiscoveryValue},
    identifiers::NumericId,
};
use alloc::vec::Vec;

/// Disjoint provider identity namespaces, not authorization levels.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountKind {
    /// Registry user.
    User,
    /// GitHub organization team.
    Team,
}
/// Validated public identity with protected, scoped metadata access.
#[derive(Debug)]
pub struct AccountRecord {
    pub(super) id: NumericId,
    pub(super) kind: AccountKind,
    pub(super) fields: DiscoveryValue,
}
impl AccountRecord {
    /// Positive ID, meaningful only inside this record's kind namespace.
    pub const fn id(&self) -> NumericId {
        self.id
    }
    /// User or team namespace.
    pub const fn kind(&self) -> AccountKind {
        self.kind
    }
    /// Scoped access to validated login text.
    pub fn with_login<R>(&self, inspect: impl FnOnce(&str) -> R) -> Result<R, DiscoveryError> {
        self.fields.required("login")?.with_text(inspect)
    }
    /// Nullable and unknown metadata; URLs are inert, not fetch capabilities.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.fields
    }
}
/// Public user response, preserving omitted versus explicitly empty links.
#[derive(Debug)]
pub struct PublicUser {
    pub(super) account: AccountRecord,
    pub(super) linked: Option<DiscoveryValue>,
}
impl PublicUser {
    /// Validated registry identity, not a linked GitHub identity.
    pub const fn account(&self) -> &AccountRecord {
        &self.account
    }
    /// Public linked accounts, present only when explicitly requested.
    pub fn linked_accounts(&self) -> Result<Option<&[DiscoveryValue]>, DiscoveryError> {
        self.linked.as_ref().map(DiscoveryValue::array).transpose()
    }
}
/// Bounded owner list. Empty is valid; IDs may overlap across kinds.
#[derive(Debug)]
pub struct OwnerList(pub(super) Vec<AccountRecord>);
impl OwnerList {
    /// Complete admitted list; no silent truncation or authority inference.
    pub fn items(&self) -> &[AccountRecord] {
        &self.0
    }
}
/// Checked public response; no variant grants mutation authority.
#[derive(Debug)]
pub enum AccountResponse {
    /// Registry user with optional linked accounts.
    User(PublicUser),
    /// Count alone does not prove user existence or ownership identity.
    UserStats {
        /// Source-bounded aggregate count.
        total_downloads: u64,
    },
    /// Team metadata.
    Team(AccountRecord),
    /// Requested combined or filtered ownership snapshot.
    Owners(OwnerList),
}
