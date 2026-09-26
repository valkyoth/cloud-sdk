use super::{TokenOperation, TokenPermit, TokenResponse};
use crate::{
    client::prepared::{self, Permit, Prepared},
    credentials::CredentialOrigin,
    discovery::DiscoveryError as Error,
    wire::JsonSuccess,
};
use cloud_sdk::transport::BoundTransport;
use cloud_sdk_sanitization::SecretBuffer;

impl Permit for TokenPermit<'_> {
    type Response = TokenResponse;
    fn origin(&self) -> CredentialOrigin {
        self.origin()
    }
    fn prepare<'a, T: BoundTransport + ?Sized>(
        &self,
        executor: &T,
        target: &'a mut [u8],
        credential: &'a mut SecretBuffer<'_>,
        _body: &'a mut [u8],
        _now: u64,
    ) -> Result<Prepared<'a>, Error> {
        let target = self.write_target(target).map_err(|_| Error::Binding)?;
        prepared::api(
            self.credential,
            self.operation.method(),
            target,
            executor,
            credential,
            &[],
        )
    }
    fn empty(&self) -> Option<Self::Response> {
        (self.operation == TokenOperation::RevokeCurrent).then_some(TokenResponse::CurrentRevoked)
    }
    fn decode(self, response: JsonSuccess<'_>, _now: u64) -> Result<Self::Response, Error> {
        self.decode_response(response)
    }
}
