use crate::{
    client::{RegistryBuffers, prepared::Permit},
    discovery::{DiscoveryClient, DiscoveryError as Error, DiscoveryExecutionError as Failure},
    query::MAX_TARGET_BYTES,
};
use cloud_sdk::transport::*;
use cloud_sdk_sanitization::SecretBuffer;

macro_rules! runner {
    ($run:ident, $transport:ident, $drive:ident, $authorized:ident $(, $bound:ident, $sync:ident)?) => {
        pub(super) async fn $run<T: $transport + BoundUserAgent $(+ $sync)? + ?Sized, R: Permit $(+ $bound)?>(
            client: &DiscoveryClient<'_, T>, permit: R, buffers: RegistryBuffers<'_>,
        ) -> Result<R::Response, Failure<T::Error>> {
            // The public wrapper owns a four-region guard before creating its
            // future. These inner guards protect material during suspension.
            let mut secret = SecretBuffer::new(buffers.credential);
            let mut body = SecretBuffer::new(buffers.body);
            let mut response = ResponseBuffer::new(buffers.response, client.maximum, buffers.headers);
            client.verify().map_err(Failure::Model)?;
            if permit.origin().endpoint() != client.endpoint {
                return Err(Failure::Model(Error::Binding));
            }
            let mut target = [0; MAX_TARGET_BYTES];
            let prepared = permit.prepare(client.executor, &mut target, &mut secret, body.as_mut_slice(),
                client.response_time_seconds().map_err(Failure::Model)?).map_err(Failure::Model)?;
            let headers = [
                RequestHeader::accept(MediaType::JSON),
                RequestHeader::new("accept-encoding", "identity").map_err(|_| Failure::Model(Error::Value))?,
                RequestHeader::new("content-type", "application/json").map_err(|_| Failure::Model(Error::Value))?,
            ];
            let count = if prepared.body.is_empty() { 2 } else { 3 };
            let headers = RequestHeaders::new(headers.get(..count).ok_or(Failure::Model(Error::Limit))?)
                .map_err(|_| Failure::Model(Error::Value))?;
            let policy = if permit.empty().is_some() {
                crate::trusted_publishing::empty::policy(client.maximum)
            } else { client.policy() }.map_err(Failure::Model)?;
            let request = TransportRequest::new(prepared.material.method(), prepared.material.target())
                .with_headers(headers).with_body(prepared.body);
            let mut attempt = client.gate.begin().map_err(Failure::Schedule)?;
            let result = match prepared.material.authorization() {
                Some(authorization) => $authorized(client.executor, prepared.material.endpoint(),
                    authorization, request, policy, response.writer()).await,
                None => $drive(client.executor, request, policy, response.writer()).await,
            };
            result.map_err(|error| match error {
                AsyncExecutionError::Transport(e) => Failure::Transport(e),
                AsyncExecutionError::Response(_) => Failure::Staging,
            })?;
            permit.finish(client, response, &mut attempt)
        }
    };
}
runner!(
    execute_local,
    LocalAuthorizedRawHttpExecutor,
    drive_local_raw,
    drive_local_authorized_raw
);
runner!(
    execute_async,
    AsyncAuthorizedRawHttpExecutor,
    drive_async_raw,
    drive_async_authorized_raw,
    Send,
    Sync
);
