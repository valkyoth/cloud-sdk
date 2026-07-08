//! Request-domain primitives shared by endpoint modules.

pub use cloud_sdk::Method;

/// Hetzner Cloud API base URL for the public v1 REST API.
pub const CLOUD_API_BASE_URL: &str = "https://api.hetzner.cloud/v1";

/// Hetzner Cloud API major version currently targeted by this SDK.
pub const CLOUD_API_VERSION: u8 = 1;

#[cfg(test)]
mod tests {
    use super::{CLOUD_API_BASE_URL, CLOUD_API_VERSION};

    #[test]
    fn exposes_cloud_v1_base_url() {
        assert_eq!(CLOUD_API_BASE_URL, "https://api.hetzner.cloud/v1");
        assert_eq!(CLOUD_API_VERSION, 1);
    }
}
