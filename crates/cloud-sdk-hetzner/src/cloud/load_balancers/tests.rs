use super::*;

macro_rules! some {
    ($value:expr) => {{
        let Some(value) = $value else { return };
        value
    }};
}

macro_rules! valid {
    ($value:expr) => {{
        let Ok(value) = $value else { return };
        value
    }};
}

macro_rules! id {
    ($value:expr) => {
        some!(LoadBalancerId::new($value))
    };
}

macro_rules! port {
    ($value:expr) => {
        valid!(LoadBalancerPort::new($value))
    };
}

#[test]
fn load_balancers_resource_paths_and_methods_are_source_locked() {
    let cases = [
        (LoadBalancerEndpoint::List, Method::Get, "/load_balancers"),
        (
            LoadBalancerEndpoint::Create,
            Method::Post,
            "/load_balancers",
        ),
        (
            LoadBalancerEndpoint::Get(id!(42)),
            Method::Get,
            "/load_balancers/42",
        ),
        (
            LoadBalancerEndpoint::Update(id!(42)),
            Method::Put,
            "/load_balancers/42",
        ),
        (
            LoadBalancerEndpoint::Delete(id!(42)),
            Method::Delete,
            "/load_balancers/42",
        ),
        (
            LoadBalancerEndpoint::Metrics(id!(42)),
            Method::Get,
            "/load_balancers/42/metrics",
        ),
    ];
    for (endpoint, method, expected) in cases {
        let mut output = [0_u8; 64];
        let len = valid!(endpoint.write_path(&mut output));
        assert_eq!(endpoint.method(), method);
        assert_eq!(output.get(..len), Some(expected.as_bytes()));
    }
}

#[test]
fn load_balancers_action_paths_cover_nondeprecated_operations() {
    let cases = [
        (
            LoadBalancerActionEndpoint::ListAll,
            "/load_balancers/actions",
        ),
        (
            LoadBalancerActionEndpoint::Get(id!(7)),
            "/load_balancers/actions/7",
        ),
        (
            LoadBalancerActionEndpoint::ListForLoadBalancer(id!(7)),
            "/load_balancers/7/actions",
        ),
        (
            LoadBalancerActionEndpoint::AddService(id!(7)),
            "/load_balancers/7/actions/add_service",
        ),
        (
            LoadBalancerActionEndpoint::AddTarget(id!(7)),
            "/load_balancers/7/actions/add_target",
        ),
        (
            LoadBalancerActionEndpoint::AttachToNetwork(id!(7)),
            "/load_balancers/7/actions/attach_to_network",
        ),
        (
            LoadBalancerActionEndpoint::ChangeAlgorithm(id!(7)),
            "/load_balancers/7/actions/change_algorithm",
        ),
        (
            LoadBalancerActionEndpoint::ChangeDnsPtr(id!(7)),
            "/load_balancers/7/actions/change_dns_ptr",
        ),
        (
            LoadBalancerActionEndpoint::ChangeProtection(id!(7)),
            "/load_balancers/7/actions/change_protection",
        ),
        (
            LoadBalancerActionEndpoint::ChangeType(id!(7)),
            "/load_balancers/7/actions/change_type",
        ),
        (
            LoadBalancerActionEndpoint::DeleteService(id!(7)),
            "/load_balancers/7/actions/delete_service",
        ),
        (
            LoadBalancerActionEndpoint::DetachFromNetwork(id!(7)),
            "/load_balancers/7/actions/detach_from_network",
        ),
        (
            LoadBalancerActionEndpoint::DisablePublicInterface(id!(7)),
            "/load_balancers/7/actions/disable_public_interface",
        ),
        (
            LoadBalancerActionEndpoint::EnablePublicInterface(id!(7)),
            "/load_balancers/7/actions/enable_public_interface",
        ),
        (
            LoadBalancerActionEndpoint::RemoveTarget(id!(7)),
            "/load_balancers/7/actions/remove_target",
        ),
        (
            LoadBalancerActionEndpoint::UpdateService(id!(7)),
            "/load_balancers/7/actions/update_service",
        ),
    ];
    for (endpoint, expected) in cases {
        let mut output = [0_u8; 80];
        let len = valid!(endpoint.write_path(&mut output));
        assert_eq!(output.get(..len), Some(expected.as_bytes()));
    }
}

