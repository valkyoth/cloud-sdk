#![no_main]

use cloud_sdk::Method;
use cloud_sdk::authentication::{
    AuthenticatedRequest, AuthenticationScopePolicy, BlockingAuthenticatedTransport,
    ScopeRequirement,
};
use cloud_sdk::operation::OperationId;
use cloud_sdk::pagination::{
    PaginationError, PaginationLimits, ProviderLinkBinding, ProviderLinkExecutionError,
    ValidatedProviderLink,
};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointScheme, RawResponsePolicy,
    RequestPath, ResponseBuffer, ResponseMediaPolicy, ResponseWriter,
};
use libfuzzer_sys::fuzz_target;

struct OfficialTransport;

impl BoundTransport for OfficialTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        official_endpoint()
    }
}

impl BlockingAuthenticatedTransport for OfficialTransport {
    type Error = ();

    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        _response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        assert_eq!(
            request.transport_request().target().path().as_str(),
            "/v2/resources"
        );
        Ok(())
    }
}

struct OtherTransport;

impl BoundTransport for OtherTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        EndpointIdentity::new(EndpointScheme::Https, "other.example", 443, "/v2")
    }
}

impl BlockingAuthenticatedTransport for OtherTransport {
    type Error = ();

    fn send_authenticated(
        &self,
        _request: AuthenticatedRequest<'_, '_>,
        _response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        panic!("mismatched transport executed")
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
    let binding = ProviderLinkBinding::new(endpoint, operation, path);
    let mut source = data.get(..8_192).unwrap_or(data).to_vec();
    let mut destination = vec![0xa5_u8; 8_192];
    let mut response_storage = [];
    let mut header_storage = [];
    let mut response = ResponseBuffer::new(&mut response_storage, 0, &mut header_storage);

    {
        let result =
            ValidatedProviderLink::transfer_from(&mut source, &mut destination, binding, limits);
        match result {
            Ok(link) => {
                assert!(source.iter().all(|byte| *byte == 0));
                assert_eq!(
                    link.execute_blocking(
                        &OfficialTransport,
                        Method::Get,
                        operation,
                        authentication(),
                        response_policy(),
                        response.writer(),
                    ),
                    Ok(())
                );
                assert_eq!(
                    link.execute_blocking(
                        &OtherTransport,
                        Method::Get,
                        operation,
                        authentication(),
                        response_policy(),
                        response.writer(),
                    ),
                    Err(ProviderLinkExecutionError::Pagination(
                        PaginationError::ProviderLinkAuthorityChanged
                    ))
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

fn authentication() -> AuthenticationScopePolicy<'static> {
    AuthenticationScopePolicy::new(
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    )
}

fn response_policy() -> RawResponsePolicy<'static> {
    RawResponsePolicy::new(
        0,
        0,
        ResponseMediaPolicy::Forbidden,
        ResponseMediaPolicy::Forbidden,
        &[],
        0,
    )
    .unwrap_or_else(|_| unreachable!())
}
