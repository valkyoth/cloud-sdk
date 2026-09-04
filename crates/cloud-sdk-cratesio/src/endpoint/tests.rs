use cloud_sdk::transport::{
    BoundTransport, CustomEndpointAcknowledgement, EndpointIdentity, EndpointIdentityError,
    EndpointScheme, RequestPathError, RequestTargetError,
};

use super::{
    AcknowledgedCustomApiEndpoint, ApiRequestTarget, CRATES_IO_API_BASE_URL,
    CRATES_IO_STAGING_API_BASE_URL, CRATES_IO_STATIC_DOWNLOAD_BASE_URL, CratesIoEndpointError,
    CratesIoTargetError, DownloadRedirect, DownloadRedirectError,
    MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES, OfficialCratesIoEndpoint, OfficialEndpointPurpose,
    RedirectAuthorization, StaticDownloadTarget,
};

#[test]
fn official_constructors_bind_exact_https_authorities() {
    let cases = [
        (
            OfficialCratesIoEndpoint::production_api(),
            OfficialEndpointPurpose::ProductionApi,
            CRATES_IO_API_BASE_URL,
            "crates.io",
        ),
        (
            OfficialCratesIoEndpoint::staging_api(),
            OfficialEndpointPurpose::StagingApi,
            CRATES_IO_STAGING_API_BASE_URL,
            "staging.crates.io",
        ),
        (
            OfficialCratesIoEndpoint::static_downloads(),
            OfficialEndpointPurpose::StaticDownloads,
            CRATES_IO_STATIC_DOWNLOAD_BASE_URL,
            "static.crates.io",
        ),
    ];
    for (endpoint, purpose, base_url, host) in cases {
        assert_eq!(endpoint.purpose(), purpose);
        assert_eq!(endpoint.base_url(), base_url);
        let identity = endpoint.identity();
        assert!(identity.is_ok());
        let Ok(identity) = identity else {
            unreachable!("official endpoint fixture construction failed");
        };
        assert_eq!(identity.scheme(), EndpointScheme::Https);
        assert_eq!(identity.host(), host);
        assert_eq!(identity.effective_port(), 443);
        assert_eq!(identity.base_path(), "/");
        assert!(
            endpoint
                .policy()
                .is_ok_and(|policy| policy.admits(identity))
        );
    }
}

#[test]
fn official_policies_reject_host_port_path_and_scheme_confusion() {
    let endpoint = OfficialCratesIoEndpoint::production_api();
    for candidate in [
        identity(EndpointScheme::Https, "evil.example", 443, "/"),
        identity(EndpointScheme::Https, "crates.io.evil.example", 443, "/"),
        identity(EndpointScheme::Https, "crates.io", 444, "/"),
        identity(EndpointScheme::Https, "crates.io", 443, "/api"),
        identity(EndpointScheme::Http, "crates.io", 80, "/"),
    ] {
        let Ok(candidate) = candidate else {
            unreachable!("endpoint confusion fixture construction failed");
        };
        assert_eq!(
            endpoint.verify_transport(&StubTransport(candidate)),
            Err(CratesIoEndpointError::DestinationMismatch)
        );
    }
}

#[test]
fn custom_api_endpoints_require_acknowledgement_and_https() {
    let acknowledgement = CustomEndpointAcknowledgement::trusted_operator_configuration();
    let https = identity(EndpointScheme::Https, "registry.example", 443, "/api");
    let http = identity(EndpointScheme::Http, "registry.example", 80, "/api");
    assert!(https.is_ok() && http.is_ok());
    let (Ok(https), Ok(http)) = (https, http) else {
        unreachable!("custom endpoint fixture construction failed");
    };
    let endpoint = AcknowledgedCustomApiEndpoint::new(https, acknowledgement);
    assert!(
        endpoint.is_ok_and(|endpoint| {
            endpoint.identity() == https && endpoint.policy().admits(https)
        })
    );
    assert_eq!(
        AcknowledgedCustomApiEndpoint::new(http, acknowledgement),
        Err(CratesIoEndpointError::HttpsRequired)
    );
}

#[test]
fn api_targets_reject_absolute_authority_and_ambiguous_paths() {
    assert_eq!(
        ApiRequestTarget::new("/api/v1/crates?q=serde").map(ApiRequestTarget::as_str),
        Ok("/api/v1/crates?q=serde")
    );
    assert_eq!(
        ApiRequestTarget::new("https://evil.example/api/v1/crates"),
        Err(CratesIoTargetError::InvalidTarget(
            RequestTargetError::Path(RequestPathError::NotOriginForm)
        ))
    );
    for value in [
        "//evil.example/api/v1/crates",
        "/api/v1/../tokens",
        "/api/v1/%2F%2Fevil.example",
        "/api/v1/crates#fragment",
        "/api/v1/crates\nheader",
        "/api/v1/crat\u{e9}s",
    ] {
        assert!(matches!(
            ApiRequestTarget::new(value),
            Err(CratesIoTargetError::InvalidTarget(_))
        ));
    }
    assert_eq!(
        ApiRequestTarget::new("/account"),
        Err(CratesIoTargetError::OutsideApiNamespace)
    );
}

