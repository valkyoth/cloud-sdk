# Migrating To v0.32

v0.32 replaces the closed `Provider` and `ApiFamily` enums with bounded
provider-owned identities. This is a deliberate pre-1.0 source break that lets
new providers and services integrate without editing `cloud-sdk`.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.32.0"
cloud-sdk-hetzner = "0.25.0"
```

Provider-neutral boundary crates move only for their `cloud-sdk` dependency:

- `cloud-sdk-reqwest 0.20.3`
- `cloud-sdk-sanitization 0.15.0` because its stable wrapper API now adapts
  to the upstream `sanitization 2.0.3` module layout
- `cloud-sdk-testkit 0.18.3`

## ProviderService Construction

Before:

```ignore
ProviderService::new(Provider::Hetzner, ApiFamily::Cloud, endpoint)
```

Hetzner callers should use the provider-owned marker:

```rust
use cloud_sdk::operation::ProviderService;
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk_hetzner::CloudService;

# fn main() -> Result<(), cloud_sdk::transport::EndpointIdentityError> {
let endpoint = EndpointIdentity::new(
    EndpointScheme::Https,
    "api.hetzner.cloud",
    443,
    "/v1",
)?;
let service = ProviderService::from_marker::<CloudService>(endpoint);
# let _ = service;
# Ok(())
# }
```

Direct construction remains available with validated `ProviderId` and
`ServiceId` values:

```rust
use cloud_sdk::operation::ProviderService;
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk::{provider_id, service_id};

# fn main() -> Result<(), cloud_sdk::transport::EndpointIdentityError> {
let endpoint = EndpointIdentity::new(
    EndpointScheme::Https,
    "api.example.invalid",
    443,
    "/v2",
)?;
let service = ProviderService::new(
    provider_id!("example"),
    service_id!("compute"),
    endpoint,
);
# let _ = service;
# Ok(())
# }
```

## Accessor Renames

| Removed | Replacement |
| --- | --- |
| `ProviderService::provider()` | `ProviderService::provider_id()` |
| `ProviderService::family()` | `ProviderService::service_id()` |
| `Provider::Hetzner` | `cloud_sdk_hetzner::HETZNER_PROVIDER_ID` |
| `ApiFamily::Cloud` | `cloud_sdk_hetzner::CLOUD_SERVICE_ID` |
| `ApiFamily::Dns` | `cloud_sdk_hetzner::DNS_SERVICE_ID` |
| `ApiFamily::Security` | `cloud_sdk_hetzner::SECURITY_SERVICE_ID` |
| `ApiFamily::Storage` | `cloud_sdk_hetzner::STORAGE_SERVICE_ID` |

`ApiFamily::Extended` has no direct replacement. Provider crates define a
specific bounded service ID and marker instead of routing unrelated future
surfaces through one catch-all value.

## Defining Another Provider

Provider crates own their markers:

```rust
use cloud_sdk::{
    ProviderId, ProviderMarker, ServiceId, ServiceMarker, provider_id,
    service_id,
};

enum ExampleProvider {}

impl ProviderMarker for ExampleProvider {
    const ID: ProviderId = provider_id!("example");
}

enum ObjectStorage {}

impl ServiceMarker for ObjectStorage {
    type Provider = ExampleProvider;
    const ID: ServiceId = service_id!("object-storage");
}
```

No `cloud-sdk` enum or registry changes are required. IDs are at most 63 bytes
and accept lowercase ASCII letters, digits, and single internal hyphens.
Leading, trailing, or repeated hyphens and non-ASCII input are rejected.

## Behavior

Request methods, endpoint matching, transport execution, response policies,
operation IDs, and Hetzner wire formats are unchanged. The migration changes
only provider/service identity representation and ownership.
