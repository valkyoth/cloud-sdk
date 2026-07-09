use super::{
    VolumeActionEndpoint, VolumeAttachRequest, VolumeCreatePlacement, VolumeCreateRequest,
    VolumeEndpoint, VolumeId, VolumeListRequest, VolumeLocation, VolumeName,
    VolumeProtectionRequest, VolumeRequestError, VolumeResizeRequest, VolumeSizeGb,
    VolumeSortField, VolumeStatus,
};
use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};

#[test]
fn storage_ip_volume_paths_match_api_matrix() {
    let id = VolumeId::new(42);
    let action_id = ActionId::new(9);
    let mut output = [0u8; 80];
    if let (Some(id), Some(action_id)) = (id, action_id) {
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
}

#[test]
fn storage_ip_volume_query_writes_filters_pagination_and_sorting() {
    let selector = LabelSelector::new("env=prod");
    let name = VolumeName::new("database-storage");
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let mut output = [0u8; 128];
    if let (Ok(selector), Ok(name), Ok(page), Ok(per_page)) = (selector, name, page, per_page) {
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
}

#[test]
fn storage_ip_volume_size_and_placement_are_validated() {
    assert_eq!(VolumeSizeGb::new(9), None);
    assert_eq!(VolumeSizeGb::new(10_241), None);

    let size = VolumeSizeGb::new(42);
    let name = VolumeName::new("database-storage");
    let location = VolumeLocation::new("fsn1");
    if let (Some(size), Ok(name), Ok(location)) = (size, name, location) {
        assert_eq!(
            VolumeCreateRequest::try_new(
                None,
                Some(name),
                Some(VolumeCreatePlacement::Location(location))
            ),
            Err(VolumeRequestError::MissingRequiredField)
        );
        let request = VolumeCreateRequest::try_new(
            Some(size),
            Some(name),
            Some(VolumeCreatePlacement::Location(location)),
        );
        assert_eq!(
            request.map(VolumeCreateRequest::placement),
            Ok(VolumeCreatePlacement::Location(location))
        );
    }
}

#[test]
fn storage_ip_volume_action_markers_require_required_fields() {
    assert_eq!(
        VolumeAttachRequest::try_new(None, false),
        Err(VolumeRequestError::MissingRequiredField)
    );
    assert_eq!(
        VolumeResizeRequest::try_new(None),
        Err(VolumeRequestError::MissingRequiredField)
    );
    if let Some(server) = VolumeId::new(42) {
        let attach = VolumeAttachRequest::try_new(Some(server), true);
        assert_eq!(attach.map(VolumeAttachRequest::automount), Ok(true));
    }
    let size = VolumeSizeGb::new(64);
    if let Some(size) = size {
        let resize = VolumeResizeRequest::try_new(Some(size));
        assert_eq!(resize.map(VolumeResizeRequest::size), Ok(size));
    }
    assert!(VolumeProtectionRequest::new(true).delete());
}
