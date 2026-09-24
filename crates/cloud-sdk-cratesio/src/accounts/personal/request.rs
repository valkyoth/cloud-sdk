use super::{MAX_NOTIFICATION_UPDATES, MAX_PERSONAL_BODY_BYTES};
use crate::{
    discovery::DiscoveryError as Error,
    endpoint::ApiRequestTarget,
    identifiers::{CrateName, NumericId},
    query::{ApiPath, FixedSegment as F, PathSegment as P, QueryError},
};
use cloud_sdk::{
    Method,
    buffer::{SnapshotEncoder, encode_snapshot_bounded},
};

/// A bounded email input. The provider validates deliverability and full address
/// syntax. Borrowed source storage and any caller-created copies remain owned
/// by the caller; no address is included in Debug.
#[derive(Clone, Copy)]
pub struct EmailAddress<'a>(&'a str);
impl<'a> EmailAddress<'a> {
    /// Admits nonempty, trimmed text without whitespace or control characters.
    pub fn new(value: &'a str) -> Result<Self, Error> {
        if value.is_empty()
            || value.len() > 254
            || value.chars().any(char::is_whitespace)
            || value.chars().any(char::is_control)
        {
            return Err(Error::Value);
        }
        let Some((local, domain)) = value.split_once('@') else {
            return Err(Error::Value);
        };
        if local.is_empty() || domain.is_empty() || domain.contains('@') {
            return Err(Error::Value);
        }
        Ok(Self(value))
    }
}
impl core::fmt::Debug for EmailAddress<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("EmailAddress([redacted])")
    }
}

/// One crate-specific setting in the deprecated upstream notification API.
#[derive(Clone, Copy, Debug)]
pub struct NotificationUpdate {
    /// Crate identifier, not an owner or invitation identifier.
    pub crate_id: NumericId,
    /// Requested value.
    pub enabled: bool,
}

