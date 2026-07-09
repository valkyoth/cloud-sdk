use super::{
    PlacementGroupCreateRequest, PlacementGroupEndpoint, PlacementGroupId,
    PlacementGroupListRequest, PlacementGroupName, PlacementGroupRequestError,
    PlacementGroupSortField, PlacementGroupType,
};
use crate::EndpointGroup;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn server_adjacent_placement_group_paths_match_api_matrix() {
    let id = PlacementGroupId::new(42);
    let mut output = [0u8; 64];
    if let Some(id) = id {
        assert_eq!(PlacementGroupEndpoint::List.write_path(&mut output), Ok(17));
        assert_eq!(
            PlacementGroupEndpoint::Create.write_path(&mut output),
            Ok(17)
        );
        assert_eq!(
            PlacementGroupEndpoint::Get(id).write_path(&mut output),
            Ok(20)
        );
        assert_eq!(PlacementGroupEndpoint::Update(id).method().as_str(), "PUT");
        assert_eq!(
            PlacementGroupEndpoint::Delete(id).method().as_str(),
            "DELETE"
        );
        let path = output
            .get(..20)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/placement_groups/42"));
        assert_eq!(
            PlacementGroupEndpoint::Get(id).endpoint_group(),
            EndpointGroup::PlacementGroups
        );
    }
}

#[test]
fn server_adjacent_placement_group_query_writes_filters_pagination_and_sorting() {
    let selector = LabelSelector::new("env=prod");
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let mut output = [0u8; 96];
    if let (Ok(selector), Ok(page), Ok(per_page)) = (selector, page, per_page) {
        let request = PlacementGroupListRequest::new()
            .with_label_selector(selector)
            .with_page(page)
            .with_per_page(per_page)
            .with_sort(PlacementGroupSortField::Created, SortDirection::Desc);
        let written = request.write_query(&mut output);
        assert_eq!(written, Ok(64));
        let query = output
            .get(..64)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(
            query,
            Some("label_selector=env%3Dprod&page=2&per_page=25&sort=created%3Adesc")
        );
    }
}

#[test]
fn server_adjacent_placement_group_type_and_required_fields_are_validated() {
    assert_eq!(
        PlacementGroupType::from_api_str("cluster"),
        Err(PlacementGroupRequestError::InvalidType)
    );
    assert_eq!(
        PlacementGroupType::from_api_str("spread").map(PlacementGroupType::as_api_str),
        Ok("spread")
    );
    let name = PlacementGroupName::new("spread-a");
    if let Ok(name) = name {
        assert_eq!(
            PlacementGroupCreateRequest::try_new(Some(name), None),
            Err(PlacementGroupRequestError::MissingRequiredField)
        );
        let request =
            PlacementGroupCreateRequest::try_new(Some(name), Some(PlacementGroupType::Spread));
        assert!(request.is_ok());
        if let Ok(request) = request {
            assert_eq!(request.endpoint(), PlacementGroupEndpoint::Create);
            assert_eq!(request.placement_group_type(), PlacementGroupType::Spread);
        }
    }
}
