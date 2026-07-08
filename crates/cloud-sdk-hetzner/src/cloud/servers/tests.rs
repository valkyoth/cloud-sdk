use super::actions::{
    DnsPtrIntent, ServerActionEndpoint, ServerActionKind, ServerActionRequest, ServerImageType,
};
use super::{
    PrimaryIpSelection, ServerCreateRequest, ServerEndpoint, ServerId, ServerListRequest,
    ServerMetricType, ServerMetricsRequest, ServerName, ServerPublicNet, ServerReference,
    ServerRequestError, ServerResourceId, ServerSortField, ServerStatus, TextValue, TimestampValue,
    UserData,
};
use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

#[test]
fn server_paths_match_api_matrix() {
    let id = ServerId::new(42);
    let mut output = [0u8; 64];
    if let Some(id) = id {
        assert_eq!(ServerEndpoint::List.write_path(&mut output), Ok(8));
        assert_eq!(ServerEndpoint::Get(id).write_path(&mut output), Ok(11));
        assert_eq!(ServerEndpoint::Metrics(id).write_path(&mut output), Ok(19));
        let path = output
            .get(..19)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/servers/42/metrics"));
        assert_eq!(ServerEndpoint::Delete(id).method().as_str(), "DELETE");
        assert_eq!(ServerEndpoint::Create.api_base_url(), ApiBaseUrl::CloudV1);
        assert_eq!(
            ServerEndpoint::Create.endpoint_group(),
            EndpointGroup::Servers
        );
    }
}

#[test]
fn server_list_query_writes_filters_pagination_and_sorting() {
    let name = ServerName::new("web-1");
    let selector = LabelSelector::new("env=prod");
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let mut output = [0u8; 160];
    if let (Ok(name), Ok(selector), Ok(page), Ok(per_page)) = (name, selector, page, per_page) {
        let request = ServerListRequest::new()
            .with_name(name)
            .with_label_selector(selector)
            .with_status(ServerStatus::Running)
            .with_page(page)
            .with_per_page(per_page)
            .with_sort(ServerSortField::Created, SortDirection::Desc);
        let written = request.write_query(&mut output);
        assert_eq!(written, Ok(90));
        let query = output
            .get(..90)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(
            query,
            Some(
                "label_selector=env%3Dprod&name=web-1&page=2&per_page=25&sort=created%3Adesc&status=running"
            )
        );
    }
}

#[test]
fn server_create_validates_required_fields_and_mutual_exclusions() {
    let name = ServerName::new("web-1");
    let server_type = ServerReference::new("cpx22");
    let image = ServerReference::new("ubuntu-24.04");
    if let (Ok(name), Ok(server_type), Ok(image)) = (name, server_type, image) {
        assert_eq!(
            ServerCreateRequest::try_new(None, Some(server_type), Some(image)),
            Err(ServerRequestError::MissingRequiredField)
        );
        let request = ServerCreateRequest::try_new(Some(name), Some(server_type), Some(image));
        assert!(request.is_ok());
        let user_data = UserData::new("#cloud-config\n");
        if let (Ok(request), Ok(user_data)) = (request, user_data) {
            assert_eq!(
                request.with_user_data(user_data).endpoint(),
                ServerEndpoint::Create
            );
        }
    }
    let primary_ip = ServerResourceId::new(7);
    if let Some(primary_ip) = primary_ip {
        assert_eq!(
            ServerPublicNet::new(
                false,
                true,
                PrimaryIpSelection::Id(primary_ip),
                PrimaryIpSelection::Auto,
            ),
            Err(ServerRequestError::MutuallyExclusiveFields)
        );
    }
}

#[test]
fn server_metrics_validate_time_range_and_write_query() {
    let id = ServerId::new(42);
    let start = TimestampValue::new("2026-07-08T10:00:00Z");
    let end = TimestampValue::new("2026-07-08T11:00:00Z");
    if let (Some(id), Ok(start), Ok(end)) = (id, start, end) {
        let request = ServerMetricsRequest::try_new(id, ServerMetricType::Cpu, start, end);
        assert!(request.is_ok());
        if let Ok(request) = request {
            let mut output = [0u8; 128];
            let written = request.write_query(&mut output);
            assert_eq!(written, Ok(68));
            let query = output
                .get(..68)
                .and_then(|bytes| core::str::from_utf8(bytes).ok());
            assert_eq!(
                query,
                Some("end=2026-07-08T11%3A00%3A00Z&start=2026-07-08T10%3A00%3A00Z&type=cpu")
            );
        }
        assert_eq!(
            ServerMetricsRequest::try_new(id, ServerMetricType::Cpu, end, start),
            Err(ServerRequestError::InvalidTimeRange)
        );
    }
}