#[test]
fn load_balancers_paths_report_small_buffers() {
    let mut output = [0_u8; 4];
    assert!(matches!(
        LoadBalancerEndpoint::Get(id!(1)).write_path(&mut output),
        Err(LoadBalancerRequestError::Cloud(
            crate::cloud::shared::CloudRequestError::PathBufferTooSmall
        ))
    ));
}

#[test]
fn load_balancers_list_query_is_encoded_and_deterministic() {
    let name = valid!(LoadBalancerName::new("public edge"));
    let selector = valid!(crate::labels::LabelSelector::new("env=prod"));
    let page = valid!(crate::pagination::Page::new(2));
    let per_page = valid!(crate::pagination::PerPage::new(50));
    let request = LoadBalancerListRequest::new()
        .with_name(name)
        .with_label_selector(selector)
        .with_page(page, per_page)
        .with_sort(
            LoadBalancerSortField::Created,
            crate::pagination::SortDirection::Desc,
        );
    let mut output = [0_u8; 128];
    let len = valid!(request.write_query(&mut output));
    assert_eq!(
        output.get(..len),
        Some(
            b"label_selector=env%3Dprod&name=public%20edge&page=2&per_page=50&sort=created%3Adesc"
                .as_slice()
        )
    );
}

#[test]
fn load_balancers_name_rejects_boundary_whitespace_and_controls() {
    assert_eq!(
        LoadBalancerName::new(" edge"),
        Err(LoadBalancerRequestError::InvalidText)
    );
    assert_eq!(
        LoadBalancerName::new("edge "),
        Err(LoadBalancerRequestError::InvalidText)
    );
    assert_eq!(
        LoadBalancerName::new("edge\nname"),
        Err(LoadBalancerRequestError::InvalidText)
    );
    assert!(LoadBalancerName::new("edge name").is_ok());
}

#[test]
fn load_balancers_create_requires_name_and_type_and_preserves_intent() {
    let name = valid!(LoadBalancerName::new("edge"));
    let ty = valid!(LoadBalancerType::new("lb11"));
    assert_eq!(
        LoadBalancerCreateRequest::try_new(None, Some(ty)),
        Err(LoadBalancerRequestError::MissingRequiredField)
    );
    let request = valid!(LoadBalancerCreateRequest::try_new(Some(name), Some(ty)))
        .with_algorithm(LoadBalancerAlgorithm::LeastConnections)
        .with_public_interface(false)
        .with_placement(LoadBalancerPlacement::NetworkZone(valid!(
            LoadBalancerNetworkZone::new("eu-central")
        )))
        .with_services(&[])
        .with_targets(&[]);
    assert_eq!(request.public_interface(), Some(false));
    assert_eq!(request.services(), Some([].as_slice()));
    assert_eq!(request.targets(), Some([].as_slice()));
}

#[test]
fn load_balancers_health_check_bounds_are_enforced() {
    assert!(HealthCheckSettings::new(port!(80), 3, 1, 1).is_ok());
    assert_eq!(
        HealthCheckSettings::new(port!(80), 2, 1, 1),
        Err(LoadBalancerRequestError::InvalidHealthCheck)
    );
    assert_eq!(
        HealthCheckSettings::new(port!(80), 60, 60, 5).map(HealthCheckSettings::timeout),
        Ok(60)
    );
    assert_eq!(
        HealthCheckSettings::new(port!(80), 61, 1, 1),
        Err(LoadBalancerRequestError::InvalidHealthCheck)
    );
}

