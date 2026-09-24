use super::{PersonalOperation, PersonalRequest};
use crate::credentials::{ApiToken, EmailConfirmationToken, OwnerInvitationToken};
use crate::identifiers::NumericId;

pub(super) enum Authority<'a> {
    Api(PersonalRequest<'a>, &'a ApiToken),
    Email(EmailConfirmationToken),
    Invitation(OwnerInvitationToken, NumericId),
}
/// Non-cloneable single-call intent bound to the exact request and credential.
/// This is caller authorization, not proof of upstream permission or token identity.
/// Creating another permit is an explicit new attempt, never an automatic retry.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::accounts::personal::PersonalPermit;
/// fn duplicate(permit: PersonalPermit<'_>) { let _ = permit.clone(); }
/// ```
///
/// ```compile_fail
/// use cloud_sdk_cratesio::{accounts::personal::PersonalRequest,
///     credentials::ApiToken, identifiers::NumericId};
/// fn rotate_while_authorized(token: &mut ApiToken, id: NumericId) {
///     let permit = PersonalRequest::resend_email(id).confirm(token);
///     token.clear();
///     let _ = permit.operation();
/// }
/// ```
pub struct PersonalPermit<'a>(pub(super) Authority<'a>);
impl<'a> PersonalRequest<'a> {
    /// Explicitly confirms this exact action, including destructive unfollow or
    /// invitation decline. The request is moved, not passed separately at execution.
    pub fn confirm(self, credential: &'a ApiToken) -> PersonalPermit<'a> {
        PersonalPermit(Authority::Api(self, credential))
    }
}
impl PersonalPermit<'_> {
    /// Immutable official destination selected by the bound credential.
    pub fn origin(&self) -> crate::credentials::CredentialOrigin {
        match &self.0 {
            Authority::Api(_, token) => token.origin(),
            Authority::Email(token) => token.origin(),
            Authority::Invitation(token, _) => token.origin(),
        }
    }
    /// Consumes the email token for one confirmation attempt. The server selects
    /// the address; no client-side identity or expiry claim is made.
    pub fn confirm_email(token: EmailConfirmationToken) -> Self {
        Self(Authority::Email(token))
    }
    /// Consumes the invitation token for one attempt and checks the returned crate
    /// ID. The token itself selects the upstream crate: a response mismatch cannot
    /// undo a mutation, so obtain token and expected ID from one trusted invitation.
    pub fn accept_invitation_token(token: OwnerInvitationToken, expected_crate: NumericId) -> Self {
        Self(Authority::Invitation(token, expected_crate))
    }
    /// Public operation identity, without target or credential data.
    pub fn operation(&self) -> PersonalOperation {
        match &self.0 {
            Authority::Api(request, _) => request.operation(),
            Authority::Email(_) => PersonalOperation::ConfirmEmail,
            Authority::Invitation(..) => PersonalOperation::AcceptInvitationToken,
        }
    }
}
impl core::fmt::Debug for PersonalPermit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PersonalPermit([redacted])")
    }
}
