use super::{
    ImageActionEndpoint, ImageArchitecture, ImageEndpoint, ImageId, ImageListRequest, ImageName,
    ImageProtectionRequest, ImageSortField, ImageStatus, ImageTypeFilter,
};
use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn server_adjacent_image_paths_match_api_matrix() {
    let id = ImageId::new(42);
    let action_id = ActionId::new(9);
    let mut output = [0u8; 64];
    let (Some(id), Some(action_id)) = (id, action_id) else {
        unreachable!("security fixture construction failed");
    };
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

#[test]
fn server_adjacent_image_list_query_writes_filters_pagination_and_sorting() {
    let bound_to = ImageId::new(42);
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let selector = LabelSelector::new("env=prod");
    let name = ImageName::new("debian");
    let mut output = [0u8; 192];
    let (Some(bound_to), Ok(page), Ok(per_page), Ok(selector), Ok(name)) =
        (bound_to, page, per_page, selector, name)
    else {
        unreachable!("security fixture construction failed");
    };
    let request = ImageListRequest::new()
        .with_architecture(ImageArchitecture::Arm)
        .with_bound_to(bound_to)
        .with_include_deprecated(false)
        .with_label_selector(selector)
        .with_name(name)
        .with_page(page)
        .with_per_page(per_page)
        .with_sort(ImageSortField::Created, SortDirection::Desc)
        .with_status(ImageStatus::Available)
        .with_type(ImageTypeFilter::Snapshot);
    let written = request.write_query(&mut output);
    assert_eq!(written, Ok(161));
    let query = output
        .get(..161)
        .and_then(|bytes| core::str::from_utf8(bytes).ok());
    assert_eq!(
        query,
        Some(
            "architecture=arm&bound_to=42&include_deprecated=false&label_selector=env%3Dprod&name=debian&page=2&per_page=25&sort=created%3Adesc&status=available&type=snapshot"
        )
    );
}

#[test]
fn server_adjacent_image_request_markers_are_explicit() {
    assert!(ImageProtectionRequest::new(true).delete());
    let mut output = [0u8; 4];
    let Some(id) = ImageId::new(42) else {
        unreachable!("security fixture construction failed");
    };
    assert!(ImageEndpoint::Get(id).write_path(&mut output).is_err());
}
