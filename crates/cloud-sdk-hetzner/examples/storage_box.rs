//! Builds a paginated read-only Storage Box list request.

use cloud_sdk_hetzner::pagination::{Page, PerPage};
use cloud_sdk_hetzner::storage::storage_boxes::{
    StorageBoxEndpoint, StorageBoxListRequest, StorageBoxName,
};

fn main() {
    let Ok(name) = StorageBoxName::new("backups") else {
        return;
    };
    let Ok(page) = Page::new(1) else { return };
    let Ok(per_page) = PerPage::new(25) else {
        return;
    };
    let request = StorageBoxListRequest::new()
        .with_name(name)
        .with_page(page)
        .with_per_page(per_page);
    let endpoint = StorageBoxEndpoint::List;
    let mut path = [0_u8; 32];
    let mut query = [0_u8; 64];
    let Ok(path_len) = endpoint.write_path(&mut path) else {
        return;
    };
    let Ok(query_len) = request.write_query(&mut query) else {
        return;
    };

    assert_eq!(endpoint.method().as_str(), "GET");
    assert_eq!(path.get(..path_len), Some("/storage_boxes".as_bytes()));
    assert_eq!(
        query.get(..query_len),
        Some("name=backups&page=1&per_page=25".as_bytes())
    );
}
