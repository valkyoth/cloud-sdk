//! Scoped owner-invitation reads and mutations. Invitation IDs are not crate IDs.
use super::schema_table as schema;
use crate::{
    credentials::ApiToken,
    discovery::{DiscoveryError as Error, DiscoveryValue, value::Builder},
    endpoint::ApiRequestTarget,
    identifiers::{CrateName, NumericId},
    query::{MAX_TARGET_BYTES, PerPage, Seek},
    wire::JsonSuccess,
};
use cloud_sdk::{
    Method, buffer::encode_snapshot_bounded, incremental_json::IncrementalJsonProgress,
};

/// Positive signed 64-bit invitation identifier returned by the registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvitationId(u64);
impl InvitationId {
    /// Rejects zero and values outside the provider's signed identifier range.
    pub fn new(id: u64) -> Result<Self, Error> {
        if id == 0 || i64::try_from(id).is_err() {
            return Err(Error::Value);
        }
        Ok(Self(id))
    }
}

/// One explicit listing authority. The provider does not combine both filters.
#[derive(Clone, Copy, Debug)]
pub enum Filter<'a> {
    /// List pending invitations for an owned crate (change-owners scope).
    Crate(CrateName<'a>),
    /// List the authenticated user's invitations (crate-owner-invitations scope).
    Invitee(NumericId),
}
/// Only the invitee can accept or decline; only a user owner can revoke.
#[derive(Clone, Copy, Debug)]
pub enum Decision {
    /// Become an owner.
    Accept,
    /// Decline the invitation.
    Decline,
    /// Revoke the outstanding invitation.
    Revoke,
}
impl Decision {
    fn state(self) -> &'static str {
        match self {
            Self::Accept => "accepted",
            Self::Decline => "declined",
            Self::Revoke => "revoked",
        }
    }
}
#[derive(Clone, Copy)]
enum Intent<'a> {
    List(Filter<'a>, PerPage, Option<Seek<'a>>),
    Update(InvitationId, Decision),
}
/// Single-attempt consent. Neither an empty filter nor numbered pagination is admitted.
pub struct InvitationPermit<'a> {
    intent: Intent<'a>,
    token: &'a ApiToken,
}
impl core::fmt::Debug for InvitationPermit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("InvitationPermit([redacted])")
    }
}
impl<'a> InvitationPermit<'a> {
    /// The one official registry origin to which this permit is bound.
    pub fn origin(&self) -> crate::credentials::CredentialOrigin {
        self.token.origin()
    }
    /// Authorizes one filtered page on the credential's official API origin.
    pub fn list(
        token: &'a ApiToken,
        filter: Filter<'a>,
        size: PerPage,
        seek: Option<Seek<'a>>,
    ) -> Self {
        Self {
            intent: Intent::List(filter, size, seek),
            token,
        }
    }
    /// Explicitly authorizes this one transition for this exact invitation.
    pub fn confirm_update(token: &'a ApiToken, id: InvitationId, decision: Decision) -> Self {
        Self {
            intent: Intent::Update(id, decision),
            token,
        }
    }
    /// Writes only the checked relative target, atomically.
    pub fn write_target<'b>(&self, output: &'b mut [u8]) -> Result<ApiRequestTarget<'b>, Error> {
        let n = encode_snapshot_bounded(
            self.intent,
            output,
            MAX_TARGET_BYTES,
            Error::Limit,
            |intent, e| {
                e.string("/api/v1/crate_owner_invitations")?;
                match intent {
                    Intent::Update(id, _) => {
                        e.byte(b'/')?;
                        e.u64(id.0)
                    }
                    Intent::List(filter, size, seek) => {
                        match filter {
                            Filter::Crate(name) => {
                                e.string("?crate_name=")?;
                                e.percent_encoded(name.as_str())?;
                            }
                            Filter::Invitee(id) => {
                                e.string("?invitee_id=")?;
                                e.u64(u64::from(id.get()))?;
                            }
                        }
                        e.string("&per_page=")?;
                        e.u64(u64::from(size.get()))?;
                        if let Some(cursor) = seek {
                            e.string("&seek=")?;
                            e.percent_encoded(cursor.as_str())?;
                        }
                        Ok(())
                    }
                }
            },
        )?;
        ApiRequestTarget::new(
            core::str::from_utf8(output.get(..n).ok_or(Error::Limit)?).map_err(|_| Error::Value)?,
        )
        .map_err(|_| Error::Value)
    }
    /// HTTP method and bounded JSON payload; never contains the token.
    pub fn wire_body(&self) -> (Method, &'static [u8]) {
        match self.intent {
            Intent::List(..) => (Method::Get, &[]),
            Intent::Update(_, d) => (
                Method::Patch,
                match d {
                    Decision::Accept => br#"{"crate_owner_invitation":{"state":"accepted"}}"#,
                    Decision::Decline => br#"{"crate_owner_invitation":{"state":"declined"}}"#,
                    Decision::Revoke => br#"{"crate_owner_invitation":{"state":"revoked"}}"#,
                },
            ),
        }
    }
    /// Decode only the admitted JSON from the associated authenticated exchange.
    /// Pagination links and user URLs remain inert metadata.
    pub fn decode_response(&self, success: JsonSuccess<'_>) -> Result<DiscoveryValue, Error> {
        let mut builder = Builder::default();
        if success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        self.check_tree(builder.finish()?)
    }

    pub(super) fn check_tree(&self, root: DiscoveryValue) -> Result<DiscoveryValue, Error> {
        let row = match self.intent {
            Intent::List(..) => schema::LIST_CRATE_OWNER_INVITATIONS,
            Intent::Update(..) => schema::UPDATE_CRATE_OWNER_INVITATION,
        };
        crate::catalog::schema::validate_table(&root, row, 0, schema::NODES)?;
        match self.intent {
            Intent::Update(expected, decision) => {
                let item = root.required("crate_owner_invitation")?;
                if invitation_id(item)? != expected
                    || !item
                        .required("state")?
                        .with_text(|s| s == decision.state())?
                {
                    return Err(Error::Binding);
                }
            }
            Intent::List(filter, size, _) => {
                let items = root.required("crate_owner_invitations")?.array()?;
                if items.len() > usize::try_from(size.get()).map_err(|_| Error::Limit)? {
                    return Err(Error::Limit);
                }
                let users = root.required("users")?.array()?;
                if users.len() > items.len().checked_mul(2).ok_or(Error::Limit)? {
                    return Err(Error::Limit);
                }
                let mut identities = alloc::vec::Vec::new();
                identities
                    .try_reserve_exact(items.len())
                    .map_err(|_| Error::Allocation)?;
                for item in items {
                    let id = invitation_id(item)?;
                    if identities.iter().any(|(previous, _, _)| *previous == id) {
                        return Err(Error::Binding);
                    }
                    let invitee = positive_id(item, "invitee_id")?;
                    identities.push((id, invitee, positive_id(item, "inviter_id")?));
                    let matched = match filter {
                        Filter::Invitee(id) => invitee == id,
                        Filter::Crate(name) => item
                            .required("crate_name")?
                            .with_text(|s| same_crate(s, name.as_str()))?,
                    };
                    if !matched || !item.required("state")?.with_text(|s| s == "pending")? {
                        return Err(Error::Binding);
                    }
                }
                let mut seen = alloc::vec::Vec::new();
                seen.try_reserve_exact(users.len())
                    .map_err(|_| Error::Allocation)?;
                for user in users {
                    let user_id = positive_id(user, "id")?;
                    if seen.contains(&user_id)
                        || !identities
                            .iter()
                            .any(|(_, a, b)| user_id == *a || user_id == *b)
                    {
                        return Err(Error::Binding);
                    }
                    seen.push(user_id);
                }
            }
        }
        Ok(root)
    }
}
fn same_crate(a: &str, b: &str) -> bool {
    fn canonical(b: u8) -> u8 {
        if b == b'-' {
            b'_'
        } else {
            b.to_ascii_lowercase()
        }
    }
    a.bytes().map(canonical).eq(b.bytes().map(canonical))
}
fn invitation_id(item: &DiscoveryValue) -> Result<InvitationId, Error> {
    for name in ["crate_id", "inviter_id", "invitee_id"] {
        positive_id(item, name)?;
    }
    InvitationId::new(item.required("id")?.count(i64::MAX as u64)?)
}
fn positive_id(item: &DiscoveryValue, name: &str) -> Result<NumericId, Error> {
    NumericId::new(item.required(name)?.count(i32::MAX as u64)?).map_err(|_| Error::Value)
}

#[cfg(any(feature = "blocking", feature = "async"))]
impl crate::client::prepared::Permit for InvitationPermit<'_> {
    type Response = DiscoveryValue;
    fn origin(&self) -> crate::credentials::CredentialOrigin {
        self.token.origin()
    }
    fn prepare<'a, T: cloud_sdk::transport::BoundTransport + ?Sized>(
        &self,
        executor: &T,
        target: &'a mut [u8],
        credential: &'a mut cloud_sdk_sanitization::SecretBuffer<'_>,
        _body: &'a mut [u8],
        _now: u64,
    ) -> Result<crate::client::prepared::Prepared<'a>, Error> {
        let (method, body) = self.wire_body();
        crate::client::prepared::api(
            self.token,
            method,
            self.write_target(target)?,
            executor,
            credential,
            body,
        )
    }
    fn decode(self, response: JsonSuccess<'_>, _now: u64) -> Result<DiscoveryValue, Error> {
        self.decode_response(response)
    }
}
