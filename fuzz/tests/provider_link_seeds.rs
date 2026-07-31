use cloud_sdk::Method;
use cloud_sdk::operation::OperationId;
use cloud_sdk::pagination::{PaginationLimits, ProviderLinkBinding, ValidatedProviderLink};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme, RequestPath};

#[test]
fn named_absolute_provider_link_seed_reaches_the_success_path() {
    let seed = include_bytes!("../seeds/provider_links/valid_absolute");
    assert!(!seed.ends_with(b"\n") && !seed.ends_with(b"\r"));
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.example", 443, "/v2")
        .unwrap_or_else(|_| unreachable!());
    let path = RequestPath::new("/v2/resources").unwrap_or_else(|_| unreachable!());
    let operation = OperationId::new("list_resources").unwrap_or_else(|_| unreachable!());
    let limits = PaginationLimits::new(8, 1_000, 8_192).unwrap_or_else(|_| unreachable!());
    let binding = ProviderLinkBinding::new(endpoint, Method::Get, operation, path);
    let mut source = seed.to_vec();
    let mut destination = [0xa5_u8; 8_192];

    let link = ValidatedProviderLink::transfer_from(&mut source, &mut destination, binding, limits)
        .unwrap_or_else(|_| panic!("reviewed positive provider-link seed was rejected"));
    assert!(source.iter().all(|byte| *byte == 0));
    drop(link);
    assert!(destination.iter().all(|byte| *byte == 0));
}
