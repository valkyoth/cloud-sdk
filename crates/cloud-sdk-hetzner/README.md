# cloud-sdk-hetzner

Hetzner provider crate for the main
[`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace.

This is the main documentation surface for Hetzner support in `cloud-sdk`.
It source-locks the Hetzner Cloud/DNS and Storage Box OpenAPI specs, owns the
Hetzner endpoint module layout, and will receive the Hetzner request/response
models in small reviewed releases.

## Install

```toml
[dependencies]
cloud-sdk = "0.2.0"
cloud-sdk-hetzner = "0.2.0"
```

## Current Scope

`0.2.0` is a source-lock and planning release. It does not yet implement HTTP
transport, serde models, endpoint builders, token storage, live API tests, retry
policy, or action polling.

Implemented in the published `0.2.0` line:

- no_std-first provider crate;
- Hetzner Cloud, DNS, security, and Storage Box endpoint ownership domains;
- source-locked API matrix and drift fingerprints in the repository docs;
- explicit optional boundaries for future transport, testkit, and sanitization
  crates.

Implemented on `main` for the next `0.3.0` release:

- endpoint path validation and base URL selection;
- endpoint group to base URL mapping;
- fixed-capacity query parameters and fixed-buffer percent encoding;
- label key, value, and selector validation;
- pagination, sorting, action status, API error, and rate-limit domains.

## Endpoint Surface Example

```rust
use cloud_sdk_hetzner::{ApiSurface, EndpointGroup};

assert_eq!(EndpointGroup::Servers.surface(), ApiSurface::Cloud);
assert_eq!(EndpointGroup::Zones.surface(), ApiSurface::Dns);
assert_eq!(EndpointGroup::Certificates.surface(), ApiSurface::Security);
assert_eq!(EndpointGroup::StorageBoxes.surface(), ApiSurface::Storage);
```

## Base URL Example

```rust
use cloud_sdk_hetzner::{CLOUD_API_BASE_URL, CLOUD_API_VERSION};

assert_eq!(CLOUD_API_BASE_URL, "https://api.hetzner.cloud/v1");
assert_eq!(CLOUD_API_VERSION, 1);
```

## Endpoint Group Base URL Example

```rust
use cloud_sdk_hetzner::{request::ApiBaseUrl, EndpointGroup};

assert_eq!(EndpointGroup::Servers.api_base_url(), ApiBaseUrl::CloudV1);
assert_eq!(EndpointGroup::Zones.api_base_url(), ApiBaseUrl::CloudV1);
assert_eq!(EndpointGroup::StorageBoxes.api_base_url(), ApiBaseUrl::HetznerV1);
```

## Query Encoding Example

```rust
use cloud_sdk_hetzner::query::{QueryBuilder, QueryParam};

# fn main() -> Result<(), cloud_sdk_hetzner::query::QueryError> {
let mut query = QueryBuilder::<1>::new();
query.push(QueryParam::new("label_selector", "env=prod")?)?;

let mut output = [0u8; 64];
let written = query.write_percent_encoded(&mut output)?;
let encoded = output
    .get(..written)
    .and_then(|bytes| core::str::from_utf8(bytes).ok());

assert_eq!(encoded, Some("label_selector=env%3Dprod"));
# Ok(())
# }
```

## Module Ownership Example

```rust
use cloud_sdk_hetzner::{cloud, dns, security, storage, EndpointGroup};

assert!(cloud::servers::ENDPOINT_GROUPS.contains(&EndpointGroup::Servers));
assert!(dns::zones::ENDPOINT_GROUPS.contains(&EndpointGroup::Zones));
assert!(security::ssh_keys::ENDPOINT_GROUPS.contains(&EndpointGroup::SshKeys));
assert!(storage::storage_boxes::ENDPOINT_GROUPS.contains(&EndpointGroup::StorageBoxes));
```

## Source-Locked API Areas

- Actions.
- Servers, server actions, server types, images, image actions, ISOs,
  placement groups, primary IPs, and primary IP actions.
- Volumes and volume actions.
- Floating IPs and floating IP actions.
- Firewalls and firewall actions.
- Load balancers, load balancer actions, and load balancer types.
- Networks and network actions.
- DNS zones, zone actions, RRSets, and RRSet actions.
- Certificates, certificate actions, and SSH keys.
- Storage Boxes, Storage Box actions, snapshots, subaccounts, subaccount
  actions, and Storage Box types.
- Locations and pricing.

Robot Webservice is intentionally post-1.0 because it has a different
authentication model, request encoding, and API shape.
