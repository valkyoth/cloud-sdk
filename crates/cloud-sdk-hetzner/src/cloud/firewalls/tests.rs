use super::actions::{FirewallActionEndpoint, FirewallResourcesRequest};
use super::rules::{
    FirewallDescription, FirewallPort, FirewallProtocol, FirewallRule, FirewallRuleError,
    FirewallRuleSet, FirewallSelectors,
};
use super::{
    FirewallCreateRequest, FirewallEndpoint, FirewallId, FirewallLabels, FirewallListRequest,
    FirewallName, FirewallResource, FirewallSortField,
};
use crate::actions::ActionId;
use crate::cloud::ip::IpCidr;
use crate::labels::{LabelKey, LabelSelector, LabelValue};
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn networks_firewalls_paths_match_source_lock() {
    let id = FirewallId::new(42);
    assert!(id.is_some(), "fixture Firewall ID must validate");
    let Some(id) = id else { return };
    let action = ActionId::new(9);
    assert!(action.is_some(), "fixture action ID must validate");
    let Some(action) = action else { return };
    let mut output = [0u8; 80];
    assert_eq!(FirewallEndpoint::List.write_path(&mut output), Ok(10));
    assert_eq!(FirewallEndpoint::Get(id).write_path(&mut output), Ok(13));
    assert_eq!(
        FirewallActionEndpoint::ListAll.write_path(&mut output),
        Ok(18)
    );
    assert_eq!(
        FirewallActionEndpoint::Get(action).write_path(&mut output),
        Ok(20)
    );
    assert_eq!(
        FirewallActionEndpoint::ListForFirewall(id).write_path(&mut output),
        Ok(21)
    );
    assert_eq!(
        FirewallActionEndpoint::ApplyToResources(id).write_path(&mut output),
        Ok(40)
    );
    assert_eq!(
        FirewallActionEndpoint::RemoveFromResources(id).write_path(&mut output),
        Ok(43)
    );
    assert_eq!(
        FirewallActionEndpoint::SetRules(id).write_path(&mut output),
        Ok(31)
    );
}

#[test]
fn networks_firewalls_list_query_includes_labels() {
    let name = FirewallName::new("edge");
    assert!(name.is_ok(), "fixture name must validate");
    let Ok(name) = name else { return };
    let selector = LabelSelector::new("env=prod");
    assert!(selector.is_ok(), "fixture selector must validate");
    let Ok(selector) = selector else { return };
    let page = Page::new(2);
    assert!(page.is_ok(), "fixture page must validate");
    let Ok(page) = page else { return };
    let per_page = PerPage::new(25);
    assert!(per_page.is_ok(), "fixture page size must validate");
    let Ok(per_page) = per_page else { return };
    let request = FirewallListRequest::new()
        .with_name(name)
        .with_label_selector(selector)
        .with_page(page, per_page)
        .with_sort(FirewallSortField::Created, SortDirection::Desc);
    let mut output = [0u8; 128];
    let written = request.write_query(&mut output);
    assert_eq!(written, Ok(74));
    let query = output
        .get(..74)
        .and_then(|bytes| core::str::from_utf8(bytes).ok());
    assert_eq!(
        query,
        Some("label_selector=env%3Dprod&name=edge&page=2&per_page=25&sort=created%3Adesc")
    );
}

#[test]
fn networks_firewalls_rules_reject_conflicts_and_bad_ports() {
    let anywhere = IpCidr::new("0.0.0.0/0");
    assert!(anywhere.is_ok(), "fixture CIDR must validate");
    let Ok(anywhere) = anywhere else { return };
    let cidrs = [anywhere];
    let incoming = FirewallSelectors::incoming(&cidrs);
    assert!(incoming.is_ok(), "fixture selectors must validate");
    let Ok(incoming) = incoming else { return };
    assert_eq!(FirewallPort::new("0"), Err(FirewallRuleError::InvalidPort));
    assert_eq!(
        FirewallPort::new("65536"),
        Err(FirewallRuleError::InvalidPort)
    );
    assert_eq!(
        FirewallPort::new("443-80"),
        Err(FirewallRuleError::InvalidPort)
    );
    let port = FirewallPort::new("80-443");
    assert!(port.is_ok(), "fixture port must validate");
    let Ok(port) = port else { return };
    assert_eq!(port.bounds(), (80, 443));
    assert_eq!(
        FirewallRule::try_new(incoming, FirewallProtocol::Icmp, Some(port)),
        Err(FirewallRuleError::PortProtocolConflict)
    );
    let rule = FirewallRule::try_new(incoming, FirewallProtocol::Tcp, Some(port));
    assert!(rule.is_ok(), "fixture rule must validate");
    let Ok(rule) = rule else { return };
    let duplicate_rules = [rule, rule];
    assert_eq!(
        FirewallRuleSet::new(&duplicate_rules),
        Err(FirewallRuleError::DuplicateRule)
    );
    let too_many_rules = [rule; 51];
    assert_eq!(
        FirewallRuleSet::new(&too_many_rules),
        Err(FirewallRuleError::TooManyRules)
    );
    let too_many_cidrs = [anywhere; 101];
    assert_eq!(
        FirewallSelectors::outgoing(&too_many_cidrs),
        Err(FirewallRuleError::TooManyCidrs)
    );
}

#[test]
fn networks_firewalls_required_fields_and_bodies_are_explicit() {
    assert!(FirewallDescription::new("allow application traffic").is_ok());
    assert_eq!(
        FirewallDescription::new("bad\ntext"),
        Err(FirewallRuleError::InvalidDescription)
    );

    let server = FirewallId::new(7);
    assert!(server.is_some(), "fixture server ID must validate");
    let Some(server) = server else { return };
    let resources = [FirewallResource::Server(server)];
    assert_eq!(FirewallResourcesRequest::new(&resources).resources().len(), 1);

    let key = LabelKey::new("env");
    let value = LabelValue::new("prod");
    assert!(key.is_ok() && value.is_ok(), "fixture label must validate");
    let (Ok(key), Ok(value)) = (key, value) else {
        return;
    };
    let entries = [(key, value)];
    let labels = FirewallLabels::new(&entries);
    assert!(labels.is_ok(), "fixture labels must validate");
    let Ok(labels) = labels else { return };
    let name = FirewallName::new("edge");
    assert!(name.is_ok(), "fixture name must validate");
    let Ok(name) = name else { return };
    let request = FirewallCreateRequest::new(name)
        .with_labels(labels)
        .with_resources(&resources);
    assert_eq!(request.labels(), Some(labels));
}
