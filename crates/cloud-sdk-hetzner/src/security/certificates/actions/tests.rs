use cloud_sdk::Method;

use super::{
    CertificateActionEndpoint, CertificateActionListForCertificateRequest,
    CertificateActionListRequest, CertificateActionSortField, MAX_CERTIFICATE_ACTION_IDS,
    MAX_CERTIFICATE_ACTION_SORTS, MAX_CERTIFICATE_ACTION_STATUSES,
};
use crate::EndpointGroup;
use crate::actions::{ActionId, ActionStatus};
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;
use crate::security::certificates::{CertificateId, SecurityRequestError};

#[test]
fn certificate_action_paths_match_source_lock() -> Result<(), SecurityRequestError> {
    let action_id = ActionId::new(42).ok_or(SecurityRequestError::InvalidNameByte)?;
    let certificate_id = CertificateId::new(7).ok_or(SecurityRequestError::InvalidNameByte)?;
    let mut output = [0_u8; 64];

    let cases = [
        (CertificateActionEndpoint::ListAll, "/certificates/actions"),
        (
            CertificateActionEndpoint::Get(action_id),
            "/certificates/actions/42",
        ),
        (
            CertificateActionEndpoint::ListForCertificate(certificate_id),
            "/certificates/7/actions",
        ),
    ];
    for (endpoint, expected) in cases {
        let len = endpoint.write_path(&mut output)?;
        assert_eq!(
            output
                .get(..len)
                .and_then(|bytes| core::str::from_utf8(bytes).ok()),
            Some(expected)
        );
        assert_eq!(endpoint.method(), Method::Get);
        assert_eq!(endpoint.api_base_url(), ApiBaseUrl::CloudV1);
        assert_eq!(endpoint.endpoint_group(), EndpointGroup::CertificateActions);
    }

    let static_path = CertificateActionEndpoint::ListAll
        .static_path()
        .ok_or(SecurityRequestError::InvalidNameByte)??;
    assert_eq!(static_path.as_str(), "/certificates/actions");
    assert!(
        CertificateActionEndpoint::Get(action_id)
            .static_path()
            .is_none()
    );
    Ok(())
}

#[test]
fn global_certificate_action_query_covers_every_filter() -> Result<(), SecurityRequestError> {
    let ids = [
        ActionId::new(7).ok_or(SecurityRequestError::InvalidNameByte)?,
        ActionId::new(42).ok_or(SecurityRequestError::InvalidNameByte)?,
    ];
    let statuses = [ActionStatus::Running, ActionStatus::Error];
    let sorts = [
        (CertificateActionSortField::Started, SortDirection::Desc),
        (CertificateActionSortField::Id, SortDirection::Asc),
    ];
    let page = Page::new(2).map_err(|_| SecurityRequestError::InvalidNameByte)?;
    let per_page = PerPage::new(50).map_err(|_| SecurityRequestError::InvalidNameByte)?;
    let request = CertificateActionListRequest::new()
        .with_action_ids(&ids)?
        .with_statuses(&statuses)?
        .with_sorts(&sorts)?
        .with_page(page)
        .with_per_page(per_page);
    let mut output = [0_u8; 160];
    let len = request.write_query(&mut output)?;

    assert_eq!(request.endpoint(), CertificateActionEndpoint::ListAll);
    assert_eq!(
        output.get(..len),
        Some(
            b"id=7&id=42&page=2&per_page=50&sort=started%3Adesc&sort=id%3Aasc&status=running&status=error"
                .as_slice()
        )
    );
    Ok(())
}

#[test]
fn certificate_local_query_cannot_encode_action_ids() -> Result<(), SecurityRequestError> {
    let certificate_id = CertificateId::new(7).ok_or(SecurityRequestError::InvalidNameByte)?;
    let statuses = [ActionStatus::Success];
    let sorts = [(CertificateActionSortField::Finished, SortDirection::Desc)];
    let page = Page::new(3).map_err(|_| SecurityRequestError::InvalidNameByte)?;
    let per_page = PerPage::new(25).map_err(|_| SecurityRequestError::InvalidNameByte)?;
    let request = CertificateActionListForCertificateRequest::new(certificate_id)
        .with_statuses(&statuses)?
        .with_sorts(&sorts)?
        .with_page(page)
        .with_per_page(per_page);
    let mut output = [0_u8; 128];
    let len = request.write_query(&mut output)?;

    assert_eq!(
        request.endpoint(),
        CertificateActionEndpoint::ListForCertificate(certificate_id)
    );
    assert_eq!(
        output.get(..len),
        Some(b"page=3&per_page=25&sort=finished%3Adesc&status=success".as_slice())
    );
    Ok(())
}

#[test]
fn certificate_action_query_encodes_every_sort_and_status_value() -> Result<(), SecurityRequestError>
{
    let sorts = [
        (CertificateActionSortField::Id, SortDirection::Asc),
        (CertificateActionSortField::Command, SortDirection::Desc),
        (CertificateActionSortField::Status, SortDirection::Asc),
        (CertificateActionSortField::Started, SortDirection::Desc),
        (CertificateActionSortField::Finished, SortDirection::Asc),
    ];
    let statuses = [
        ActionStatus::Running,
        ActionStatus::Success,
        ActionStatus::Error,
    ];
    let request = CertificateActionListRequest::new()
        .with_sorts(&sorts)?
        .with_statuses(&statuses)?;
    let mut output = [0_u8; 256];
    let len = request.write_query(&mut output)?;

    assert_eq!(
        output.get(..len),
        Some(
            b"sort=id%3Aasc&sort=command%3Adesc&sort=status%3Aasc&sort=started%3Adesc&sort=finished%3Aasc&status=running&status=success&status=error"
                .as_slice()
        )
    );
    Ok(())
}

#[test]
fn certificate_action_filters_are_bounded() -> Result<(), SecurityRequestError> {
    let action_id = ActionId::new(1).ok_or(SecurityRequestError::InvalidNameByte)?;
    let ids = [action_id; MAX_CERTIFICATE_ACTION_IDS + 1];
    assert_eq!(
        CertificateActionListRequest::new().with_action_ids(&ids),
        Err(SecurityRequestError::TooManyActionIds)
    );

    let statuses = [ActionStatus::Running; MAX_CERTIFICATE_ACTION_STATUSES + 1];
    assert_eq!(
        CertificateActionListRequest::new().with_statuses(&statuses),
        Err(SecurityRequestError::TooManyActionStatuses)
    );

    let sort = (CertificateActionSortField::Id, SortDirection::Asc);
    let sorts = [sort; MAX_CERTIFICATE_ACTION_SORTS + 1];
    assert_eq!(
        CertificateActionListRequest::new().with_sorts(&sorts),
        Err(SecurityRequestError::TooManyActionSorts)
    );
    Ok(())
}

#[test]
fn certificate_action_paths_and_queries_report_small_buffers() -> Result<(), SecurityRequestError> {
    let action_id = ActionId::new(42).ok_or(SecurityRequestError::InvalidNameByte)?;
    let ids = [action_id];
    let request = CertificateActionListRequest::new().with_action_ids(&ids)?;

    assert_eq!(
        CertificateActionEndpoint::Get(action_id).write_path(&mut [0_u8; 23]),
        Err(SecurityRequestError::PathBufferTooSmall)
    );
    assert_eq!(
        request.write_query(&mut [0_u8; 4]),
        Err(SecurityRequestError::QueryBufferTooSmall)
    );
    Ok(())
}
