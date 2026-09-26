use super::{PersonalPermit, PersonalResponse, permit::Authority};
use crate::{
    client::prepared::{self, Permit, Prepared},
    credentials::{CredentialContext, CredentialOrigin},
    discovery::DiscoveryError as Error,
    wire::JsonSuccess,
};
use cloud_sdk::transport::BoundTransport;
use cloud_sdk_sanitization::SecretBuffer;

impl Permit for PersonalPermit<'_> {
    type Response = PersonalResponse;
    fn origin(&self) -> CredentialOrigin {
        self.origin()
    }
    fn prepare<'a, T: BoundTransport + ?Sized>(
        &self,
        executor: &T,
        target: &'a mut [u8],
        credential: &'a mut SecretBuffer<'_>,
        body: &'a mut [u8],
        _now: u64,
    ) -> Result<Prepared<'a>, Error> {
        let (request, token) = match &self.0 {
            Authority::Api(request, token) => (request, token),
            Authority::Email(token) => {
                let material = token
                    .stage_for_adapter(
                        &CredentialContext::confirm_email(token.origin()),
                        executor,
                        credential,
                    )
                    .map_err(|_| Error::Binding)?;
                return Ok(Prepared {
                    material,
                    body: &[],
                });
            }
            Authority::Invitation(token, _) => {
                let material = token
                    .stage_for_adapter(
                        &CredentialContext::accept_invitation(token.origin()),
                        executor,
                        credential,
                    )
                    .map_err(|_| Error::Binding)?;
                return Ok(Prepared {
                    material,
                    body: &[],
                });
            }
        };
        let length = request.body(body)?;
        let body = body.get(..length).ok_or(Error::Limit)?;
        let target = request.write_target(target).map_err(|_| Error::Binding)?;
        prepared::api(
            token,
            request.operation().method(),
            target,
            executor,
            credential,
            body,
        )
    }
    fn decode(self, response: JsonSuccess<'_>, _now: u64) -> Result<Self::Response, Error> {
        self.decode_response(response)
    }
}
