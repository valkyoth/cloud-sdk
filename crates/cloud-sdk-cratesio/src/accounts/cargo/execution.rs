use super::{CargoOwners, CargoOwnersRequest, Error};
use crate::credentials::CredentialContext;

#[cfg(feature = "async")]
impl crate::client::prepared::Permit for CargoOwnersRequest<'_> {
    type Response = CargoOwners;
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
        let context = CredentialContext::cargo_owners(self.token.origin(), self.name, target)
            .map_err(|_| Error::Binding)?;
        let material = self
            .token
            .stage_for_adapter(&context, executor, credential)
            .map_err(|_| Error::Binding)?;
        Ok(crate::client::prepared::Prepared {
            material,
            body: &[],
        })
    }
    fn decode(
        self,
        response: crate::wire::JsonSuccess<'_>,
        _now: u64,
    ) -> Result<CargoOwners, Error> {
        CargoOwners::decode(response)
    }
}

#[cfg(feature = "blocking")]
impl CargoOwnersRequest<'_> {
    pub(crate) fn execute<
        T: cloud_sdk::transport::BlockingAuthorizedRawHttpExecutor
            + cloud_sdk::transport::BoundUserAgent
            + ?Sized,
    >(
        self,
        client: &crate::discovery::DiscoveryClient<'_, T>,
        buffers: crate::client::RegistryBuffers<'_>,
    ) -> Result<CargoOwners, crate::discovery::DiscoveryExecutionError<T::Error>> {
        use crate::discovery::DiscoveryExecutionError as Failure;
        use cloud_sdk::transport::*;
        use cloud_sdk_sanitization::SecretBuffer;
        let mut secret = SecretBuffer::new(buffers.credential);
        let _body = SecretBuffer::new(buffers.body);
        let mut response = ResponseBuffer::new(buffers.response, client.maximum, buffers.headers);
        client.verify().map_err(Failure::Model)?;
        if self.token.origin().endpoint() != client.endpoint {
            return Err(Failure::Model(Error::Binding));
        }
        let mut target = [0; crate::query::MAX_TARGET_BYTES];
        let context = CredentialContext::cargo_owners(self.token.origin(), self.name, &mut target)
            .map_err(|_| Failure::Model(Error::Binding))?;
        let headers = client.headers().map_err(Failure::Model)?;
        let policy = client.policy().map_err(Failure::Model)?;
        let mut attempt = self
            .token
            .with_material_for_adapter(
                &context,
                client.executor,
                secret.as_mut_slice(),
                |executor, material| {
                    let request = TransportRequest::new(material.method(), material.target())
                        .with_headers(
                            RequestHeaders::new(&headers)
                                .map_err(|_| Failure::Model(Error::Value))?,
                        );
                    let active = client.gate.begin().map_err(Failure::Schedule)?;
                    let authorization = material
                        .authorization()
                        .ok_or(Failure::Model(Error::Binding))?;
                    executor
                        .execute_authorized(
                            material.endpoint(),
                            authorization,
                            request,
                            policy,
                            response.writer(),
                        )
                        .map_err(Failure::Transport)?;
                    Ok(active)
                },
            )
            .map_err(|_| Failure::Model(Error::Binding))??;
        CargoOwners::decode(client.admit(response, &mut attempt)?).map_err(Failure::Model)
    }
}