#[test]
fn load_balancers_http_health_check_requires_path_and_limits_status_codes() {
    assert_eq!(
        HttpHealthCheck::try_new(None, None),
        Err(LoadBalancerRequestError::MissingRequiredField)
    );
    let path = valid!(HealthCheckPath::new("/ready"));
    assert_eq!(
        HealthCheckPath::new("/not ready"),
        Err(LoadBalancerRequestError::InvalidText)
    );
    assert_eq!(
        HttpHealthCheck::try_new(Some("bad host"), Some(path)),
        Err(LoadBalancerRequestError::InvalidText)
    );
    let code = valid!(HealthCheckStatusCode::new("2??"));
    let statuses = [code; 21];
    assert_eq!(
        valid!(HttpHealthCheck::try_new(None, Some(path))).with_status_codes(&statuses),
        Err(LoadBalancerRequestError::TooManyItems)
    );
}

#[test]
fn load_balancers_http_service_limits_are_enforced() {
    let cookie = valid!(StickyCookieName::new("HCLBSTICKY"));
    assert_eq!(
        HttpServiceConfig::new().with_cookie(cookie, 29),
        Err(LoadBalancerRequestError::InvalidServiceConfiguration)
    );
    assert!(HttpServiceConfig::new().with_cookie(cookie, 86_400).is_ok());
    assert_eq!(
        HttpServiceConfig::new().with_idle_timeout(301),
        Err(LoadBalancerRequestError::InvalidServiceConfiguration)
    );
}

#[test]
fn load_balancers_service_protocols_carry_only_compatible_settings() {
    let certificate = id!(17);
    let certificates = [certificate];
    let http = HttpServiceConfig::new().with_sticky_sessions(true);
    let https = HttpsServiceConfig::new(http)
        .with_certificates(&certificates)
        .with_redirect_http(true);
    let settings = valid!(HealthCheckSettings::new(port!(443), 15, 10, 3));
    let service = LoadBalancerService::new(
        LoadBalancerServiceProtocol::Https(https),
        port!(443),
        port!(8443),
        false,
        LoadBalancerHealthCheck::Tcp(settings),
    );

    assert_eq!(
        service.protocol(),
        LoadBalancerServiceProtocol::Https(https)
    );
    assert_eq!(https.certificates(), Some(certificates.as_slice()));
    assert!(https.redirect_http());
    assert_eq!(service.destination_port().get(), 8443);
}

#[test]
fn load_balancers_network_actions_preserve_one_address_mode() {
    let network = id!(4);
    let ip = valid!(LoadBalancerIp::new("10.0.1.10"));
    let range = valid!(crate::cloud::ip::SubnetIpRange::new("10.0.1.0/24"));
    let request = LoadBalancerAttachNetworkRequest::new(network).with_address(
        LoadBalancerNetworkAddress::IpInRange {
            ip,
            ip_range: range,
        },
    );

    assert_eq!(request.network(), network);
    assert!(matches!(
        request.address(),
        Some(LoadBalancerNetworkAddress::IpInRange { .. })
    ));
    assert_eq!(
        LoadBalancerDetachNetworkRequest::new(network).network(),
        network
    );
}

#[test]
fn load_balancers_target_selection_prevents_conflicting_private_ip_intent() {
    let direct = LoadBalancerTarget::Ip(valid!(LoadBalancerIp::new("10.0.0.4")));
    assert_eq!(
        LoadBalancerAddTargetRequest::try_new(direct, true),
        Err(LoadBalancerRequestError::InvalidTargetConfiguration)
    );
    let public = valid!(LoadBalancerPublicIp::new("8.8.8.8"));
    let server = LoadBalancerTarget::Server {
        id: id!(9),
        public_ip: Some(public),
    };
    assert_eq!(
        LoadBalancerAddTargetRequest::try_new(server, true),
        Err(LoadBalancerRequestError::InvalidTargetConfiguration)
    );
    let server = LoadBalancerTarget::Server {
        id: id!(9),
        public_ip: None,
    };
    assert!(LoadBalancerAddTargetRequest::try_new(server, true).is_ok());
}

