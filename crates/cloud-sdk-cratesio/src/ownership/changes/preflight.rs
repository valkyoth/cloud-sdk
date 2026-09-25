use super::{
    OwnerChangeError as Error, OwnerChangePermit, OwnerChangeRequest, OwnerSelector,
    identity::same_user,
};
use crate::{
    credentials::ApiToken,
    identifiers::{CrateName, Owner, UserLogin},
};

/// Explicit local policy; Allow does not override last-individual-owner checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelfRemoval {
    /// Refuse removing the caller-supplied acting registry user.
    Deny,
    /// Explicitly allow self removal when another individual remains.
    Allow,
}
/// Caller-asserted complete active ownership snapshot, never pending invitations.
/// No endpoint response echoes a crate identity, so the caller must associate
/// these identities with the trusted list exchange for this exact crate. This
/// is a local preflight, not authenticated provenance, authority or a CAS lock.
#[derive(Debug)]
pub struct RemovalSnapshot<'a> {
    name: CrateName<'a>,
    owners: &'a [Owner<'a>],
}
impl<'a> RemovalSnapshot<'a> {
    /// User entries are registry usernames, not linked GitHub logins. Rejects
    /// duplicate identities and oversized/empty lists; no silent truncation.
    pub fn from_complete_active_list(
        name: CrateName<'a>,
        owners: &'a [Owner<'a>],
    ) -> Result<Self, Error> {
        if owners.is_empty() || owners.len() > 256 {
            return Err(Error::Limit);
        }
        for (index, owner) in owners.iter().enumerate() {
            if owners
                .get(..index)
                .ok_or(Error::Limit)?
                .iter()
                .any(|p| equivalent(*owner, *p))
            {
                return Err(Error::Value);
            }
        }
        Ok(Self { name, owners })
    }
    fn check(
        &self,
        request: &OwnerChangeRequest<'_>,
        actor: UserLogin<'_>,
        self_removal: SelfRemoval,
    ) -> Result<(), Error> {
        let canon = |b: u8| {
            if b == b'_' {
                b'-'
            } else {
                b.to_ascii_lowercase()
            }
        };
        if !self
            .name
            .as_str()
            .bytes()
            .map(canon)
            .eq(request.name.as_str().bytes().map(canon))
        {
            return Err(Error::Binding);
        }
        if !self
            .owners
            .iter()
            .any(|o| equivalent(*o, Owner::User(actor)))
        {
            return Err(Error::Binding);
        }
        for selected in request.owners {
            let selected = local_identity(*selected)?;
            if !self.owners.iter().any(|o| equivalent(*o, selected)) {
                return Err(Error::Binding);
            }
            if self_removal == SelfRemoval::Deny && equivalent(selected, Owner::User(actor)) {
                return Err(Error::Binding);
            }
        }
        let remaining =
            self.owners
                .iter()
                .filter(|owner| matches!(owner, Owner::User(_)))
                .any(|owner| {
                    !request.owners.iter().any(|selected| {
                        local_identity(*selected).is_ok_and(|s| equivalent(s, *owner))
                    })
                });
        if !remaining {
            return Err(Error::Binding);
        }
        Ok(())
    }
}
impl<'a> OwnerChangeRequest<'a> {
    /// Optional fail-closed preflight plus explicit destructive consent. Refuses
    /// unqualified/GitHub user selectors: a registry snapshot cannot resolve them.
    /// A stale or fabricated snapshot cannot replace upstream checks.
    pub fn confirm_removal_after_preflight(
        self,
        credential: &'a ApiToken,
        snapshot: &RemovalSnapshot<'_>,
        actor: UserLogin<'_>,
        self_removal: SelfRemoval,
    ) -> Result<OwnerChangePermit<'a>, Error> {
        snapshot.check(&self, actor, self_removal)?;
        self.confirm_removal(credential)
    }
}
fn local_identity(selector: OwnerSelector<'_>) -> Result<Owner<'_>, Error> {
    match selector {
        OwnerSelector::RegistryUser(user) => Ok(Owner::User(user)),
        OwnerSelector::Team(team) => Ok(Owner::Team(team)),
        _ => Err(Error::Binding),
    }
}
fn equivalent(left: Owner<'_>, right: Owner<'_>) -> bool {
    match (left, right) {
        (Owner::User(a), Owner::User(b)) => same_user(a.as_str(), b.as_str()),
        (Owner::Team(a), Owner::Team(b)) => a.as_str().eq_ignore_ascii_case(b.as_str()),
        _ => false,
    }
}
