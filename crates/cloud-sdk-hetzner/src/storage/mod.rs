//! Storage Box resource modules.

pub mod storage_boxes;

use crate::EndpointGroup;

/// Endpoint groups owned by the Storage Boxes API module.
pub const STORAGE_ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::StorageBoxes,
    EndpointGroup::StorageBoxActions,
    EndpointGroup::StorageBoxSubaccounts,
];

#[cfg(test)]
mod tests {
    use super::STORAGE_ENDPOINT_GROUPS;
    use crate::EndpointGroup;

    #[test]
    fn includes_storage_box_groups() {
        assert!(STORAGE_ENDPOINT_GROUPS.contains(&EndpointGroup::StorageBoxes));
        assert!(STORAGE_ENDPOINT_GROUPS.contains(&EndpointGroup::StorageBoxSubaccounts));
    }
}
