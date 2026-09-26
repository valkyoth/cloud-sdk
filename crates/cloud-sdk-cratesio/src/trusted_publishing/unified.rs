use super::{TrustedPublishingPermit, TrustedPublishingResponse, request::Intent};
use crate::{
    client::prepared::{self, Permit, Prepared},
    credentials::{CredentialContext, CredentialOrigin},
    discovery::DiscoveryError as Error,
    wire::JsonSuccess,
};
use cloud_sdk::transport::BoundTransport;
use cloud_sdk_sanitization::SecretBuffer;

impl Permit for TrustedPublishingPermit<'_> {
    type Response = TrustedPublishingResponse;
    fn origin(&self) -> CredentialOrigin {
        self.origin()
    }
    fn prepare<'a, T: BoundTransport + ?Sized>(
        &self,
        executor: &T,
        target: &'a mut [u8],
        credential: &'a mut SecretBuffer<'_>,
        body: &'a mut [u8],
        now: u64,
    ) -> Result<Prepared<'a>, Error> {
        let target = self.write_target(target).map_err(|_| Error::Binding)?;
        let length = self.body(body)?;
        let body = body.get(..length).ok_or(Error::Limit)?;
        match &self.0 {
            Intent::List(_, _, token) | Intent::Create(_, token) | Intent::Delete(_, _, token) => {
                prepared::api(
                    token,
                    self.operation().method(),
                    target,
                    executor,
                    credential,
                    body,
                )
            }
            Intent::Exchange(token, policy) => {
                let material = token
                    .stage_for_adapter(
                        &CredentialContext::exchange(self.origin()),
                        executor,
                        credential,
                    )
                    .map_err(|_| Error::Binding)?;
                let body = material.json_body().ok_or(Error::Binding)?;
                policy.preflight(body, now)?;
                Ok(Prepared { material, body })
            }
            Intent::Revoke(token) => {
                let material = token
                    .credential
                    .stage_for_adapter(
                        &CredentialContext::revoke(self.origin()),
                        executor,
                        credential,
                    )
                    .map_err(|_| Error::Binding)?;
                Ok(Prepared { material, body })
            }
        }
    }
    fn empty(&self) -> Option<Self::Response> {
        match self.0 {
            Intent::Delete(..) => Some(TrustedPublishingResponse::Deleted),
            Intent::Revoke(..) => Some(TrustedPublishingResponse::Revoked),
            _ => None,
        }
    }
    fn decode(self, response: JsonSuccess<'_>, now: u64) -> Result<Self::Response, Error> {
        self.decode_response(response, now)
    }
}
