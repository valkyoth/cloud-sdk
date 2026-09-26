use crate::{
    credentials::{ApiToken, CredentialContext, CredentialOrigin, ScopedCredentialMaterial},
    discovery::{DiscoveryClient, DiscoveryError as Error, DiscoveryExecutionError as Failure},
    endpoint::ApiRequestTarget,
    wire::{JsonSuccess, OfficialApiAttempt},
};
use cloud_sdk::{
    Method,
    transport::{BoundTransport, BoundUserAgent, ResponseBuffer},
};
use cloud_sdk_sanitization::SecretBuffer;

pub(crate) struct Prepared<'a> {
    pub material: ScopedCredentialMaterial<'a>,
    pub body: &'a [u8],
}

pub(crate) fn api<'a, T: BoundTransport + ?Sized>(
    token: &ApiToken,
    method: Method,
    target: ApiRequestTarget<'a>,
    executor: &T,
    credential: &'a mut SecretBuffer<'_>,
    body: &'a [u8],
) -> Result<Prepared<'a>, Error> {
    let context =
        CredentialContext::api(token.origin(), method, target).map_err(|_| Error::Binding)?;
    let material = token
        .stage_for_adapter(&context, executor, credential)
        .map_err(|_| Error::Binding)?;
    Ok(Prepared { material, body })
}

// Only provider-owned implementations can prepare a permit. No arbitrary path,
// decoder or credential-kind implementation is available to downstream code.
pub(crate) trait Permit: Sized {
    type Response;
    fn origin(&self) -> CredentialOrigin;
    fn prepare<'a, T: BoundTransport + ?Sized>(
        &self,
        executor: &T,
        target: &'a mut [u8],
        credential: &'a mut SecretBuffer<'_>,
        body: &'a mut [u8],
        now: u64,
    ) -> Result<Prepared<'a>, Error>;
    fn empty(&self) -> Option<Self::Response> {
        None
    }
    fn decode(self, response: JsonSuccess<'_>, now: u64) -> Result<Self::Response, Error>;
    fn finish<T: BoundTransport + BoundUserAgent + ?Sized, E>(
        self,
        client: &DiscoveryClient<'_, T>,
        response: ResponseBuffer<'_>,
        attempt: &mut OfficialApiAttempt,
    ) -> Result<Self::Response, Failure<E>> {
        let now = client.response_time_seconds().map_err(Failure::Model)?;
        if let Some(value) = self.empty()
            && response
                .with_response(|r| r.status().get() == 204)
                .map_err(|_| Failure::Staging)?
        {
            let delay = crate::trusted_publishing::empty::admit(&response, now)?;
            client
                .defer_response_delay(attempt, delay, now)
                .map_err(Failure::Schedule)?;
            return Ok(value);
        }
        self.decode(client.admit(response, attempt)?, now)
            .map_err(Failure::Model)
    }
}
