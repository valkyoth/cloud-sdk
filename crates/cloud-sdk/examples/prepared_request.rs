//! Prepares a complete provider-neutral operation without performing I/O.

use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, PreparationStorage,
    PrepareOperation, PreparedRequest, ProviderService, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointScheme, MediaType, RequestTarget, StatusCode,
};
use cloud_sdk::transport::{MAX_REQUEST_TARGET_BYTES, TransportRequest};
use cloud_sdk::{ApiFamily, Method, Provider};

static OK_STATUS: [StatusCode; 1] = [StatusCode::OK];
static JSON_MEDIA: [MediaType<'static>; 1] = [MediaType::JSON];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PrepareError {
    TargetBuffer,
    InvalidTarget,
    InvalidMetadata,
    InvalidResponsePolicy,
    InvalidEndpoint,
}

struct ListResources;

impl PrepareOperation for ListResources {
    type Error = PrepareError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        let (target_storage, _body_storage) = storage.into_parts();
        let target = target_storage
            .get_mut(..17)
            .ok_or(PrepareError::TargetBuffer)?;
        target.copy_from_slice(b"/resources?page=1");
        let target = core::str::from_utf8(target).map_err(|_| PrepareError::InvalidTarget)?;
        let target = RequestTarget::new(target).map_err(|_| PrepareError::InvalidTarget)?;
        let request = TransportRequest::new(Method::Get, target);
        let metadata = OperationMetadata::new(
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
            CostIntent::NoKnownCost,
        )
        .map_err(|_| PrepareError::InvalidMetadata)?;
        let response_policy = ResponsePolicy::new(
            &OK_STATUS,
            ContentTypePolicy::Required(&JSON_MEDIA),
            ResponseBodyPolicy::Required,
            65_536,
        )
        .map_err(|_| PrepareError::InvalidResponsePolicy)?;
        let endpoint =
            EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1")
                .map_err(|_| PrepareError::InvalidEndpoint)?;
        Ok(PreparedRequest::new(
            request,
            ProviderService::new(Provider::Hetzner, ApiFamily::Cloud, endpoint),
            metadata,
            response_policy,
        ))
    }
}

fn main() {
    let mut target = [0_u8; MAX_REQUEST_TARGET_BYTES];
    let mut body = [0_u8; 1];
    let prepared = ListResources.prepare(PreparationStorage::new(&mut target, &mut body));
    let Ok(prepared) = prepared else { return };

    assert_eq!(prepared.transport_request().method(), Method::Get);
    assert_eq!(prepared.metadata().impact(), OperationImpact::ReadOnly);
    assert_eq!(prepared.response_policy().max_body_bytes(), 65_536);
}
