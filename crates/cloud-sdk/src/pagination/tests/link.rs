use super::super::{PaginationError, PaginationLimits, ProviderLinkBinding, ValidatedProviderLink};
use super::assert_redacted;
use crate::Method;
use crate::operation::OperationId;
use crate::transport::{EndpointIdentity, EndpointScheme, RequestPath};

fn operation(value: &'static str) -> OperationId {
    OperationId::new(value).unwrap_or_else(|_| unreachable!())
}

fn binding() -> ProviderLinkBinding<'static> {
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.digitalocean.com", 443, "/v2")
        .unwrap_or_else(|_| unreachable!());
    let path = RequestPath::new("/v2/droplets").unwrap_or_else(|_| unreachable!());
    ProviderLinkBinding::new(endpoint, Method::Get, operation("list_droplets"), path)
}

fn limits() -> PaginationLimits {
    PaginationLimits::new(8, 1_000, 512).unwrap_or_else(|_| unreachable!())
}

#[test]
fn digitalocean_absolute_link_preserves_raw_query_order_duplicates_and_percent_encoding() {
    let expected = "/v2/droplets?tag_name=a%2fb&filter=a+b==&raw=%41&page=2&page=3";
    let mut source =
        *b"https://api.digitalocean.com/v2/droplets?tag_name=a%2fb&filter=a+b==&raw=%41&page=2&page=3";
    let mut storage = [0xa5_u8; 128];
    {
        let link =
            ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits())
                .unwrap_or_else(|_| unreachable!());
        let observed = link.with_request(Method::Get, operation("list_droplets"), |request| {
            let target = request.target();
            assert!(matches!(
                target.query(),
                crate::transport::RequestQuery::ProviderLink(_)
            ));
            let mut output = [0xa5_u8; 128];
            assert_eq!(
                crate::transport::RequestTarget::assemble(
                    RequestPath::new("/v2/account").unwrap_or_else(|_| unreachable!()),
                    target.query(),
                    &mut output,
                ),
                Err(crate::transport::RequestTargetError::ProviderLinkQueryCannotAssemble)
            );
            assert_eq!(output, [0xa5; 128]);
            target.as_str() == expected
        });
        assert_eq!(observed, Ok(true));
        assert_redacted(&link);
    }
    assert!(source.iter().all(|byte| *byte == 0));
    assert_eq!(storage, [0; 128]);
}

#[test]
fn origin_form_link_remains_operation_bound() {
    let mut source = *b"/v2/droplets?page=2&per_page=20";
    let mut storage = [0_u8; 64];
    let link = ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits())
        .unwrap_or_else(|_| unreachable!());
    assert!(
        link.with_request(Method::Get, operation("list_droplets"), |_| true)
            .is_ok()
    );
    assert_eq!(
        link.with_request(Method::Post, operation("list_droplets"), |_| true),
        Err(PaginationError::ProviderLinkMethodChanged)
    );
    assert_eq!(
        link.with_request(Method::Get, operation("delete_droplet"), |_| true),
        Err(PaginationError::ProviderLinkOperationChanged)
    );
}

#[test]
fn rejects_scheme_authority_userinfo_fragment_and_operation_path_changes() {
    let cases: [(&[u8], PaginationError); 5] = [
        (
            b"http://api.digitalocean.com/v2/droplets?page=2",
            PaginationError::ProviderLinkSchemeChanged,
        ),
        (
            b"https://evil.example/v2/droplets?page=2",
            PaginationError::ProviderLinkAuthorityChanged,
        ),
        (
            b"https://user@api.digitalocean.com/v2/droplets?page=2",
            PaginationError::ProviderLinkUserinfo,
        ),
        (
            b"https://api.digitalocean.com/v2/droplets?page=2#next",
            PaginationError::ProviderLinkFragment,
        ),
        (
            b"https://api.digitalocean.com/v2/account?page=2",
            PaginationError::ProviderLinkPathChanged,
        ),
    ];
    for (value, expected) in cases {
        let mut source = [0_u8; 96];
        let Some(source) = source.get_mut(..value.len()) else {
            return;
        };
        source.copy_from_slice(value);
        let mut destination = [0xa5_u8; 128];
        assert!(matches!(
            ValidatedProviderLink::transfer_from(source, &mut destination, binding(), limits()),
            Err(error) if error == expected
        ));
        assert!(source.iter().all(|byte| *byte == 0));
        assert_eq!(destination, [0; 128]);
    }
}

#[test]
fn accepts_explicit_default_port_and_rejects_insufficient_output_atomically() {
    let mut source = *b"https://api.digitalocean.com:443/v2/droplets?page=2";
    let mut storage = [0_u8; 64];
    assert!(
        ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding(), limits(),)
            .is_ok()
    );

    let mut source = *b"/v2/droplets?page=2";
    let mut output = [0xa5_u8; 4];
    assert!(matches!(
        ValidatedProviderLink::transfer_from(&mut source, &mut output, binding(), limits()),
        Err(PaginationError::OutputTooSmall)
    ));
    assert!(source.iter().all(|byte| *byte == 0));
    assert_eq!(output, [0; 4]);
}

#[test]
fn accepts_equivalent_ipv6_authority_and_rejects_invalid_raw_queries() {
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "[2001:db8::1]", 443, "/v2")
        .unwrap_or_else(|_| unreachable!());
    let path = RequestPath::new("/v2/droplets").unwrap_or_else(|_| unreachable!());
    let binding = ProviderLinkBinding::new(endpoint, Method::Get, operation("list_droplets"), path);
    let mut source = *b"https://[2001:0db8:0:0:0:0:0:1]/v2/droplets?page=2";
    let mut storage = [0_u8; 64];
    assert!(
        ValidatedProviderLink::transfer_from(&mut source, &mut storage, binding, limits()).is_ok()
    );

    for value in [
        b"/v2/droplets?cursor=%".as_slice(),
        b"/v2/droplets?cursor=%00".as_slice(),
        b"/v2/droplets?cursor=a b".as_slice(),
        b"/v2/droplets?cursor=a\\b".as_slice(),
    ] {
        let mut source = [0_u8; 32];
        let Some(source) = source.get_mut(..value.len()) else {
            return;
        };
        source.copy_from_slice(value);
        let mut destination = [0xa5_u8; 64];
        assert!(matches!(
            ValidatedProviderLink::transfer_from(source, &mut destination, binding, limits()),
            Err(PaginationError::InvalidProviderLink)
        ));
        assert!(source.iter().all(|byte| *byte == 0));
        assert_eq!(destination, [0; 64]);
    }
}
