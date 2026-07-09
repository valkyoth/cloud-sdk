use super::{
    FloatingIpActionEndpoint, FloatingIpAddress, FloatingIpAssignRequest,
    FloatingIpChangeDnsPtrRequest, FloatingIpCreatePlacement, FloatingIpCreateRequest,
    FloatingIpDnsPtr, FloatingIpDnsPtrIntent, FloatingIpEndpoint, FloatingIpHomeLocation,
    FloatingIpId, FloatingIpListRequest, FloatingIpName, FloatingIpProtectionRequest,
    FloatingIpRequestError, FloatingIpSortField, FloatingIpType,
};
use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn storage_ip_floating_ip_paths_match_api_matrix() {
    let id = FloatingIpId::new(42);
    let action_id = ActionId::new(9);
    let mut output = [0u8; 88];
    if let (Some(id), Some(action_id)) = (id, action_id) {
        assert_eq!(FloatingIpEndpoint::List.write_path(&mut output), Ok(13));
        assert_eq!(FloatingIpEndpoint::Get(id).write_path(&mut output), Ok(16));
        assert_eq!(FloatingIpEndpoint::Update(id).method().as_str(), "PUT");
        assert_eq!(FloatingIpEndpoint::Delete(id).method().as_str(), "DELETE");
        assert_eq!(
            FloatingIpEndpoint::Create.endpoint_group(),
            EndpointGroup::FloatingIps
        );

        assert_eq!(
            FloatingIpActionEndpoint::ListAll.write_path(&mut output),
            Ok(21)
        );
        assert_eq!(
            FloatingIpActionEndpoint::Get(action_id).write_path(&mut output),
            Ok(23)
        );
        assert_eq!(
            FloatingIpActionEndpoint::ListForFloatingIp(id).write_path(&mut output),
            Ok(24)
        );
        assert_eq!(
            FloatingIpActionEndpoint::Assign(id).write_path(&mut output),
            Ok(31)
        );
        assert_eq!(
            FloatingIpActionEndpoint::ChangeDnsPtr(id).write_path(&mut output),
            Ok(39)
        );
        assert_eq!(
            FloatingIpActionEndpoint::ChangeProtection(id).write_path(&mut output),
            Ok(42)
        );
        let path = output
            .get(..42)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/floating_ips/42/actions/change_protection"));
        assert_eq!(
            FloatingIpActionEndpoint::Unassign(id).write_path(&mut output),
            Ok(33)
        );
    }
}

#[test]
fn storage_ip_floating_ip_query_writes_filters_pagination_and_sorting() {
    let selector = LabelSelector::new("env=prod");
    let name = FloatingIpName::new("edge-ip");
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let mut output = [0u8; 112];
    if let (Ok(selector), Ok(name), Ok(page), Ok(per_page)) = (selector, name, page, per_page) {
        let request = FloatingIpListRequest::new()
            .with_label_selector(selector)
            .with_name(name)
            .with_page(page)
            .with_per_page(per_page)
            .with_sort(FloatingIpSortField::Created, SortDirection::Desc);
        let written = request.write_query(&mut output);
        assert_eq!(written, Ok(77));
        let query = output
            .get(..77)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(
            query,
            Some("label_selector=env%3Dprod&name=edge-ip&page=2&per_page=25&sort=created%3Adesc")
        );
    }
}

#[test]
fn storage_ip_floating_ip_create_selection_is_explicit() {
    assert_eq!(
        FloatingIpCreateRequest::try_new(Some(FloatingIpType::Ipv4), None),
        Err(FloatingIpRequestError::MissingRequiredField)
    );
    let location = FloatingIpHomeLocation::new("fsn1");
    if let Ok(location) = location {
        let request = FloatingIpCreateRequest::try_new(
            Some(FloatingIpType::Ipv6),
            Some(FloatingIpCreatePlacement::HomeLocation(location)),
        );
        assert_eq!(
            request.map(FloatingIpCreateRequest::placement),
            Ok(FloatingIpCreatePlacement::HomeLocation(location))
        );
    }
}

#[test]
fn storage_ip_floating_ip_action_markers_require_required_fields() {
    assert_eq!(
        FloatingIpAssignRequest::try_new(None),
        Err(FloatingIpRequestError::MissingRequiredField)
    );
    if let Some(server) = FloatingIpId::new(42) {
        let assign = FloatingIpAssignRequest::try_new(Some(server));
        assert_eq!(assign.map(FloatingIpAssignRequest::server), Ok(server));
    }

    let ip = FloatingIpAddress::new("2001:db8::1");
    let ptr = FloatingIpDnsPtr::new("server.example.com");
    if let (Ok(ip), Ok(ptr)) = (ip, ptr) {
        assert_eq!(
            FloatingIpChangeDnsPtrRequest::try_new(ip, None),
            Err(FloatingIpRequestError::MissingDnsPtrIntent)
        );
        let set =
            FloatingIpChangeDnsPtrRequest::try_new(ip, Some(FloatingIpDnsPtrIntent::Set(ptr)));
        assert_eq!(
            set.map(FloatingIpChangeDnsPtrRequest::dns_ptr),
            Ok(FloatingIpDnsPtrIntent::Set(ptr))
        );
        let reset = FloatingIpChangeDnsPtrRequest::try_new(ip, Some(FloatingIpDnsPtrIntent::Reset));
        assert_eq!(
            reset.map(FloatingIpChangeDnsPtrRequest::dns_ptr),
            Ok(FloatingIpDnsPtrIntent::Reset)
        );
    }
    assert!(FloatingIpProtectionRequest::new(true).delete());
}
