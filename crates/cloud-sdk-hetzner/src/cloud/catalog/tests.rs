use super::{
    CatalogGetEndpoint, CatalogId, CatalogListEndpoint, CatalogListRequest, CatalogRequestError,
    CatalogSingletonEndpoint, PublicImageKind, validate_written_path,
};
use crate::EndpointGroup;
use crate::pagination::{Page, PerPage, Sort, SortDirection, SortKey};
use crate::request::{ApiBaseUrl, EndpointPathError};

#[test]
fn catalog_list_paths_match_api_matrix() {
    assert_eq!(
        CatalogListEndpoint::Locations
            .path()
            .map(|path| path.as_str()),
        Ok("/locations")
    );
    assert_eq!(
        CatalogListEndpoint::ServerTypes
            .path()
            .map(|path| path.as_str()),
        Ok("/server_types")
    );
    assert_eq!(
        CatalogListEndpoint::LoadBalancerTypes
            .path()
            .map(|path| path.as_str()),
        Ok("/load_balancer_types")
    );
    assert_eq!(
        CatalogListEndpoint::Isos.path().map(|path| path.as_str()),
        Ok("/isos")
    );
    assert_eq!(
        CatalogListEndpoint::PublicImages(PublicImageKind::System)
            .path()
            .map(|path| path.as_str()),
        Ok("/images")
    );
}

#[test]
fn catalog_get_paths_match_api_matrix() {
    let id = CatalogId::new(42);
    let mut output = [0u8; 64];
    if let Some(id) = id {
        let request = CatalogGetEndpoint::LoadBalancerType(id);
        assert_eq!(request.write_path(&mut output), Ok(23));
        let path = output
            .get(..23)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/load_balancer_types/42"));
    }
}

#[test]
fn catalog_get_path_reports_too_small_buffer() {
    let id = CatalogId::new(42);
    let mut output = [0u8; 4];
    if let Some(id) = id {
        let request = CatalogGetEndpoint::Location(id);
        assert_eq!(
            request.write_path(&mut output),
            Err(CatalogRequestError::PathBufferTooSmall)
        );
    }
}

#[test]
fn catalog_written_path_validation_rejects_invalid_paths() {
    assert_eq!(validate_written_path(b"/images/42", 10), Ok(()));
    assert_eq!(
        validate_written_path(b"/images/../42", 13),
        Err(CatalogRequestError::InvalidPath(
            EndpointPathError::ParentDirectorySegment
        ))
    );
    assert_eq!(
        validate_written_path(b"/images/\xff", 9),
        Err(CatalogRequestError::PathEncodingFailed)
    );
}

#[test]
fn catalog_pricing_path_matches_api_matrix() {
    assert_eq!(
        CatalogSingletonEndpoint::Pricing
            .path()
            .map(|path| path.as_str()),
        Ok("/pricing")
    );
    assert_eq!(
        CatalogSingletonEndpoint::Pricing.endpoint_group(),
        EndpointGroup::Pricing
    );
}

#[test]
fn catalog_list_query_writes_pagination_and_sorting() {
    let page = Page::new(2);
    let per_page = PerPage::new(50);
    let sort_key = SortKey::new("name");
    let mut output = [0u8; 64];
    if let (Ok(page), Ok(per_page), Ok(sort_key)) = (page, per_page, sort_key) {
        let request = CatalogListRequest::new(CatalogListEndpoint::Locations)
            .with_page(page)
            .and_then(|request| request.with_per_page(per_page))
            .and_then(|request| request.with_sort(Sort::new(sort_key, SortDirection::Asc)));
        if let Ok(request) = request {
            assert_eq!(request.write_query(&mut output), Ok(34));
            let query = output
                .get(..34)
                .and_then(|bytes| core::str::from_utf8(bytes).ok());
            assert_eq!(query, Some("page=2&per_page=50&sort=name%3Aasc"));
        }
    }
}

#[test]
fn catalog_all_list_endpoints_accept_pagination_from_api_matrix() {
    let endpoints = [
        CatalogListEndpoint::Locations,
        CatalogListEndpoint::ServerTypes,
        CatalogListEndpoint::LoadBalancerTypes,
        CatalogListEndpoint::Isos,
        CatalogListEndpoint::PublicImages(PublicImageKind::System),
    ];
    let page = Page::new(1);
    if let Ok(page) = page {
        for endpoint in endpoints {
            assert!(CatalogListRequest::new(endpoint).with_page(page).is_ok());
        }
    }
}

#[test]
fn catalog_sortable_list_endpoints_write_sort_query_from_api_matrix() {
    let endpoints = [
        CatalogListEndpoint::Locations,
        CatalogListEndpoint::PublicImages(PublicImageKind::System),
    ];
    let sort_key = SortKey::new("name");
    if let Ok(sort_key) = sort_key {
        for endpoint in endpoints {
            let mut output = [0u8; 64];
            let request = CatalogListRequest::new(endpoint)
                .with_sort(Sort::new(sort_key, SortDirection::Desc));
            if let Ok(request) = request {
                assert_eq!(
                    request.write_query(&mut output),
                    Ok(sort_query_len(endpoint))
                );
            }
        }
    }
}

#[test]
fn catalog_public_image_query_is_restricted_to_public_types() {
    let mut output = [0u8; 64];
    let request = CatalogListRequest::new(CatalogListEndpoint::PublicImages(PublicImageKind::App));
    assert_eq!(request.write_query(&mut output), Ok(8));
    let query = output
        .get(..8)
        .and_then(|bytes| core::str::from_utf8(bytes).ok());
    assert_eq!(query, Some("type=app"));
}

#[test]
fn catalog_rejects_sorting_where_api_matrix_does_not_allow_it() {
    let sort_key = SortKey::new("name");
    if let Ok(sort_key) = sort_key {
        let request = CatalogListRequest::new(CatalogListEndpoint::Isos)
            .with_sort(Sort::new(sort_key, SortDirection::Asc));
        assert_eq!(request, Err(CatalogRequestError::UnsupportedSorting));
    }
}

#[test]
fn catalog_list_requests_are_cloud_get_requests() {
    let request = CatalogListRequest::new(CatalogListEndpoint::ServerTypes);
    assert_eq!(request.method().as_str(), "GET");
    assert_eq!(request.api_base_url(), ApiBaseUrl::CloudV1);
    assert_eq!(
        request.endpoint().endpoint_group(),
        EndpointGroup::ServerTypes
    );
}

fn sort_query_len(endpoint: CatalogListEndpoint) -> usize {
    match endpoint {
        CatalogListEndpoint::Locations => 16,
        CatalogListEndpoint::PublicImages(_) => 28,
        CatalogListEndpoint::ServerTypes
        | CatalogListEndpoint::LoadBalancerTypes
        | CatalogListEndpoint::Isos => 0,
    }
}
