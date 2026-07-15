use super::{
    VolumeActionEndpoint, VolumeAttachRequest, VolumeCreatePlacement, VolumeCreateRequest,
    VolumeEndpoint, VolumeId, VolumeListRequest, VolumeLocation, VolumeName,
    VolumeProtectionRequest, VolumeResizeRequest, VolumeSizeGb,
    VolumeSortField, VolumeStatus,
};
use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn storage_ip_volume_paths_match_api_matrix() {
    let id = VolumeId::new(42);
    assert!(id.is_some(), "fixture volume ID must validate");
    let Some(id) = id else { return };
    let action_id = ActionId::new(9);
    assert!(action_id.is_some(), "fixture action ID must validate");
    let Some(action_id) = action_id else { return };
    let mut output = [0u8; 80];
    assert_eq!(VolumeEndpoint::List.write_path(&mut output), Ok(8));
    assert_eq!(VolumeEndpoint::Get(id).write_path(&mut output), Ok(11));
    assert_eq!(VolumeEndpoint::Update(id).method().as_str(), "PUT");
    assert_eq!(VolumeEndpoint::Delete(id).method().as_str(), "DELETE");
    assert_eq!(
        VolumeEndpoint::Create.endpoint_group(),
        EndpointGroup::Volumes
    );

    assert_eq!(
        VolumeActionEndpoint::ListAll.write_path(&mut output),
        Ok(16)
    );
    assert_eq!(
        VolumeActionEndpoint::Get(action_id).write_path(&mut output),
        Ok(18)
    );
    assert_eq!(
        VolumeActionEndpoint::ListForVolume(id).write_path(&mut output),
        Ok(19)
    );
    assert_eq!(
        VolumeActionEndpoint::Attach(id).write_path(&mut output),
        Ok(26)
    );
    assert_eq!(
        VolumeActionEndpoint::ChangeProtection(id).write_path(&mut output),
        Ok(37)
    );
    let path = output
        .get(..37)
        .and_then(|bytes| core::str::from_utf8(bytes).ok());
    assert_eq!(path, Some("/volumes/42/actions/change_protection"));
    assert_eq!(
        VolumeActionEndpoint::Detach(id).write_path(&mut output),
        Ok(26)
    );
    assert_eq!(
        VolumeActionEndpoint::Resize(id).write_path(&mut output),
        Ok(26)
    );
    assert_eq!(
        VolumeActionEndpoint::Attach(id).endpoint_group(),
        EndpointGroup::VolumeActions
    );
}

#[test]
fn storage_ip_volume_query_writes_filters_pagination_and_sorting() {
    let selector = LabelSelector::new("env=prod");
    assert!(selector.is_ok(), "fixture label selector must validate");
    let Ok(selector) = selector else { return };
    let name = VolumeName::new("database-storage");
    assert!(name.is_ok(), "fixture volume name must validate");
    let Ok(name) = name else { return };
    let page = Page::new(2);
    assert!(page.is_ok(), "fixture page must validate");
    let Ok(page) = page else { return };
    let per_page = PerPage::new(25);
    assert!(per_page.is_ok(), "fixture per_page must validate");
    let Ok(per_page) = per_page else { return };
    let mut output = [0u8; 128];
    let request = VolumeListRequest::new()
        .with_label_selector(selector)
        .with_name(name)
        .with_page(page)
        .with_per_page(per_page)
        .with_sort(VolumeSortField::Created, SortDirection::Desc)
        .with_status(VolumeStatus::Available);
    let written = request.write_query(&mut output);
    assert_eq!(written, Ok(103));
    let query = output
        .get(..103)
        .and_then(|bytes| core::str::from_utf8(bytes).ok());
    assert_eq!(
        query,
        Some(
            "label_selector=env%3Dprod&name=database-storage&page=2&per_page=25&sort=created%3Adesc&status=available"
        )
    );
}

#[test]
fn storage_ip_volume_size_and_placement_are_validated() {
    assert_eq!(VolumeSizeGb::new(9), None);
    assert_eq!(VolumeSizeGb::new(10_241), None);

    let size = VolumeSizeGb::new(42);
    assert!(size.is_some(), "fixture volume size must validate");
    let Some(size) = size else { return };
    let name = VolumeName::new("database-storage");
    assert!(name.is_ok(), "fixture volume name must validate");
    let Ok(name) = name else { return };
    let location = VolumeLocation::new("fsn1");
    assert!(location.is_ok(), "fixture volume location must validate");
    let Ok(location) = location else { return };
    let request = VolumeCreateRequest::new(
        size,
        name,
        VolumeCreatePlacement::Location(location),
    );
    assert_eq!(
        request.placement(),
        VolumeCreatePlacement::Location(location)
    );
}

#[test]
fn storage_ip_volume_action_markers_preserve_required_fields() {
    let server = VolumeId::new(42);
    assert!(server.is_some(), "fixture server ID must validate");
    let Some(server) = server else { return };
    let attach = VolumeAttachRequest::new(server, true);
    assert!(attach.automount());
    let size = VolumeSizeGb::new(64);
    assert!(size.is_some(), "fixture resize size must validate");
    let Some(size) = size else { return };
    let resize = VolumeResizeRequest::new(size);
    assert_eq!(resize.size(), size);
    assert!(VolumeProtectionRequest::new(true).delete());
}
