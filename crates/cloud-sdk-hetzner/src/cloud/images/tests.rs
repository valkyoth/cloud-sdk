use super::{
    ImageActionEndpoint, ImageEndpoint, ImageId, ImageListRequest, ImageProtectionRequest,
    ImageSortField, ImageTypeFilter,
};
use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn server_adjacent_image_paths_match_api_matrix() {
    let id = ImageId::new(42);
    let action_id = ActionId::new(9);
    let mut output = [0u8; 64];
    if let (Some(id), Some(action_id)) = (id, action_id) {
        assert_eq!(ImageEndpoint::List.write_path(&mut output), Ok(7));
        assert_eq!(ImageEndpoint::Get(id).write_path(&mut output), Ok(10));
        assert_eq!(ImageEndpoint::Update(id).method().as_str(), "PUT");
        assert_eq!(ImageEndpoint::Delete(id).method().as_str(), "DELETE");
        assert_eq!(
            ImageEndpoint::Get(id).endpoint_group(),
            EndpointGroup::Images
        );

        assert_eq!(ImageActionEndpoint::ListAll.write_path(&mut output), Ok(15));
        assert_eq!(
            ImageActionEndpoint::Get(action_id).write_path(&mut output),
            Ok(17)
        );
        assert_eq!(
            ImageActionEndpoint::ListForImage(id).write_path(&mut output),
            Ok(18)
        );
        assert_eq!(
            ImageActionEndpoint::ChangeProtection(id).write_path(&mut output),
            Ok(36)
        );
        let path = output
            .get(..36)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/images/42/actions/change_protection"));
        assert_eq!(
            ImageActionEndpoint::ChangeProtection(id).endpoint_group(),
            EndpointGroup::ImageActions
        );
    }
}

#[test]
fn server_adjacent_image_list_query_writes_filters_pagination_and_sorting() {
    let bound_to = ImageId::new(42);
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let mut output = [0u8; 96];
    if let (Some(bound_to), Ok(page), Ok(per_page)) = (bound_to, page, per_page) {
        let request = ImageListRequest::new()
            .with_bound_to(bound_to)
            .with_page(page)
            .with_per_page(per_page)
            .with_sort(ImageSortField::Created, SortDirection::Desc)
            .with_type(ImageTypeFilter::Snapshot);
        let written = request.write_query(&mut output);
        assert_eq!(written, Ok(64));
        let query = output
            .get(..64)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(
            query,
            Some("bound_to=42&page=2&per_page=25&sort=created%3Adesc&type=snapshot")
        );
    }
}

#[test]
fn server_adjacent_image_request_markers_are_explicit() {
    assert!(ImageProtectionRequest::new(true).delete());
    let mut output = [0u8; 4];
    if let Some(id) = ImageId::new(42) {
        assert!(ImageEndpoint::Get(id).write_path(&mut output).is_err());
    }
}
