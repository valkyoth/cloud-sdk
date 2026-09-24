use super::{PersonalPermit, permit::Authority, request::Action};
use crate::{
    discovery::{DiscoveryError as Error, value::Builder},
    identifiers::NumericId,
    wire::JsonSuccess,
};
use cloud_sdk::incremental_json::IncrementalJsonProgress;

/// Checked acknowledgement, not a claim of email delivery or a verified identity.
#[derive(Debug, Eq, PartialEq)]
pub enum PersonalResponse {
    /// Provider returned an explicit true `ok` field.
    Acknowledged,
    /// Provider echoed the exact intended crate and acceptance state.
    Invitation {
        /// Bound crate identifier.
        crate_id: NumericId,
        /// False means an intentional decline, not a failed acceptance.
        accepted: bool,
    },
}
impl PersonalPermit<'_> {
    /// Decodes an already admitted response and clears its input. Callers using
    /// this lower-level boundary must associate it with the exact permitted
    /// exchange; it does not send a request or prove server-side authorization.
    pub fn decode_response(&self, success: JsonSuccess<'_>) -> Result<PersonalResponse, Error> {
        let mut builder = Builder::default();
        if success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        let root = builder.finish()?;
        use super::{PersonalOperation as O, schema_table as s};
        let schema = match self.operation() {
            O::Follow => s::FOLLOW_CRATE,
            O::Unfollow => s::UNFOLLOW_CRATE,
            O::HandleInvitation => s::HANDLE_CRATE_OWNER_INVITATION,
            O::AcceptInvitationToken => s::ACCEPT_CRATE_OWNER_INVITATION_WITH_TOKEN,
            O::Notifications => s::UPDATE_EMAIL_NOTIFICATIONS,
            O::ResendEmail => s::RESEND_EMAIL_VERIFICATION,
            O::UpdateUser => s::UPDATE_USER,
            O::ConfirmEmail => s::CONFIRM_USER_EMAIL,
        };
        crate::catalog::schema::validate_table(&root, schema, 0, s::NODES)?;
        let expected = match &self.0 {
            Authority::Api(request, _) => match request.0 {
                Action::Invitation(id, accepted) => Some((id, accepted)),
                _ => None,
            },
            Authority::Invitation(_, id) => Some((*id, true)),
            Authority::Email(_) => None,
        };
        if let Some((id, accepted)) = expected {
            let value = root.required("crate_owner_invitation")?;
            if value.required("crate_id")?.count(i32::MAX as u64)? != u64::from(id.get())
                || value.required("accepted")?.boolean()? != accepted
            {
                return Err(Error::Binding);
            }
            Ok(PersonalResponse::Invitation {
                crate_id: id,
                accepted,
            })
        } else if root.required("ok")?.boolean()? {
            Ok(PersonalResponse::Acknowledged)
        } else {
            Err(Error::Value)
        }
    }
}
