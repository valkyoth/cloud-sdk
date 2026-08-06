//! Source-locked OVHcloud authority and OAuth expiry conformance fixtures.

use cloud_sdk::authentication::{
    CredentialLifetime, CredentialLifetimeError, CredentialLifetimeState, CredentialTimestamp,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointPairPolicy, EndpointPairPolicyError, EndpointScheme,
    RegionalEndpointPair,
};

const REVIEWED_PAIRS: &str = include_str!("fixtures/ovhcloud-authority-pairs.tsv");

fn endpoint(host: &'static str, path: &'static str) -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, host, 443, path).unwrap_or_else(|_| unreachable!())
}

fn pairs() -> [RegionalEndpointPair<'static>; 2] {
    assert_eq!(
        REVIEWED_PAIRS,
        concat!(
            "region\tapi_host\tapi_port\tapi_base_path\t",
            "token_host\ttoken_port\ttoken_base_path\n",
            "ca\tca.api.ovh.com\t443\t/v2\tca.ovh.com\t443\t/auth/oauth2/token\n",
            "eu\teu.api.ovh.com\t443\t/v2\twww.ovh.com\t443\t/auth/oauth2/token\n",
        )
    );
    [
        RegionalEndpointPair::new(
            "ca",
            endpoint("ca.api.ovh.com", "/v2"),
            endpoint("ca.ovh.com", "/auth/oauth2/token"),
        )
        .unwrap_or_else(|_| unreachable!()),
        RegionalEndpointPair::new(
            "eu",
            endpoint("eu.api.ovh.com", "/v2"),
            endpoint("www.ovh.com", "/auth/oauth2/token"),
        )
        .unwrap_or_else(|_| unreachable!()),
    ]
}

#[test]
fn reviewed_geographic_api_and_token_authorities_are_exact_pairs() {
    let pairs = pairs();
    let policy = EndpointPairPolicy::new(&pairs).unwrap_or_else(|_| unreachable!());
    let eu_api = endpoint("eu.api.ovh.com", "/v2");
    let eu_token = endpoint("www.ovh.com", "/auth/oauth2/token");
    let ca_api = endpoint("ca.api.ovh.com", "/v2");
    let ca_token = endpoint("ca.ovh.com", "/auth/oauth2/token");
    assert!(policy.verify("eu", eu_api, eu_token).is_ok());
    assert!(policy.verify("ca", ca_api, ca_token).is_ok());
    assert_eq!(
        policy.verify("eu", eu_api, ca_token),
        Err(EndpointPairPolicyError::PairMismatch)
    );
    assert_eq!(
        policy.verify("ca", ca_api, eu_token),
        Err(EndpointPairPolicyError::PairMismatch)
    );
}

#[test]
fn console_and_historical_aliases_are_not_credential_destinations() {
    let pairs = pairs();
    let policy = EndpointPairPolicy::new(&pairs).unwrap_or_else(|_| unreachable!());
    let eu_token = endpoint("www.ovh.com", "/auth/oauth2/token");
    for alias in [
        endpoint("api.eu.ovhcloud.com", "/v2"),
        endpoint("eu.api.ovh.com", "/v1"),
        endpoint("api.ovh.com", "/v2"),
    ] {
        assert_eq!(
            policy.verify("eu", alias, eu_token),
            Err(EndpointPairPolicyError::PairMismatch)
        );
    }
    let eu_api = endpoint("eu.api.ovh.com", "/v2");
    for alias in [
        endpoint("eu.ovh.com", "/auth/oauth2/token"),
        endpoint("www.ovh.com", "/auth/oauth2"),
    ] {
        assert_eq!(
            policy.verify("eu", eu_api, alias),
            Err(EndpointPairPolicyError::PairMismatch)
        );
    }
}

#[test]
fn expires_in_is_converted_through_caller_time_with_a_refresh_margin() {
    let lifetime =
        CredentialLifetime::from_expires_in(CredentialTimestamp::from_seconds(10_000), 3_599, 300)
            .unwrap_or_else(|_| unreachable!());
    assert_eq!(lifetime.refresh_at().as_seconds(), 13_299);
    assert_eq!(lifetime.expires_at().as_seconds(), 13_599);
    assert_eq!(
        lifetime.state_at(CredentialTimestamp::from_seconds(13_298)),
        CredentialLifetimeState::Fresh
    );
    assert_eq!(
        lifetime.state_at(CredentialTimestamp::from_seconds(13_299)),
        CredentialLifetimeState::RefreshRequired
    );
    assert_eq!(
        lifetime.state_at(CredentialTimestamp::from_seconds(13_599)),
        CredentialLifetimeState::Expired
    );
    assert_eq!(
        CredentialLifetime::from_expires_in(CredentialTimestamp::from_seconds(0), 3_599, 3_599),
        Err(CredentialLifetimeError::RefreshWindowTooLarge)
    );
}