#[test]
fn server_action_paths_match_api_matrix() {
    let server_id = ServerId::new(42);
    let action_id = ActionId::new(9);
    let mut output = [0u8; 96];
    if let (Some(server_id), Some(action_id)) = (server_id, action_id) {
        assert_eq!(
            ServerActionEndpoint::ListAll.write_path(&mut output),
            Ok(16)
        );
        assert_eq!(
            ServerActionEndpoint::Get(action_id).write_path(&mut output),
            Ok(18)
        );
        assert_eq!(
            ServerActionEndpoint::ListForServer(server_id).write_path(&mut output),
            Ok(19)
        );
        assert_eq!(
            ServerActionEndpoint::Start(server_id, ServerActionKind::ChangeDnsPtr)
                .write_path(&mut output),
            Ok(34)
        );
        let path = output
            .get(..34)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/servers/42/actions/change_dns_ptr"));
        assert_eq!(
            ServerActionEndpoint::Start(server_id, ServerActionKind::Shutdown).endpoint_group(),
            EndpointGroup::ServerActions
        );
    }
}

#[test]
fn all_server_action_start_paths_match_api_matrix() {
    let server_id = ServerId::new(42);
    let cases = [
        (
            ServerActionKind::AddToPlacementGroup,
            "/servers/42/actions/add_to_placement_group",
        ),
        (
            ServerActionKind::AttachIso,
            "/servers/42/actions/attach_iso",
        ),
        (
            ServerActionKind::AttachToNetwork,
            "/servers/42/actions/attach_to_network",
        ),
        (
            ServerActionKind::ChangeAliasIps,
            "/servers/42/actions/change_alias_ips",
        ),
        (
            ServerActionKind::ChangeDnsPtr,
            "/servers/42/actions/change_dns_ptr",
        ),
        (
            ServerActionKind::ChangeProtection,
            "/servers/42/actions/change_protection",
        ),
        (
            ServerActionKind::ChangeType,
            "/servers/42/actions/change_type",
        ),
        (
            ServerActionKind::CreateImage,
            "/servers/42/actions/create_image",
        ),
        (
            ServerActionKind::DetachFromNetwork,
            "/servers/42/actions/detach_from_network",
        ),
        (
            ServerActionKind::DetachIso,
            "/servers/42/actions/detach_iso",
        ),
        (
            ServerActionKind::DisableBackup,
            "/servers/42/actions/disable_backup",
        ),
        (
            ServerActionKind::DisableRescue,
            "/servers/42/actions/disable_rescue",
        ),
        (
            ServerActionKind::EnableBackup,
            "/servers/42/actions/enable_backup",
        ),
        (
            ServerActionKind::EnableRescue,
            "/servers/42/actions/enable_rescue",
        ),
        (ServerActionKind::Poweroff, "/servers/42/actions/poweroff"),
        (ServerActionKind::Poweron, "/servers/42/actions/poweron"),
        (ServerActionKind::Reboot, "/servers/42/actions/reboot"),
        (ServerActionKind::Rebuild, "/servers/42/actions/rebuild"),
        (
            ServerActionKind::RemoveFromPlacementGroup,
            "/servers/42/actions/remove_from_placement_group",
        ),
        (
            ServerActionKind::RequestConsole,
            "/servers/42/actions/request_console",
        ),
        (ServerActionKind::Reset, "/servers/42/actions/reset"),
        (
            ServerActionKind::ResetPassword,
            "/servers/42/actions/reset_password",
        ),
        (ServerActionKind::Shutdown, "/servers/42/actions/shutdown"),
    ];
    if let Some(server_id) = server_id {
        for (kind, expected) in cases {
            let mut output = [0u8; 96];
            let written = ServerActionEndpoint::Start(server_id, kind).write_path(&mut output);
            assert_eq!(written, Ok(expected.len()));
            let path = output
                .get(..expected.len())
                .and_then(|bytes| core::str::from_utf8(bytes).ok());
            assert_eq!(path, Some(expected));
        }
    }
}

#[test]
fn server_actions_validate_body_requirements_and_dns_ptr_intent() {
    let ip = TextValue::new("2001:db8::1");
    if let Ok(ip) = ip {
        assert_eq!(
            ServerActionRequest::change_dns_ptr(ip, None),
            Err(ServerRequestError::MissingDnsPtrIntent)
        );
        assert!(ServerActionRequest::change_dns_ptr(ip, Some(DnsPtrIntent::Reset)).is_ok());
    }
    assert_eq!(
        ServerActionRequest::empty(ServerActionKind::Rebuild),
        Err(ServerRequestError::MissingRequiredField)
    );
    assert!(ServerActionRequest::empty(ServerActionKind::Poweroff).is_ok());
    let network = ServerResourceId::new(7);
    if let Some(network) = network {
        assert_eq!(
            ServerActionRequest::change_alias_ips(network, &[]),
            Err(ServerRequestError::MissingRequiredField)
        );
    }
    assert_eq!(ServerImageType::Snapshot, ServerImageType::Snapshot);
}