#[test]
fn static_targets_are_anonymous_path_only_archives() {
    assert_eq!(
        StaticDownloadTarget::new("/crates/serde/serde-1.0.0.crate")
            .map(StaticDownloadTarget::as_str),
        Ok("/crates/serde/serde-1.0.0.crate")
    );
    assert_eq!(
        StaticDownloadTarget::new("/crates/serde/serde-1.0.0.crate?token=secret"),
        Err(CratesIoTargetError::StaticQueryForbidden)
    );
    for value in [
        "/api/v1/crates/serde/1.0.0/download",
        "/crates/serde/",
        "/crates/serde/nested/serde-1.0.0.crate",
        "/crates/serde/serde-1.0.0.zip",
    ] {
        assert_eq!(
            StaticDownloadTarget::new(value),
            Err(CratesIoTargetError::InvalidStaticDownloadPath)
        );
    }
}

#[test]
fn download_redirect_is_exact_correlated_and_never_authorized() {
    let source = ApiRequestTarget::new("/api/v1/crates/serde/1.0.0/download");
    assert!(source.is_ok());
    let Ok(source) = source else {
        unreachable!("download source fixture construction failed");
    };
    let redirect = DownloadRedirect::new(
        OfficialCratesIoEndpoint::production_api(),
        source,
        "https://static.crates.io/crates/serde/serde-1.0.0.crate",
    );
    assert!(redirect.is_ok());
    let Ok(redirect) = redirect else {
        unreachable!("download redirect fixture construction failed");
    };
    assert_eq!(
        redirect.endpoint(),
        OfficialCratesIoEndpoint::static_downloads()
    );
    assert_eq!(redirect.authorization(), RedirectAuthorization::Omit);
    assert_eq!(
        redirect.target().as_str(),
        "/crates/serde/serde-1.0.0.crate"
    );
}

#[test]
fn redirects_reject_downgrade_user_info_ports_and_authority_confusion() {
    let source = ApiRequestTarget::new("/api/v1/crates/serde/1.0.0/download");
    assert!(source.is_ok());
    let Ok(source) = source else {
        unreachable!("download source fixture construction failed");
    };
    for location in [
        "http://static.crates.io/crates/serde/serde-1.0.0.crate",
        "https://user@static.crates.io/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io:443/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io.evil.example/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io@evil.example/crates/serde/serde-1.0.0.crate",
        "https://static.crates.io./crates/serde/serde-1.0.0.crate",
        "https://static.crat\u{e9}s.io/crates/serde/serde-1.0.0.crate",
    ] {
        assert!(
            DownloadRedirect::new(OfficialCratesIoEndpoint::production_api(), source, location,)
                .is_err()
        );
    }
}

#[test]
fn redirects_reject_wrong_source_and_destination_content() {
    let source = ApiRequestTarget::new("/api/v1/crates/serde/1.0.0/download");
    assert!(source.is_ok());
    let Ok(source) = source else {
        unreachable!("download source fixture construction failed");
    };
    assert_eq!(
        DownloadRedirect::new(
            OfficialCratesIoEndpoint::staging_api(),
            source,
            "https://static.crates.io/crates/serde/serde-1.0.0.crate",
        ),
        Err(DownloadRedirectError::InvalidSourceEndpoint)
    );
    for location in [
        "https://static.crates.io/crates/other/other-1.0.0.crate",
        "https://static.crates.io/crates/serde/serde-2.0.0.crate",
        "https://static.crates.io/crates/serde/other-1.0.0.crate",
        "https://static.crates.io/crates/serde/serde-1.0.0.crate?token=secret",
        "https://static.crates.io/crates/serde/%2E%2E.crate",
    ] {
        assert!(
            DownloadRedirect::new(OfficialCratesIoEndpoint::production_api(), source, location,)
                .is_err()
        );
    }
    let wrong_source = ApiRequestTarget::new("/api/v1/crates/serde");
    assert!(wrong_source.is_ok());
    let Ok(wrong_source) = wrong_source else {
        unreachable!("wrong download source fixture construction failed");
    };
    assert_eq!(
        DownloadRedirect::new(
            OfficialCratesIoEndpoint::production_api(),
            wrong_source,
            "https://static.crates.io/crates/serde/serde-1.0.0.crate",
        ),
        Err(DownloadRedirectError::InvalidSourcePath)
    );
}

#[test]
fn redirect_locations_are_bounded_before_target_validation() {
    let source = ApiRequestTarget::new("/api/v1/crates/serde/1.0.0/download");
    assert!(source.is_ok());
    let Ok(source) = source else {
        unreachable!("download source fixture construction failed");
    };
    let oversized = alloc::string::String::from_utf8(alloc::vec![
        b'a';
        MAX_DOWNLOAD_REDIRECT_LOCATION_BYTES + 1
    ]);
    assert!(oversized.is_ok());
    let Ok(oversized) = oversized else {
        unreachable!("oversized redirect fixture is not UTF-8");
    };
    assert_eq!(
        DownloadRedirect::new(
            OfficialCratesIoEndpoint::production_api(),
            source,
            &oversized,
        ),
        Err(DownloadRedirectError::LocationTooLong)
    );
}

fn identity(
    scheme: EndpointScheme,
    host: &'static str,
    port: u16,
    base_path: &'static str,
) -> Result<EndpointIdentity<'static>, EndpointIdentityError> {
    EndpointIdentity::new(scheme, host, port, base_path)
}

struct StubTransport(EndpointIdentity<'static>);

impl BoundTransport for StubTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.0)
    }
}
