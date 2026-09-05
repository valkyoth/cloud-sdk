use super::{
    tests::{Transport, credential, transport},
    *,
};
use crate::endpoint::ApiRequestTarget;
use cloud_sdk::Method;
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme, RequestHeader};

#[test]
fn every_admitted_api_route_has_an_accepted_context() {
    for (method, template) in super::policy::API_ROUTES {
        let path = template
            .replace("{name}", "example")
            .replace("{version}", "1.0.0")
            .replace("{id}", "1")
            .replace("{user}", "2")
            .replace("{crate_id}", "3");
        let target = ApiRequestTarget::new(&path).unwrap_or_else(|_| unreachable!("route fixture"));
        assert!(
            CredentialContext::api(CredentialOrigin::Production, *method, target).is_ok(),
            "{template}"
        );
    }
}

#[test]
fn anonymous_foreign_cookie_and_wrong_method_routes_cannot_gain_api_authorization() {
    for (method, path) in [
        (Method::Get, "/api/v1/summary"),
        (Method::Get, "/api/v1/me"),
        (Method::Get, "/api/v1/crates/example/1.0.0/download"),
        (Method::Post, "/api/v1/trusted_publishing/tokens"),
        (Method::Delete, "/api/v1/trusted_publishing/tokens"),
        (Method::Get, "/api/v1/crates/new"),
        (Method::Post, "/api/v1/crates/new"),
        (Method::Put, "/api/v1/crates/new?extra=value"),
        (
            Method::Put,
            "/api/v1/me/crate_owner_invitations/accept/secret",
        ),
        (Method::Put, "/api/v1/confirm/secret"),
        (Method::Put, "/api/v1/users/not-an-id"),
        (Method::Patch, "/api/v1/crates/%2e%2e"),
    ] {
        let target = ApiRequestTarget::new(path);
        match target {
            Ok(target) => assert!(
                CredentialContext::api(CredentialOrigin::Production, method, target).is_err()
            ),
            Err(_) => assert!(path.contains('%'), "unexpected fixture rejection"),
        }
    }
    // Ordinary headers cannot inject a competing Authorization field, with
    // either spelling or sensitivity. Adapter material owns exactly one slot.
    for name in [
        "Authorization",
        "authorization",
        "AUTHORIZATION",
        "Proxy-Authorization",
    ] {
        assert!(RequestHeader::new(name, "competing").is_err());
        assert!(RequestHeader::sensitive(name, "competing").is_err());
    }
}

#[test]
fn wrong_origins_schemes_ports_and_paths_fail_before_material_callback() {
    let token = credential::<TrustedPublishing>(b"destination-fixture");
    let context = CredentialContext::publish(CredentialOrigin::Production);
    for (scheme, host, port, path) in [
        (EndpointScheme::Https, "evil.example", 443, "/"),
        (EndpointScheme::Https, "static.crates.io", 443, "/"),
        (EndpointScheme::Https, "staging.crates.io", 443, "/"),
        (EndpointScheme::Https, "crates.io.evil.example", 443, "/"),
        (EndpointScheme::Https, "crates.io", 444, "/"),
        (EndpointScheme::Https, "crates.io", 443, "/api"),
        (EndpointScheme::Http, "crates.io", 80, "/"),
    ] {
        let transport = Transport(
            EndpointIdentity::new(scheme, host, port, path)
                .unwrap_or_else(|_| unreachable!("endpoint fixture")),
        );
        let mut output = [0xa5; 128];
        assert_eq!(
            token.with_material_for_adapter(
                &context,
                &transport,
                &mut output,
                |_, _| unreachable!("wrong destination applied")
            ),
            Err(CredentialError::DestinationMismatch)
        );
        assert_eq!(output, [0; 128]);
    }
    let mut output = [0xa5; 128];
    let staging = CredentialContext::publish(CredentialOrigin::Staging);
    assert_eq!(
        token.with_material_for_adapter(&staging, &transport(), &mut output, |_, _| unreachable!(
            "wrong context applied"
        )),
        Err(CredentialError::DestinationMismatch)
    );
    assert_eq!(output, [0; 128]);

    let mut staging_source = *b"staging-fixture";
    let staging_token =
        TrustedPublishingToken::from_mut_bytes(CredentialOrigin::Staging, &mut staging_source)
            .unwrap_or_else(|_| unreachable!("staging fixture"));
    let staging_transport = Transport(
        CredentialOrigin::Staging
            .endpoint()
            .identity()
            .unwrap_or_else(|_| unreachable!("staging endpoint")),
    );
    staging_token
        .with_material_for_adapter(
            &staging,
            &staging_transport,
            &mut output,
            |transport, material| {
                assert_eq!(material.endpoint().host(), "staging.crates.io");
                assert!(core::ptr::eq(transport, &staging_transport));
            },
        )
        .unwrap_or_else(|_| unreachable!("staging material"));
    assert_eq!(output, [0; 128]);
}
