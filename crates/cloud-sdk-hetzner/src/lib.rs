#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(any(feature = "std", test))]
extern crate std;

pub mod actions;
pub mod cloud;
pub mod dns;
pub mod endpoint;
pub mod labels;
pub mod pagination;
pub mod query;
pub mod rate_limit;
pub mod request;
pub mod response;
pub mod security;
pub mod storage;

pub use endpoint::{ApiSurface, EndpointGroup};
pub use request::{CLOUD_API_BASE_URL, CLOUD_API_VERSION};

#[cfg(test)]
mod tests {
    use super::{ApiSurface, EndpointGroup};

    #[test]
    fn sdk_exposes_endpoint_domains() {
        assert_eq!(EndpointGroup::Firewalls.surface(), ApiSurface::Cloud);
        assert_eq!(EndpointGroup::ZoneActions.surface(), ApiSurface::Dns);
        assert_eq!(EndpointGroup::StorageBoxes.surface(), ApiSurface::Storage);
    }
}
