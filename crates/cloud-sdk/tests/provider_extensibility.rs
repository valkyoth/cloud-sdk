//! External-crate proof that providers and services need no core registration.

use cloud_sdk::operation::ProviderService;
use cloud_sdk::transport::{EndpointIdentity, EndpointPolicy, EndpointScheme};
use cloud_sdk::{ProviderId, ProviderMarker, ServiceId, ServiceMarker, provider_id, service_id};

enum IndependentProvider {}

impl ProviderMarker for IndependentProvider {
    const ID: ProviderId = provider_id!("independent-cloud");
}

enum ComputeV2 {}

impl ServiceMarker for ComputeV2 {
    type Provider = IndependentProvider;
    const ID: ServiceId = service_id!("compute-v2");
}

#[test]
fn external_provider_owns_identity_without_core_enum_changes() {
    let endpoint =
        EndpointIdentity::new(EndpointScheme::Https, "api.independent.invalid", 443, "/v2");
    assert!(endpoint.is_ok());
    let Ok(endpoint) = endpoint else {
        unreachable!("security fixture construction failed")
    };
    let policy = EndpointPolicy::fixed(endpoint);
    let service = ProviderService::from_marker::<ComputeV2>(policy);

    assert_eq!(service.provider_id(), IndependentProvider::ID);
    assert_eq!(service.service_id(), ComputeV2::ID);
    assert_eq!(service.endpoint_policy(), policy);
}
