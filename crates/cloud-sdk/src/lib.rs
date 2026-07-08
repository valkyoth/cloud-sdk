#![no_std]
//! Provider-neutral foundations for the `cloud-sdk` workspace.

#[cfg(feature = "std")]
extern crate std;

/// Cloud provider namespace.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Provider {
    /// Hetzner Cloud and DNS APIs.
    Hetzner,
}

/// Provider API family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApiFamily {
    /// Cloud infrastructure API.
    Cloud,
    /// DNS API.
    Dns,
    /// Security-related API resources.
    Security,
    /// Storage-related API resources.
    Storage,
    /// Provider-specific post-1.0 API surface.
    Extended,
}

/// HTTP method for a provider operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Method {
    /// GET request.
    Get,
    /// POST request.
    Post,
    /// PUT request.
    Put,
    /// DELETE request.
    Delete,
}

#[cfg(test)]
mod tests {
    use super::{ApiFamily, Method, Provider};

    #[test]
    fn exposes_provider_neutral_domains() {
        assert_eq!(Provider::Hetzner, Provider::Hetzner);
        assert_eq!(ApiFamily::Cloud, ApiFamily::Cloud);
        assert_eq!(Method::Get, Method::Get);
    }
}
