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
cloud-sdk = "0.4.0"
cloud-sdk-hetzner = "0.4.0"
```

## Current Scope

The current main branch has reached the `0.5.0` implementation stop for
security request domains. It does not yet implement HTTP transport, serde
models, body serialization, token storage, live API tests, retry policy,
pagination iterators, or action polling.

Implemented in the published `0.2.0` line:

- no_std-first provider crate;
- Hetzner Cloud, DNS, security, and Storage Box endpoint ownership domains;
- source-locked API matrix and drift fingerprints in the repository docs;
- explicit optional boundaries for future transport, testkit, and sanitization
  crates.

Implemented in the published `0.3.0` line:

- endpoint path validation and base URL selection;
- endpoint group to base URL mapping;
- fixed-capacity query parameters and fixed-buffer percent encoding;
- label key, value, and selector validation;
- pagination, sorting, action status, API error, and rate-limit domains.

Implemented in the published `0.4.0` line:

- read-only catalog request primitives for locations, pricing, server types,
  load balancer types, ISOs, and public images;
- fixed-buffer get-path and list-query construction for catalog endpoints;
- pagination and sorting capability checks from the source-locked API matrix.

Implemented on main for next `0.5.0`:

- SSH key list/create/get/update/delete request primitives;
- certificate list/create/get/update/delete request primitives;
- certificate retry action request primitive;
- validation for names, SSH public keys, PEM values, managed certificate domain
  names, labels, pagination, and source-locked sorting;
- redacted `Debug` output for secret-adjacent SSH public key and certificate
  PEM request values.

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

## Catalog Request Example

```rust
use cloud_sdk_hetzner::cloud::catalog::{
    CatalogListEndpoint, CatalogListRequest, PublicImageKind,
};
use cloud_sdk_hetzner::pagination::{Page, PerPage};

# fn main() -> Result<(), cloud_sdk_hetzner::cloud::catalog::CatalogRequestError> {
let page = match Page::new(1) {
    Ok(page) => page,
    Err(_) => return Ok(()),
};
let per_page = match PerPage::new(25) {
    Ok(per_page) => per_page,
    Err(_) => return Ok(()),
};

let request = CatalogListRequest::new(CatalogListEndpoint::PublicImages(
    PublicImageKind::System,
))
.with_page(page)?
.with_per_page(per_page)?;

let mut output = [0u8; 64];
let written = request.write_query(&mut output)?;
let encoded = output
    .get(..written)
    .and_then(|bytes| core::str::from_utf8(bytes).ok());

assert_eq!(encoded, Some("type=system&page=1&per_page=25"));
# Ok(())
# }
```

## Security Request Example

```rust
use cloud_sdk_hetzner::security::ssh_keys::{
    SshKeyCreateRequest, SshKeyName, SshPublicKey,
};

# fn main() -> Result<(), cloud_sdk_hetzner::security::ssh_keys::SecurityRequestError> {
let name = SshKeyName::new("deploy")?;
let public_key = SshPublicKey::new("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMockKey")?;
let request = SshKeyCreateRequest::try_new(Some(name), Some(public_key))?;

assert_eq!(request.endpoint().method().as_str(), "POST");
assert_eq!(request.endpoint().write_path(&mut [0u8; 16])?, 9);
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
