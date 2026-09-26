use super::*;
use crate::{
    client::prepared::{self, Permit, Prepared},
    credentials::CredentialOrigin,
    discovery::DiscoveryError as Error,
    wire::JsonSuccess,
};
use cloud_sdk::transport::BoundTransport;
use cloud_sdk_sanitization::SecretBuffer;

impl<'r> Permit for YankPermit<'r> {
    type Response = YankAcknowledgement<'r>;
    fn origin(&self) -> CredentialOrigin {
        self.credential.origin()
    }
    fn prepare<'a, T: BoundTransport + ?Sized>(
        &self,
        executor: &T,
        target: &'a mut [u8],
        credential: &'a mut SecretBuffer<'_>,
        _body: &'a mut [u8],
        _now: u64,
    ) -> Result<Prepared<'a>, Error> {
        let body = &[];
        let target = self
            .request
            .write_target(target)
            .map_err(|_| Error::Binding)?;
        prepared::api(
            self.credential,
            self.operation().method(),
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
