//! Hetzner-owned provider and service identities.

use cloud_sdk::{ProviderId, ProviderMarker, ServiceId, ServiceMarker, provider_id, service_id};

/// Canonical Hetzner provider identifier.
pub const HETZNER_PROVIDER_ID: ProviderId = provider_id!("hetzner");

/// Canonical Hetzner Cloud API service identifier.
pub const CLOUD_SERVICE_ID: ServiceId = service_id!("cloud");

/// Canonical Hetzner DNS service identifier.
pub const DNS_SERVICE_ID: ServiceId = service_id!("dns");

/// Canonical Hetzner security-resource service identifier.
pub const SECURITY_SERVICE_ID: ServiceId = service_id!("security");

/// Canonical Hetzner Console Storage service identifier.
pub const STORAGE_SERVICE_ID: ServiceId = service_id!("storage");

/// Hetzner provider namespace marker.
pub enum Hetzner {}

impl ProviderMarker for Hetzner {
    const ID: ProviderId = HETZNER_PROVIDER_ID;
}

/// Hetzner Cloud API marker.
pub enum CloudService {}

impl ServiceMarker for CloudService {
    type Provider = Hetzner;
    const ID: ServiceId = CLOUD_SERVICE_ID;
}

/// Hetzner DNS API marker.
pub enum DnsService {}

impl ServiceMarker for DnsService {
    type Provider = Hetzner;
    const ID: ServiceId = DNS_SERVICE_ID;
}

/// Hetzner security-resource API marker.
pub enum SecurityService {}

impl ServiceMarker for SecurityService {
    type Provider = Hetzner;
    const ID: ServiceId = SECURITY_SERVICE_ID;
}

/// Hetzner Console Storage API marker.
pub enum StorageService {}

impl ServiceMarker for StorageService {
    type Provider = Hetzner;
    const ID: ServiceId = STORAGE_SERVICE_ID;
}

#[cfg(test)]
mod tests {
    use cloud_sdk::{ProviderMarker, ServiceMarker};

    use super::{
        CLOUD_SERVICE_ID, CloudService, DNS_SERVICE_ID, DnsService, HETZNER_PROVIDER_ID, Hetzner,
        SECURITY_SERVICE_ID, STORAGE_SERVICE_ID, SecurityService, StorageService,
    };

    #[test]
    fn owns_every_public_service_identity() {
        assert_eq!(Hetzner::ID, HETZNER_PROVIDER_ID);
        assert_eq!(CloudService::ID, CLOUD_SERVICE_ID);
        assert_eq!(DnsService::ID, DNS_SERVICE_ID);
        assert_eq!(SecurityService::ID, SECURITY_SERVICE_ID);
        assert_eq!(StorageService::ID, STORAGE_SERVICE_ID);
        assert_eq!(
            <<StorageService as ServiceMarker>::Provider as ProviderMarker>::ID,
            HETZNER_PROVIDER_ID
        );
    }
}
