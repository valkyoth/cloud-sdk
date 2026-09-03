//! External-consumer checks for crates.io provider identities.

use cloud_sdk::{ProviderMarker, ServiceMarker};
use cloud_sdk_cratesio::{CRATES_IO_PROVIDER_ID, CratesIo, REGISTRY_SERVICE_ID, RegistryService};

#[test]
fn external_consumers_receive_provider_owned_identities() {
    assert_eq!(CratesIo::ID, CRATES_IO_PROVIDER_ID);
    assert_eq!(RegistryService::ID, REGISTRY_SERVICE_ID);
    assert_eq!(
        <<RegistryService as ServiceMarker>::Provider as ProviderMarker>::ID,
        CRATES_IO_PROVIDER_ID,
    );
}
