use cloud_sdk::transport::{
    CanonicalQuery, EndpointIdentity, EndpointPolicy, EndpointScheme, FormQuery,
    MAX_ENDPOINT_BASE_PATH_BYTES, MAX_ENDPOINT_HOST_BYTES, RequestPath, RequestQuery,
    RequestTarget,
};

use super::{EndpointError, HttpsEndpoint, custom_endpoint, test_timeouts};
use crate::blocking::{
    LinkLocalHttpEndpoint, MAX_CONFIGURED_ENDPOINT_BYTES, RawBlockingClientBuilder, UserAgent,
};

#[test]
fn exact_link_local_endpoint_builds_only_the_raw_credential_free_client() {
    let identity = EndpointIdentity::new(EndpointScheme::Http, "169.254.169.254", 80, "/");
    let (Ok(identity), Some(timeouts), Ok(user_agent)) = (
        identity,
        test_timeouts(),
        UserAgent::new("cloud-sdk-metadata-test/0.97"),
    ) else {
        unreachable!("security fixture construction failed")
    };
    let endpoint = LinkLocalHttpEndpoint::new_with_policy(
        "http://169.254.169.254",
        EndpointPolicy::fixed(identity),
    );
    let Ok(endpoint) = endpoint else {
        unreachable!("security fixture construction failed")
    };
    assert!(
        RawBlockingClientBuilder::new_link_local(endpoint, user_agent, timeouts)
            .build()
            .is_ok()
    );
}

#[test]
fn exact_composite_endpoint_limit_is_accepted() {
    let maximum_host = [
        "a".repeat(63),
        "b".repeat(63),
        "c".repeat(63),
        "d".repeat(61),
    ]
    .join(".");
    assert_eq!(maximum_host.len(), MAX_ENDPOINT_HOST_BYTES);

    let endpoint = std::format!(
        "https://{maximum_host}:65535/{}",
        "p".repeat(MAX_ENDPOINT_BASE_PATH_BYTES - 1)
    );
    assert_eq!(endpoint.len(), MAX_CONFIGURED_ENDPOINT_BYTES);
    assert!(custom_endpoint(&endpoint).is_ok());

    let oversized = std::format!("{endpoint}p");
    assert_eq!(oversized.len(), MAX_CONFIGURED_ENDPOINT_BYTES + 1);
    assert!(matches!(
        custom_endpoint(&oversized),
        Err(EndpointError::InputTooLong)
    ));
}

#[test]
fn canonical_request_targets_preserve_exact_wire_bytes() {
    let endpoint = custom_endpoint("https://api.example.test/v1");
    let path = RequestPath::new("/resources");
    assert!(endpoint.is_ok() && path.is_ok());
    let (Ok(endpoint), Ok(path)) = (endpoint, path) else {
        unreachable!("security fixture construction failed");
    };

    let mut empty_output = [0_u8; 16];
    let empty_query = CanonicalQuery::new("");
    assert!(empty_query.is_ok());
    let Ok(empty_query) = empty_query else {
        unreachable!("security fixture construction failed");
    };
    let empty_target = RequestTarget::assemble(
        path,
        RequestQuery::Canonical(empty_query),
        &mut empty_output,
    );
    assert!(empty_target.is_ok());
    let Ok(empty_target) = empty_target else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        endpoint
            .compose(empty_target)
            .as_ref()
            .map(reqwest::Url::as_str),
        Ok("https://api.example.test/v1/resources?")
    );

    let mut form_output = [0_u8; 48];
    let form_query = FormQuery::new("name=test+server");
    assert!(form_query.is_ok());
    let Ok(form_query) = form_query else {
        unreachable!("security fixture construction failed");
    };
    let form_target =
        RequestTarget::assemble(path, RequestQuery::Form(form_query), &mut form_output);
    assert!(form_target.is_ok());
    let Ok(form_target) = form_target else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        endpoint
            .compose(form_target)
            .as_ref()
            .map(reqwest::Url::as_str),
        Ok("https://api.example.test/v1/resources?name=test+server")
    );
}