/// Source-locked personal operation identity. No variant enables automatic retry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersonalOperation {
    /// Follow a crate for the authenticated user.
    Follow,
    /// Remove a follow for the authenticated user.
    Unfollow,
    /// Accept or decline an invitation using an API token.
    HandleInvitation,
    /// Accept an invitation using the token delivered by email.
    AcceptInvitationToken,
    /// Update the deprecated crate notification setting.
    Notifications,
    /// Regenerate and send email verification.
    ResendEmail,
    /// Change one user setting.
    UpdateUser,
    /// Confirm the address selected by an email token.
    ConfirmEmail,
}
impl PersonalOperation {
    /// Exact public OpenAPI operation name.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Follow => "follow_crate",
            Self::Unfollow => "unfollow_crate",
            Self::HandleInvitation => "handle_crate_owner_invitation",
            Self::AcceptInvitationToken => "accept_crate_owner_invitation_with_token",
            Self::Notifications => "update_email_notifications",
            Self::ResendEmail => "resend_email_verification",
            Self::UpdateUser => "update_user",
            Self::ConfirmEmail => "confirm_user_email",
        }
    }
    /// Follow/unfollow converge on a desired state. This is not retry authority;
    /// concurrent user intent can change between attempts. All others are conservative.
    pub const fn is_state_idempotent(self) -> bool {
        matches!(self, Self::Follow | Self::Unfollow)
    }
    /// Exact request verb.
    pub const fn method(self) -> Method {
        if matches!(self, Self::Unfollow) {
            Method::Delete
        } else {
            Method::Put
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Action<'a> {
    Follow(CrateName<'a>, bool),
    Invitation(NumericId, bool),
    Notifications(&'a [NotificationUpdate]),
    Resend(NumericId),
    Email(NumericId, EmailAddress<'a>),
    PublishNotifications(NumericId, bool),
}
/// Immutable request intent, not permission. User IDs are checked by the server
/// against the API token. A request cannot change after its permit is issued.
pub struct PersonalRequest<'a>(pub(super) Action<'a>);
impl core::fmt::Debug for PersonalRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PersonalRequest([redacted])")
    }
}
impl<'a> PersonalRequest<'a> {
    /// Follow this exact crate.
    pub const fn follow(name: CrateName<'a>) -> Self {
        Self(Action::Follow(name, true))
    }
    /// Unfollow this exact crate; requires a fresh explicit permit.
    pub const fn unfollow(name: CrateName<'a>) -> Self {
        Self(Action::Follow(name, false))
    }
    /// Accept this invitation. Path and body always use the same crate ID.
    pub const fn accept_invitation(crate_id: NumericId) -> Self {
        Self(Action::Invitation(crate_id, true))
    }
    /// Decline this invitation. This consumes the pending invitation upstream.
    pub const fn decline_invitation(crate_id: NumericId) -> Self {
        Self(Action::Invitation(crate_id, false))
    }
    /// Regenerate verification for the token's user. The server rejects other users.
    pub const fn resend_email(user: NumericId) -> Self {
        Self(Action::Resend(user))
    }
    /// Change only the email, avoiding upstream partial application of a combined
    /// email/notification update. Success does not mean delivery or verification.
    pub const fn update_email(user: NumericId, email: EmailAddress<'a>) -> Self {
        Self(Action::Email(user, email))
    }
    /// Change only the publish-notification flag.
    pub const fn publish_notifications(user: NumericId, enabled: bool) -> Self {
        Self(Action::PublishNotifications(user, enabled))
    }
    /// Deprecated upstream compatibility: this setting never became a complete
    /// notification service. Rejects empty, oversized or duplicate crate batches.
    pub fn legacy_email_notifications(updates: &'a [NotificationUpdate]) -> Result<Self, Error> {
        if updates.is_empty() || updates.len() > MAX_NOTIFICATION_UPDATES {
            return Err(Error::Limit);
        }
        for (i, item) in updates.iter().enumerate() {
            if updates
                .get(..i)
                .ok_or(Error::Limit)?
                .iter()
                .any(|p| p.crate_id == item.crate_id)
            {
                return Err(Error::Value);
            }
        }
        Ok(Self(Action::Notifications(updates)))
    }
    /// Exact operation identity.
    pub const fn operation(&self) -> PersonalOperation {
        match self.0 {
            Action::Follow(_, true) => PersonalOperation::Follow,
            Action::Follow(_, false) => PersonalOperation::Unfollow,
            Action::Invitation(..) => PersonalOperation::HandleInvitation,
            Action::Notifications(_) => PersonalOperation::Notifications,
            Action::Resend(_) => PersonalOperation::ResendEmail,
            Action::Email(..) | Action::PublishNotifications(..) => PersonalOperation::UpdateUser,
        }
    }
    /// Atomic non-secret target encoding. Token-bearing targets are never exposed here.
    pub fn write_target<'b>(
        &self,
        output: &'b mut [u8],
    ) -> Result<ApiRequestTarget<'b>, QueryError> {
        let (parts, len) = match self.0 {
            Action::Follow(name, _) => (
                [P::Fixed(F::Crates), P::Crate(name), P::Fixed(F::Follow)],
                3,
            ),
            Action::Invitation(id, _) => (
                [
                    P::Fixed(F::Me),
                    P::Fixed(F::CrateOwnerInvitations),
                    P::Id(id),
                ],
                3,
            ),
            Action::Notifications(_) => (
                [
                    P::Fixed(F::Me),
                    P::Fixed(F::EmailNotifications),
                    P::Fixed(F::Me),
                ],
                2,
            ),
            Action::Resend(id) => ([P::Fixed(F::Users), P::Id(id), P::Fixed(F::Resend)], 3),
            Action::Email(id, _) | Action::PublishNotifications(id, _) => {
                ([P::Fixed(F::Users), P::Id(id), P::Fixed(F::Me)], 2)
            }
        };
        ApiPath::new(parts.get(..len).ok_or(QueryError::Output)?)?.write(output)
    }
    /// Encodes JSON into cleanup-owned caller scratch and exposes it only for
    /// the callback. The entire scratch buffer is cleared on every exit. Callers
    /// must not retain unprotected copies of an email address from this callback.
    pub fn with_json_body<R>(
        &self,
        output: &mut [u8],
        inspect: impl FnOnce(&[u8]) -> R,
    ) -> Result<R, Error> {
        let mut output = cloud_sdk_sanitization::SecretBuffer::new(output);
        cloud_sdk_sanitization::sanitize_bytes(output.as_mut_slice());
        let length = self.body(output.as_mut_slice())?;
        Ok(inspect(
            output.as_slice().get(..length).ok_or(Error::Limit)?,
        ))
    }
    pub(super) fn body(&self, output: &mut [u8]) -> Result<usize, Error> {
        encode_snapshot_bounded(
            self.0,
            output,
            MAX_PERSONAL_BODY_BYTES,
            Error::Limit,
            encode,
        )
    }
}
fn boolean(e: &mut SnapshotEncoder<'_, Error>, value: bool) -> Result<(), Error> {
    e.string(if value { "true" } else { "false" })
}
fn encode(action: Action<'_>, e: &mut SnapshotEncoder<'_, Error>) -> Result<(), Error> {
    match action {
        Action::Follow(..) | Action::Resend(_) => Ok(()),
        Action::Invitation(id, accepted) => {
            e.string("{\"crate_owner_invite\":{\"crate_id\":")?;
            e.u64(u64::from(id.get()))?;
            e.string(",\"accepted\":")?;
            boolean(e, accepted)?;
            e.string("}}")
        }
        Action::Email(_, email) => {
            e.string("{\"user\":{\"email\":")?;
            e.json_string(email.0)?;
            e.string("}}")
        }
        Action::PublishNotifications(_, enabled) => {
            e.string("{\"user\":{\"publish_notifications\":")?;
            boolean(e, enabled)?;
            e.string("}}")
        }
        Action::Notifications(updates) => {
            e.byte(b'[')?;
            for (i, item) in updates.iter().enumerate() {
                if i != 0 {
                    e.byte(b',')?;
                }
                e.string("{\"id\":")?;
                e.u64(u64::from(item.crate_id.get()))?;
                e.string(",\"email_notifications\":")?;
                boolean(e, item.enabled)?;
                e.byte(b'}')?;
            }
            e.byte(b']')
        }
    }
}
