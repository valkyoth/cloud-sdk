#![no_main]

use cloud_sdk::Method;
use cloud_sdk::operation::OperationId;
use cloud_sdk::pagination::{
    PaginationError, PaginationLimits, ProviderLinkBinding, ValidatedProviderLink,
};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointScheme, RequestPath,
};
use libfuzzer_sys::fuzz_target;

struct OfficialTransport;

impl BoundTransport for OfficialTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        official_endpoint()
    }
}

struct OtherTransport;

impl BoundTransport for OtherTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        EndpointIdentity::new(EndpointScheme::Https, "other.example", 443, "/v2")
    }
}

fuzz_target!(|data: &[u8]| {
    let Ok(endpoint) = official_endpoint() else {
        return;
    };
    let Ok(path) = RequestPath::new("/v2/resources") else {
        return;
    };
    let Ok(operation) = OperationId::new("list_resources") else {
        return;
    };
    let Ok(limits) = PaginationLimits::new(8, 1_000, 8_192) else {
        return;
    };
    let binding = ProviderLinkBinding::new(endpoint, Method::Get, operation, path);
    let mut source = data.get(..8_192).unwrap_or(data).to_vec();
    let mut destination = vec![0xa5_u8; 8_192];

    {
        let result =
            ValidatedProviderLink::transfer_from(&mut source, &mut destination, binding, limits);
        match result {
            Ok(link) => {
                assert!(source.iter().all(|byte| *byte == 0));
                assert!(
                    link.with_bound_request(
                        &OfficialTransport,
                        Method::Get,
                        operation,
                        |request| { request.target().path() == path }
                    )
                    .is_ok_and(|matches| matches)
                );
                assert_eq!(
                    link.with_bound_request(&OtherTransport, Method::Get, operation, |_| true),
                    Err(PaginationError::ProviderLinkAuthorityChanged)
                );
                drop(link);
            }
            Err(_) => assert!(source.iter().all(|byte| *byte == 0)),
        }
    }
    assert!(destination.iter().all(|byte| *byte == 0));
});

fn official_endpoint() -> Result<EndpointIdentity<'static>, EndpointIdentityError> {
    EndpointIdentity::new(EndpointScheme::Https, "api.example", 443, "/v2")
}
