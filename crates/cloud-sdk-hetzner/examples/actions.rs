//! Builds bounded global and certificate action queries without network access.

use cloud_sdk_hetzner::actions::{ActionEndpoint, ActionId, ActionListRequest, ActionStatus};
use cloud_sdk_hetzner::pagination::{Page, PerPage, SortDirection};
use cloud_sdk_hetzner::security::certificates::{
    CertificateActionEndpoint, CertificateActionListForCertificateRequest,
    CertificateActionListRequest, CertificateActionSortField, CertificateId,
};

fn main() {
    let Some(first_action) = ActionId::new(7) else {
        return;
    };
    let Some(second_action) = ActionId::new(42) else {
        return;
    };
    let action_ids = [first_action, second_action];
    let Ok(global_actions) = ActionListRequest::try_new(&action_ids) else {
        return;
    };

    let mut path = [0_u8; 64];
    let Ok(path_len) = ActionEndpoint::Get(second_action).write_path(&mut path) else {
        return;
    };
    assert_eq!(path.get(..path_len), Some(b"/actions/42".as_slice()));

    let mut query = [0_u8; 256];
    let Ok(query_len) = global_actions.write_query(&mut query) else {
        return;
    };
    assert_eq!(query.get(..query_len), Some(b"id=7&id=42".as_slice()));

    let statuses = [ActionStatus::Running, ActionStatus::Error];
    let sorts = [(CertificateActionSortField::Started, SortDirection::Desc)];
    let Ok(certificate_actions) = CertificateActionListRequest::new()
        .with_action_ids(&action_ids)
        .and_then(|request| request.with_statuses(&statuses))
        .and_then(|request| request.with_sorts(&sorts))
    else {
        return;
    };
    let Ok(page) = Page::new(1) else {
        return;
    };
    let Ok(per_page) = PerPage::new(25) else {
        return;
    };
    let certificate_actions = certificate_actions.with_page(page).with_per_page(per_page);
    let Ok(query_len) = certificate_actions.write_query(&mut query) else {
        return;
    };
    assert_eq!(
        query.get(..query_len),
        Some(
            b"id=7&id=42&page=1&per_page=25&sort=started%3Adesc&status=running&status=error"
                .as_slice()
        )
    );

    let Some(certificate_id) = CertificateId::new(100) else {
        return;
    };
    let Ok(certificate_actions) = CertificateActionListForCertificateRequest::new(certificate_id)
        .with_statuses(&statuses)
        .and_then(|request| request.with_sorts(&sorts))
    else {
        return;
    };
    let Ok(path_len) = certificate_actions.endpoint().write_path(&mut path) else {
        return;
    };
    assert_eq!(
        path.get(..path_len),
        Some(b"/certificates/100/actions".as_slice())
    );

    let Ok(path_len) = CertificateActionEndpoint::Get(second_action).write_path(&mut path) else {
        return;
    };
    assert_eq!(
        path.get(..path_len),
        Some(b"/certificates/actions/42".as_slice())
    );
}
