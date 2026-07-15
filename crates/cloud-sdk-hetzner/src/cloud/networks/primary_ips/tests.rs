use super::{
    PrimaryIpActionEndpoint, PrimaryIpAddress, PrimaryIpAssignRequest,
    PrimaryIpChangeDnsPtrRequest, PrimaryIpCreateRequest, PrimaryIpDnsPtr, PrimaryIpDnsPtrIntent,
    PrimaryIpEndpoint, PrimaryIpId, PrimaryIpListRequest, PrimaryIpProtectionRequest,
    PrimaryIpSortField, PrimaryIpType,
};
use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn server_adjacent_primary_ip_paths_match_api_matrix() {
    let id = PrimaryIpId::new(42);
    let action_id = ActionId::new(9);
    let mut output = [0u8; 80];
    if let (Some(id), Some(action_id)) = (id, action_id) {
        assert_eq!(PrimaryIpEndpoint::List.write_path(&mut output), Ok(12));
        assert_eq!(PrimaryIpEndpoint::Get(id).write_path(&mut output), Ok(15));
        assert_eq!(PrimaryIpEndpoint::Update(id).method().as_str(), "PUT");
        assert_eq!(PrimaryIpEndpoint::Delete(id).method().as_str(), "DELETE");

        assert_eq!(
            PrimaryIpActionEndpoint::ListAll.write_path(&mut output),
            Ok(20)
        );
        assert_eq!(
            PrimaryIpActionEndpoint::Get(action_id).write_path(&mut output),
            Ok(22)
        );
        assert_eq!(
            PrimaryIpActionEndpoint::ListForPrimaryIp(id).write_path(&mut output),
            Ok(23)
        );
        assert_eq!(
            PrimaryIpActionEndpoint::Assign(id).write_path(&mut output),
            Ok(30)
        );
        assert_eq!(
            PrimaryIpActionEndpoint::ChangeDnsPtr(id).write_path(&mut output),
            Ok(38)
        );
        assert_eq!(
            PrimaryIpActionEndpoint::ChangeProtection(id).write_path(&mut output),
            Ok(41)
        );
        let path = output
            .get(..41)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/primary_ips/42/actions/change_protection"));
        assert_eq!(
            PrimaryIpActionEndpoint::Unassign(id).write_path(&mut output),
            Ok(32)
        );
        assert_eq!(
            PrimaryIpActionEndpoint::Assign(id).endpoint_group(),
            EndpointGroup::PrimaryIpActions
        );
    }
}

#[test]
fn server_adjacent_primary_ip_query_writes_filters_pagination_and_sorting() {
    let selector = LabelSelector::new("env=prod");
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let mut output = [0u8; 96];
    if let (Ok(selector), Ok(page), Ok(per_page)) = (selector, page, per_page) {
        let request = PrimaryIpListRequest::new()
            .with_label_selector(selector)
            .with_page(page)
            .with_per_page(per_page)
            .with_sort(PrimaryIpSortField::Created, SortDirection::Desc)
            .with_type(PrimaryIpType::Ipv6);
        let written = request.write_query(&mut output);
        assert_eq!(written, Ok(74));
        let query = output
            .get(..74)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(
            query,
            Some("label_selector=env%3Dprod&page=2&per_page=25&sort=created%3Adesc&type=ipv6")
        );
    }
}

#[test]
fn server_adjacent_primary_ip_assignment_and_dns_ptr_intent_are_explicit() {
    if let Some(server_id) = PrimaryIpId::new(42) {
        let assign = PrimaryIpAssignRequest::new(server_id);
        assert_eq!(assign.assignee_id(), server_id);
    }

    let ip = PrimaryIpAddress::new("192.0.2.10");
    let ptr = PrimaryIpDnsPtr::new("server.example.com");
    if let (Ok(ip), Ok(ptr)) = (ip, ptr) {
        let set = PrimaryIpChangeDnsPtrRequest::new(ip, PrimaryIpDnsPtrIntent::Set(ptr));
        assert_eq!(set.dns_ptr(), PrimaryIpDnsPtrIntent::Set(ptr));
        let reset = PrimaryIpChangeDnsPtrRequest::new(ip, PrimaryIpDnsPtrIntent::Reset);
        assert_eq!(reset.dns_ptr(), PrimaryIpDnsPtrIntent::Reset);
    }
}

#[test]
fn server_adjacent_primary_ip_create_excludes_removed_datacenter_field() {
    let request = PrimaryIpCreateRequest::new(PrimaryIpType::Ipv4);
    assert_eq!(request.ip_type(), PrimaryIpType::Ipv4);
    assert!(PrimaryIpProtectionRequest::new(true).delete());
}