#[test]
fn load_balancers_public_server_ip_rejects_private_and_special_addresses() {
    for value in [
        "10.0.0.1",
        "127.0.0.1",
        "169.254.1.1",
        "100.64.0.1",
        "192.0.2.1",
        "198.18.0.1",
        "203.0.113.1",
        "::1",
        "fe80::1",
        "fc00::1",
        "2001:db8::1",
        "::ffff:10.0.0.1",
        "::10.0.0.1",
        "::ffff:0:10.0.0.1",
        "64:ff9b::c000:201",
        "fec0::1",
        "2001:0:4136:e378:8000:63bf:3fff:fdd2",
        "2001:2::1",
        "2001:10::1",
        "2002:c000:0201::",
        "3fff::1",
        "2200::1",
        "3000::1",
        "3ffe::1",
        "2620:4f:8000::1",
    ] {
        assert_eq!(
            LoadBalancerPublicIp::new(value),
            Err(LoadBalancerRequestError::InvalidTargetConfiguration)
        );
    }
    assert!(LoadBalancerPublicIp::new("2606:4700:4700::1111").is_ok());
    assert!(LoadBalancerPublicIp::new("2001:4860:4860::8888").is_ok());
    assert!(LoadBalancerPublicIp::new("2a01:4f8:1c1c::1").is_ok());
}

#[test]
fn load_balancers_dns_pointer_requires_explicit_set_or_reset() {
    let ip = valid!(LoadBalancerIp::new("2001:db8::1"));
    assert_eq!(
        LoadBalancerChangeDnsPtrRequest::try_new(ip, None),
        Err(LoadBalancerRequestError::MissingDnsPtrIntent)
    );
    let reset = valid!(LoadBalancerChangeDnsPtrRequest::try_new(
        ip,
        Some(LoadBalancerDnsPtrIntent::Reset)
    ));
    assert_eq!(reset.dns_ptr(), LoadBalancerDnsPtrIntent::Reset);
    assert_eq!(
        LoadBalancerDnsPtr::new("-bad.example"),
        Err(LoadBalancerRequestError::InvalidText)
    );
}

#[test]
fn load_balancers_metrics_require_valid_increasing_range() {
    assert_eq!(
        LoadBalancerTimestamp::new("2025-02-29T00:00:00Z"),
        Err(LoadBalancerRequestError::InvalidText)
    );
    let start = valid!(LoadBalancerTimestamp::new("2024-02-29T00:00:00Z"));
    let end = valid!(LoadBalancerTimestamp::new("2024-02-29T01:00:00Z"));
    let metrics = LoadBalancerMetricTypes::new(LoadBalancerMetricType::Bandwidth)
        .with(LoadBalancerMetricType::OpenConnections)
        .with(LoadBalancerMetricType::Bandwidth);
    assert_eq!(metrics.as_api_str(), "open_connections,bandwidth");
    assert_eq!(
        LoadBalancerMetricsRequest::try_new(id!(1), metrics, end, start),
        Err(LoadBalancerRequestError::InvalidTimeRange)
    );
    let request = valid!(LoadBalancerMetricsRequest::try_new(
        id!(1),
        metrics,
        start,
        end
    ))
    .with_step(valid!(LoadBalancerMetricsStep::new(60)));
    let mut output = [0_u8; 192];
    let len = valid!(request.write_query(&mut output));
    assert_eq!(
        output.get(..len),
        Some(b"end=2024-02-29T01%3A00%3A00Z&step=60&start=2024-02-29T00%3A00%3A00Z&type=open_connections%2Cbandwidth".as_slice())
    );
    assert_eq!(
        request.write_query(&mut [0_u8; 8]),
        Err(LoadBalancerRequestError::Cloud(
            crate::cloud::shared::CloudRequestError::QueryBufferTooSmall
        ))
    );
}

use cloud_sdk::Method;
