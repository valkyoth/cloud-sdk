use super::{
    NetworkCreateRequest, NetworkEndpoint, NetworkId, NetworkLabels, NetworkListRequest,
    NetworkName, NetworkRequestError, NetworkRoute, NetworkSortField, NetworkSubnet,
    NetworkSubnetType, NetworkVswitchId, NetworkZone,
};
use crate::actions::ActionId;
use crate::cloud::ip::{NetworkIpRange, RouteDestination, RouteGateway, SubnetIpRange};
use crate::cloud::networks::actions::{
    NetworkActionEndpoint, NetworkAddSubnetRequest, NetworkChangeIpRangeRequest,
    NetworkDeleteSubnetRequest, NetworkRouteRequest,
};
use crate::labels::{LabelKey, LabelSelector, LabelValue};
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn networks_firewalls_network_paths_match_source_lock() {
    let id = NetworkId::new(42);
    assert!(id.is_some());
    let Some(id) = id else { return };
    let action = ActionId::new(9);
    assert!(action.is_some());
    let Some(action) = action else { return };
    let mut output = [0u8; 80];
    assert_eq!(NetworkEndpoint::List.write_path(&mut output), Ok(9));
    assert_eq!(NetworkEndpoint::Get(id).write_path(&mut output), Ok(12));
    assert_eq!(
        NetworkActionEndpoint::ListAll.write_path(&mut output),
        Ok(17)
    );
    assert_eq!(
        NetworkActionEndpoint::Get(action).write_path(&mut output),
        Ok(19)
    );
    assert_eq!(
        NetworkActionEndpoint::ListForNetwork(id).write_path(&mut output),
        Ok(20)
    );
    assert_eq!(
        NetworkActionEndpoint::AddRoute(id).write_path(&mut output),
        Ok(30)
    );
    assert_eq!(
        NetworkActionEndpoint::AddSubnet(id).write_path(&mut output),
        Ok(31)
    );
    assert_eq!(
        NetworkActionEndpoint::ChangeIpRange(id).write_path(&mut output),
        Ok(36)
    );
    assert_eq!(
        NetworkActionEndpoint::ChangeProtection(id).write_path(&mut output),
        Ok(38)
    );
    assert_eq!(
        NetworkActionEndpoint::DeleteRoute(id).write_path(&mut output),
        Ok(33)
    );
    assert_eq!(
        NetworkActionEndpoint::DeleteSubnet(id).write_path(&mut output),
        Ok(34)
    );
}

#[test]
fn networks_firewalls_network_query_includes_labels() {
    let Ok(name) = NetworkName::new("private") else {
        return;
    };
    let Ok(selector) = LabelSelector::new("env=prod") else {
        return;
    };
    let Ok(page) = Page::new(2) else { return };
    let Ok(per_page) = PerPage::new(25) else {
        return;
    };
    let request = NetworkListRequest::new()
        .with_name(name)
        .with_label_selector(selector)
        .with_page(page, per_page)
        .with_sort(NetworkSortField::Name, SortDirection::Asc);
    let mut output = [0u8; 128];
    let written = request.write_query(&mut output);
    assert_eq!(written, Ok(73));
    let query = output
        .get(..73)
        .and_then(|bytes| core::str::from_utf8(bytes).ok());
    assert_eq!(
        query,
        Some("label_selector=env%3Dprod&name=private&page=2&per_page=25&sort=name%3Aasc")
    );
}

#[test]
fn networks_firewalls_subnet_and_route_markers_enforce_shape() {
    let Ok(zone) = NetworkZone::new("eu-central") else {
        return;
    };
    let Ok(subnet_range) = SubnetIpRange::new("10.0.1.0/24") else {
        return;
    };
    let cloud = NetworkSubnet::cloud(zone, Some(subnet_range));
    assert_eq!(cloud.subnet_type(), NetworkSubnetType::Cloud);
    assert_eq!(cloud.vswitch_id(), None);

    assert_eq!(NetworkVswitchId::new(0), None);
    let Some(vswitch_id) = NetworkVswitchId::new(1000) else {
        return;
    };
    let vswitch = NetworkSubnet::vswitch(zone, Some(subnet_range), vswitch_id);
    assert_eq!(vswitch.vswitch_id(), Some(vswitch_id));

    let Ok(destination) = RouteDestination::new("10.100.1.0/24") else {
        return;
    };
    let Ok(gateway) = RouteGateway::new("10.0.1.1") else {
        return;
    };
    let route = NetworkRoute::new(destination, gateway);
    assert_eq!(
        NetworkRouteRequest::try_new(Some(route)).map(NetworkRouteRequest::route),
        Ok(route)
    );
    assert_eq!(
        NetworkAddSubnetRequest::try_new(Some(vswitch)).map(NetworkAddSubnetRequest::subnet),
        Ok(vswitch)
    );
}

#[test]
fn networks_firewalls_network_required_fields_are_explicit() {
    let Ok(name) = NetworkName::new("private") else {
        return;
    };
    let Ok(ip_range) = NetworkIpRange::new("10.0.0.0/16") else {
        return;
    };
    assert_eq!(
        NetworkCreateRequest::try_new(None, Some(ip_range)),
        Err(NetworkRequestError::MissingRequiredField)
    );
    assert_eq!(
        NetworkCreateRequest::try_new(Some(name), None),
        Err(NetworkRequestError::MissingRequiredField)
    );
    assert_eq!(
        NetworkDeleteSubnetRequest::try_new(None),
        Err(NetworkRequestError::MissingRequiredField)
    );
    assert_eq!(
        NetworkChangeIpRangeRequest::try_new(None),
        Err(NetworkRequestError::MissingRequiredField)
    );
    assert_eq!(
        NetworkCreateRequest::try_new(Some(name), Some(ip_range))
            .map(NetworkCreateRequest::ip_range),
        Ok(ip_range)
    );

    let key = LabelKey::new("env");
    let value = LabelValue::new("prod");
    assert!(key.is_ok() && value.is_ok(), "fixture label must validate");
    let (Ok(key), Ok(value)) = (key, value) else {
        return;
    };
    let entries = [(key, value)];
    let labels = NetworkLabels::new(&entries);
    assert!(labels.is_ok(), "fixture labels must validate");
    let Ok(labels) = labels else { return };
    let request = NetworkCreateRequest::try_new(Some(name), Some(ip_range))
        .map(|request| request.with_labels(labels));
    assert_eq!(request.map(NetworkCreateRequest::labels), Ok(Some(labels)));
}