#[test]
fn endpoints_reject_authority_and_normalization_ambiguity() {
    let redacted = custom_endpoint("https://api.example.test/v1");
    assert!(redacted.is_ok());
    let Ok(redacted) = redacted else {
        unreachable!("security fixture construction failed");
    };
    let debug = std::format!("{redacted:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("api.example.test"));

    assert!(matches!(
        custom_endpoint("http://api.example.test/v1"),
        Err(EndpointError::HttpsRequired)
    ));
    assert!(matches!(
        custom_endpoint("https://user@api.example.test/v1"),
        Err(EndpointError::CredentialsForbidden)
    ));
    assert!(matches!(
        custom_endpoint("https://api.example.test/v1?token=x"),
        Err(EndpointError::QueryForbidden)
    ));
    assert!(matches!(
        custom_endpoint("https://api.example.test/v1/"),
        Err(EndpointError::TrailingSlash)
    ));
    for endpoint in [
        "https://api.example.test/v1/../admin",
        "https://api.example.test/%76%31",
        "https://api.example.test/v1//admin",
        "https://api.example.test/v1\\admin",
        "https://api.example.test/v1\tadmin",
        "https://api.example.test/v1\nadmin",
        "https://api.example.test/v1\radmin",
        "https://api.example.test/v1 admin",
        "https://api.example.test/v1\u{007f}admin",
        "https://api.example.test/v1\u{00e4}admin",
    ] {
        assert!(matches!(
            custom_endpoint(endpoint),
            Err(EndpointError::IdentityRejected)
        ));
    }
    let oversized = "x".repeat(MAX_CONFIGURED_ENDPOINT_BYTES + 1);
    assert!(matches!(
        custom_endpoint(&oversized),
        Err(EndpointError::InputTooLong)
    ));
    let maximum_path = std::format!(
        "https://api.example.test/{}",
        "a".repeat(MAX_ENDPOINT_BASE_PATH_BYTES - 1)
    );
    assert!(custom_endpoint(&maximum_path).is_ok());
    let oversized_path = std::format!(
        "https://api.example.test/{}",
        "a".repeat(MAX_ENDPOINT_BASE_PATH_BYTES)
    );
    assert!(matches!(
        custom_endpoint(&oversized_path),
        Err(EndpointError::IdentityRejected)
    ));
    for endpoint in [
        "https://API.example.test/v1",
        "https://api.example.test./v1",
        "https://t\u{00e4}st.example/v1",
        "https://api%2eexample.test/v1",
        "https://[fe80::1%25eth0]/v1",
        "https://2001:db8::1/v1",
        "https://127.1/v1",
        "https://127.00.0.1/v1",
        "https://api.example.test:0443/v1",
    ] {
        assert!(
            matches!(
                custom_endpoint(endpoint),
                Err(EndpointError::AmbiguousAuthority)
            ),
            "{endpoint}"
        );
    }
    assert!(custom_endpoint("https://xn--tst-qla.example/v1").is_ok());

    let compressed = custom_endpoint("https://[2001:db8::1]/v1");
    let expanded = custom_endpoint("https://[2001:0db8:0:0:0:0:0:1]/v1");
    assert!(compressed.is_ok() && expanded.is_ok());
    let (Ok(compressed), Ok(expanded)) = (compressed, expanded) else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(compressed.identity(), expanded.identity());

    let official = EndpointIdentity::new(EndpointScheme::Https, "api.example.test", 443, "/v1");
    assert!(official.is_ok());
    let Ok(official) = official else {
        unreachable!("security fixture construction failed");
    };
    let policy = EndpointPolicy::fixed(official);
    assert!(HttpsEndpoint::new_with_policy("https://api.example.test/v1", policy).is_ok());
    assert!(matches!(
        HttpsEndpoint::new_with_policy("https://other.example.test/v1", policy),
        Err(EndpointError::PolicyRejected)
    ));

    let endpoint = custom_endpoint("https://api.example.test/v1");
    let safe = RequestTarget::new("/servers?name=test%20server");
    let (Ok(endpoint), Ok(safe)) = (endpoint, safe) else {
        unreachable!("security fixture construction failed");
    };
    let url = endpoint.compose(safe);
    assert_eq!(
        url.as_ref().map(reqwest::Url::as_str),
        Ok("https://api.example.test/v1/servers?name=test%20server")
    );
    for target in ["/%2e%2e/admin", "/x%2fy", "/x%5cevil", "/x%25%32%66"] {
        assert!(RequestTarget::new(target).is_err());
    }
    assert!(RequestTarget::new("/servers/../admin").is_err());
}
