//! Builds a paginated read-only Hetzner public-image catalog request.

use cloud_sdk_hetzner::cloud::catalog::{CatalogListEndpoint, CatalogListRequest, PublicImageKind};
use cloud_sdk_hetzner::pagination::{Page, PerPage};

fn main() {
    let endpoint = CatalogListEndpoint::PublicImages(PublicImageKind::System);
    let Ok(page) = Page::new(1) else { return };
    let Ok(per_page) = PerPage::new(25) else {
        return;
    };
    let Ok(request) = CatalogListRequest::new(endpoint).with_page(page) else {
        return;
    };
    let Ok(request) = request.with_per_page(per_page) else {
        return;
    };
    let mut query = [0_u8; 64];
    let Ok(written) = request.write_query(&mut query) else {
        return;
    };
    let Some(query) = query
        .get(..written)
        .and_then(|value| core::str::from_utf8(value).ok())
    else {
        return;
    };

    assert_eq!(request.method().as_str(), "GET");
    assert_eq!(request.endpoint().path_str(), "/images");
    assert_eq!(query, "type=system&page=1&per_page=25");
}
