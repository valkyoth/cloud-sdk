//! crates.io-owned provider and service identities.

use cloud_sdk::{ProviderId, ProviderMarker, ServiceId, ServiceMarker, provider_id, service_id};

/// Canonical crates.io provider identifier.
pub const CRATES_IO_PROVIDER_ID: ProviderId = provider_id!("crates-io");

/// Canonical crates.io registry service identifier.
pub const REGISTRY_SERVICE_ID: ServiceId = service_id!("registry");

/// crates.io provider namespace marker.
pub enum CratesIo {}

impl ProviderMarker for CratesIo {
    const ID: ProviderId = CRATES_IO_PROVIDER_ID;
}

/// crates.io registry API marker.
pub enum RegistryService {}

impl ServiceMarker for RegistryService {
    type Provider = CratesIo;
    const ID: ServiceId = REGISTRY_SERVICE_ID;
}

#[cfg(test)]
mod tests {
    use cloud_sdk::{ProviderMarker, ServiceMarker};

    use super::{CRATES_IO_PROVIDER_ID, CratesIo, REGISTRY_SERVICE_ID, RegistryService};

    #[test]
    fn owns_the_registry_identity() {
        assert_eq!(CratesIo::ID, CRATES_IO_PROVIDER_ID);
        assert_eq!(RegistryService::ID, REGISTRY_SERVICE_ID);
        assert_eq!(
            <<RegistryService as ServiceMarker>::Provider as ProviderMarker>::ID,
            CRATES_IO_PROVIDER_ID,
        );
    }
}
