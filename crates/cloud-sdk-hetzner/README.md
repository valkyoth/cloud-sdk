<p align="center">
  <b>no_std-first Hetzner provider crate for cloud-sdk.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">cloud-sdk crate</a>
  |
  <a href="https://docs.rs/cloud-sdk-hetzner">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/cloud-sdk">
    <img src="https://raw.githubusercontent.com/valkyoth/cloud-sdk/main/.github/images/cloud-sdk.webp" alt="cloud-sdk Rust crate overview">
  </a>
</p>

# cloud-sdk-hetzner

Hetzner provider crate for the main GitHub
[`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace and the
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate on crates.io.

This is the main documentation surface for Hetzner support in `cloud-sdk`.
It covers the Hetzner Cloud, DNS, and Storage Box APIs and provides validated
request models with reviewed shared response, pagination, and action
boundaries.

## Install

```toml
[dependencies]
cloud-sdk = "0.31.0"
cloud-sdk-hetzner = "0.24.0"
```

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps provider models allocation-free, transport-free, and `no_std`. |
| `alloc` | no | Enables provider APIs that require the Rust `alloc` crate. |
| `serde` | no | Enables reviewed RRSet request serialization and checked response decoding for every active operation; also enables `alloc`. |
| `std` | no | Enables `alloc` and standard-library integration without selecting a transport. |

Docs.rs builds with all features. The default dependency graph still includes
no network client, TLS implementation, async runtime, filesystem, or clock.

## Workflow Examples

Compile-checked read-only, mutation, pagination, action polling, DNS, and
Storage Box examples are indexed in the
[Hetzner workflow guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/HETZNER_EXAMPLES.md).
Security-sensitive transport decisions are covered by the
[security recipes](https://github.com/valkyoth/cloud-sdk/blob/main/docs/SECURITY_RECIPES.md).

Every active operation can be converted into a bounded provider-neutral
prepared request. This mutation example performs no network operation:

```rust
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk_hetzner::cloud::load_balancers::{
    LoadBalancerCreateRequest, LoadBalancerName, LoadBalancerType,
};

let name = LoadBalancerName::new("edge")?;
let load_balancer_type = LoadBalancerType::new("lb11")?;
let operation = LoadBalancerCreateRequest::new(name, load_balancer_type);
let mut target = [0_u8; 128];
let mut body = [0_u8; 512];
let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(prepared.transport_request().target().as_str(), "/load_balancers");
# Ok::<(), Box<dyn core::error::Error>>(())
```

Secret-bearing operations need successful-path cleanup after transport use.
Add `cloud-sdk-sanitization = "0.14.0"` and guard the complete body buffer:

```rust
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk_hetzner::storage::storage_boxes::{
    StorageBoxCreateRequest, StorageBoxLocation, StorageBoxName,
    StorageBoxPassword, StorageBoxTypeRef,
};
use cloud_sdk_sanitization::SecretBuffer;

let name = StorageBoxName::new("backup")?;
let location = StorageBoxLocation::new("fsn1")?;
let box_type = StorageBoxTypeRef::new("bx20")?;
let password = StorageBoxPassword::new("example-only-not-a-real-secret")?;
let operation = StorageBoxCreateRequest::new(name, location, box_type, password);
let mut target = [0_u8; 128];
let mut body_bytes = [0_u8; 512];
{
    let mut body = SecretBuffer::new(&mut body_bytes);
    let prepared = operation.prepare(PreparationStorage::new(
        &mut target,
        body.as_mut_slice(),
    ))?;
    assert!(!prepared.transport_request().body().is_empty());
    // Send while `prepared` borrows the guarded body, then leave this scope.
}
assert!(body_bytes.iter().all(|byte| *byte == 0));
# Ok::<(), Box<dyn core::error::Error>>(())
```

Before a custom transport sends credentials, call
`verify_official_endpoint(&transport, expected_base)`. The helper fails closed
unless scheme, host, effective port, and base path exactly match the selected
official Cloud or Storage API endpoint.

## Request Operation Coverage

The current release has complete prepared-request coverage for all 208
source-locked non-deprecated Cloud, DNS, and Storage Box operations. Each
prepared operation binds its method, target, bounded body, response policy,
safety and retry classification, cost intent, and exact official endpoint.

| Hetzner API area | Request models and path/query encoding |
| --- | --- |
| Global actions | Complete |
| Servers, images, ISOs, placement groups, and primary IPs | Complete |
| Volumes and floating IPs | Complete |
| Firewalls, load balancers, and networks | Complete |
| DNS zones and RRSets | Complete |
| Certificates and SSH keys | Complete |
| Storage Boxes, snapshots, and subaccounts | Complete |
| Locations, server types, load balancer types, and pricing | Complete |

### Capability Coverage

| Capability | Current coverage | Planned completion |
| --- | --- | --- |
| Request models | Complete for all 208 non-deprecated operations | Current |
| Path/query encoding | Complete for all 208 non-deprecated operations | Current |
| Body serialization | Complete for all 91 non-deprecated operations with request bodies | Current |
| Success response models | Complete checked envelope and resource-identity coverage for all 208 non-deprecated operations | Provider-complete resource fields before `1.0.0` |
| Error response models | Complete checked typed API error decoding for all active operations | Current |
| End-to-end client | Not available | `v0.32.0` |

Thirteen deprecated operations remain deliberately unavailable. A checked
release gate prevents non-deprecated request operations from returning to a
planned or deferred state. See the
[API matrix](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_MATRIX.md)
for operation-level request status and the
[release plan](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_PLAN.md)
for prepared-request, serialization, response, and client milestones.
Upstream source monitoring and lock-refresh decisions follow the
[API drift maintenance runbook](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_DRIFT_MAINTENANCE.md).
Breaking v0.27 constructor and custom-endpoint changes are listed in the
[migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.27.0.md).
Shared transport and credential lifecycle changes are listed in the
[v0.29 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.29.0.md).

## Optional Serde Boundary

Enable Serde explicitly; it is never part of the default graph:

```toml
[dependencies]
cloud-sdk-hetzner = { version = "0.24.0", features = ["serde"] }
```

The feature admits serde_json with `default-features = false` and `alloc` only.
The checked decoder consumes a `TransportResponse` together with its exact
`PreparedRequest`, applies the prepared status/content-type/body policy, rejects
duplicate or malformed JSON, and returns validated typed success or API errors.
Resource responses currently expose validated identity and common state fields;
provider-complete resource field models remain planned before `1.0.0`. The
bounded parser tree and its volatile-clearing string storage remain private:

```rust
# #[cfg(feature = "serde")]
# fn main() {
use cloud_sdk_hetzner::dns::rrsets::{
    RrsetName, RrsetProtectionRequest, RrsetReference, RrsetType,
};
use cloud_sdk_hetzner::dns::zones::{ZoneName, ZoneReference};
use cloud_sdk_hetzner::serde::RrsetRequestBody;

let Ok(zone_name) = ZoneName::new("example.com") else {
    return;
};
let Ok(rrset_name) = RrsetName::new("www") else {
    return;
};
let reference = RrsetReference::new(
    ZoneReference::Name(zone_name),
    rrset_name,
    RrsetType::A,
);
let request = RrsetProtectionRequest::new(reference, true);
let Ok(body) = RrsetRequestBody::protection(request) else {
    return;
};

let json = serde_json::to_string(&body);
assert!(json.is_ok());
if let Ok(json) = json {
    assert_eq!(json, r#"{"change":true}"#);
}
# }
# #[cfg(not(feature = "serde"))]
# fn main() {}
```

Decode only through the prepared request that produced the response:

```rust
# #[cfg(feature = "serde")]
# fn main() -> Result<(), Box<dyn core::error::Error>> {
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{ResponseContentType, StatusCode, TransportResponse};
use cloud_sdk_hetzner::cloud::servers::{ServerEndpoint, ServerId};
use cloud_sdk_hetzner::serde::{HetznerSuccess, decode_response};

let endpoint = ServerEndpoint::Get(ServerId::new(42).ok_or("invalid ID")?);
let mut target = [0_u8; 64];
let mut body = [];
let prepared = endpoint.prepare(PreparationStorage::new(&mut target, &mut body))?;
let response_body = br#"{"server":{"id":42,"name":"web-1","status":"running"}}"#;
let content_type = ResponseContentType::new("application/json")?;
let response = TransportResponse::new(StatusCode::OK, response_body)
    .with_content_type(content_type);
let decoded = decode_response(prepared, response)?;

let HetznerSuccess::Resource(server) = decoded.success() else {
    return Err("unexpected response family".into());
};
assert_eq!(server.name(), Some("web-1"));
# Ok(())
# }
# #[cfg(not(feature = "serde"))]
# fn main() {}
```

Direct parser use bypasses the prepared status, content-type, body-shape, and
operation-binding checks. Secret-bearing responses and zonefiles move their
already protected parser strings into the response model without another
plaintext allocation, expose their text only through checked closures, and use
redacted diagnostics. Every parsed string value uses volatile-clearing storage
from admission onward, including parser and model error paths. A shared
65,536-node budget bounds aggregate JSON structure allocation. Cloned response
models share the protected allocation, which is cleared after the final clone
drops. Callers still own and must clear the original transport response buffer.

## RRSet Request Example

```rust
use cloud_sdk::Method;
use cloud_sdk_hetzner::dns::rrsets::{
    Record, RecordValue, Records, RrsetCreateRequest, RrsetName, RrsetType,
};
use cloud_sdk_hetzner::dns::zones::{ZoneName, ZoneReference};

# fn main() -> Result<(), cloud_sdk_hetzner::dns::rrsets::RrsetRequestError> {
let zone_name = ZoneName::new("example.com")
    .map_err(|_| cloud_sdk_hetzner::dns::rrsets::RrsetRequestError::InvalidName)?;
let zone = ZoneReference::Name(zone_name);
let name = RrsetName::new("www")?;
let values = [Record::new(RecordValue::new("192.0.2.1")?)];
let records = Records::new(&values)?;
let request = RrsetCreateRequest::new(zone, name, RrsetType::A, records);

assert_eq!(request.endpoint().method(), Method::Post);
let mut path = [0_u8; 64];
let written = request.endpoint().write_path(&mut path)?;
assert_eq!(
    path.get(..written),
    Some(b"/zones/example.com/rrsets".as_slice())
);
# Ok(())
# }
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

## Pagination Response Example

The optional Serde boundary can extract shared pagination metadata from any
Hetzner list response while ignoring the resource-specific fields:

```rust
# #[cfg(feature = "serde")]
# fn main() {
use cloud_sdk::pagination::{PageLimit, PaginationCursor};
use cloud_sdk_hetzner::serde::PaginationEnvelope;

let body = br#"{
    "servers": [{"id": 42}],
    "meta": {"pagination": {
        "page": 1,
        "per_page": 25,
        "previous_page": null,
        "next_page": null,
        "last_page": 1,
        "total_entries": 1
    }}
}"#;
let Ok(envelope) = serde_json::from_slice::<PaginationEnvelope>(body) else {
    return;
};
let metadata = envelope.pagination();
let Ok(limit) = PageLimit::new(10) else { return };
let Ok(first) = cloud_sdk::pagination::PageNumber::new(1) else { return };
let Ok(mut cursor) = PaginationCursor::new(
    first,
    u64::from(metadata.per_page().get()),
    limit,
) else {
    return;
};
let Ok(boundary) = cursor.observe(metadata.as_core(), 1, None) else {
    return;
};

assert!(boundary.is_terminal());
assert_eq!(metadata.total_entries(), Some(1));
# }
# #[cfg(not(feature = "serde"))]
# fn main() {}
```

Pass `TransportResponse::rate_limit()` as the final `observe` argument when a
real or mock transport supplies it. The caller remains responsible for
decoding the resource array and reporting its exact entry count.

## Action Polling Example

```rust
# #[cfg(feature = "serde")]
# fn main() {
use core::time::Duration;
use cloud_sdk::action_polling::{
    ActionPollStep, ActionPoller, PollContext, PollDecision, PollPolicy,
};
use cloud_sdk_hetzner::serde::ActionEnvelope;

struct FixedDelay;

impl PollPolicy for FixedDelay {
    type Error = ();

    fn decide(&mut self, _context: PollContext) -> Result<PollDecision, Self::Error> {
        Ok(PollDecision::Delay(Duration::from_secs(2)))
    }
}

let body = br#"{"action":{
    "id":42,"command":"create_server","status":"running","progress":25,
    "started":"2026-07-13T12:00:00Z","finished":null,
    "resources":[],"error":null
}}"#;
let Ok(envelope) = serde_json::from_slice::<ActionEnvelope<'_>>(body) else {
    return;
};
let mut poller = ActionPoller::new();
let mut policy = FixedDelay;
let step = poller.observe(
    envelope.action().polling_update(),
    envelope.action().progress(),
    None,
    &mut policy,
);

assert_eq!(step, Ok(ActionPollStep::Delay(Duration::from_secs(2))));
# }
# #[cfg(not(feature = "serde"))]
# fn main() {}
```

For an `error` action, the step is `ActionPollStep::Failed` and carries the
validated optional Hetzner error response. The SDK never sleeps, retries, or
declares a timeout on its own.

## Security Request Example

```rust
use cloud_sdk_hetzner::security::ssh_keys::{
    SshKeyCreateRequest, SshKeyName, SshPublicKey,
};

# fn main() -> Result<(), cloud_sdk_hetzner::security::ssh_keys::SecurityRequestError> {
let name = SshKeyName::new("deploy")?;
let public_key = SshPublicKey::new("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMockKey")?;
let request = SshKeyCreateRequest::new(name, public_key);

assert_eq!(request.endpoint().method().as_str(), "POST");
assert_eq!(request.endpoint().write_path(&mut [0u8; 16])?, 9);
# Ok(())
# }
```

## Server Request Example

```rust
use cloud_sdk_hetzner::cloud::servers::{
    ServerCreateRequest, ServerName, ServerReference,
};

# fn main() -> Result<(), cloud_sdk_hetzner::cloud::servers::ServerRequestError> {
let name = ServerName::new("web-1")?;
let server_type = ServerReference::new("cpx22")?;
let image = ServerReference::new("ubuntu-24.04")?;
let request = ServerCreateRequest::new(name, server_type, image);

assert_eq!(request.endpoint().method().as_str(), "POST");
assert_eq!(request.endpoint().write_path(&mut [0u8; 16])?, 8);
# Ok(())
# }
```

## Firewall And Network Examples

### Firewall Rule

```rust
use cloud_sdk_hetzner::cloud::firewalls::rules::{
    FirewallPort, FirewallProtocol, FirewallRule, FirewallSelectors,
};
use cloud_sdk_hetzner::cloud::ip::IpCidr;

let source = match IpCidr::new("192.0.2.0/24") {
    Ok(source) => source,
    Err(_) => return,
};
let sources = [source];
let selectors = match FirewallSelectors::incoming(&sources) {
    Ok(selectors) => selectors,
    Err(_) => return,
};
let port = match FirewallPort::new("443") {
    Ok(port) => port,
    Err(_) => return,
};
let rule = match FirewallRule::try_new(selectors, FirewallProtocol::Tcp, Some(port)) {
    Ok(rule) => rule,
    Err(_) => return,
};

assert_eq!(rule.protocol(), FirewallProtocol::Tcp);
```

### Network Create Request

```rust
use cloud_sdk_hetzner::cloud::ip::NetworkIpRange;
use cloud_sdk_hetzner::cloud::networks::{NetworkCreateRequest, NetworkName};

let name = match NetworkName::new("private") {
    Ok(name) => name,
    Err(_) => return,
};
let ip_range = match NetworkIpRange::new("10.0.0.0/16") {
    Ok(ip_range) => ip_range,
    Err(_) => return,
};
let request = NetworkCreateRequest::new(name, ip_range);

assert_eq!(request.ip_range().as_str(), "10.0.0.0/16");
```

## Load Balancer Request Example

```rust
use cloud_sdk_hetzner::cloud::load_balancers::{
    LoadBalancerAlgorithm, LoadBalancerCreateRequest, LoadBalancerName,
    LoadBalancerType,
};

# fn main() -> Result<(), cloud_sdk_hetzner::cloud::load_balancers::LoadBalancerRequestError> {
let name = LoadBalancerName::new("public-edge")?;
let load_balancer_type = LoadBalancerType::new("lb11")?;
let request = LoadBalancerCreateRequest::new(name, load_balancer_type)
    .with_algorithm(LoadBalancerAlgorithm::LeastConnections)
    .with_public_interface(true);

let mut path = [0u8; 32];
let written = request.endpoint().write_path(&mut path)?;
let path = path
    .get(..written)
    .and_then(|value| core::str::from_utf8(value).ok());

assert_eq!(request.endpoint().method().as_str(), "POST");
assert_eq!(path, Some("/load_balancers"));
# Ok(())
# }
```

## DNS Zone Request Example

```rust
use cloud_sdk_hetzner::dns::zones::{
    ZoneCreateMode, ZoneCreateRequest, ZoneName, ZoneTtl,
};

# fn main() -> Result<(), cloud_sdk_hetzner::dns::zones::ZoneRequestError> {
let name = ZoneName::new("example.com")?;
let ttl = ZoneTtl::new(3600)?;
let request = ZoneCreateRequest::new(name, ZoneCreateMode::Primary)
    .with_ttl(ttl);

let mut path = [0u8; 16];
let written = request.endpoint().write_path(&mut path)?;
let path = path
    .get(..written)
    .and_then(|value| core::str::from_utf8(value).ok());

assert_eq!(request.endpoint().method().as_str(), "POST");
assert_eq!(request.ttl().map(ZoneTtl::get), Some(3600));
assert_eq!(path, Some("/zones"));
# Ok(())
# }
```

## Security And Operations

### Live Smoke Harness

The repository provides an ignored, read-only live harness for selected public
catalog endpoints. It requires a dedicated Hetzner project, a read-only token,
and the documented root-sealed build and private token-file workflow. The
harness never belongs in downstream crate builds and destructive execution is
disabled. Follow
[`LIVE_SMOKE_TESTING.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/LIVE_SMOKE_TESTING.md)
for setup, execution, and cleanup.

### Sensitive Output Buffers

`ZoneFile::write_json_string`, `TsigKey::write_json_string`,
`UserData::write_json_string`, `StorageBoxPassword::write_json_string`, and
`PrivateKeyPem::write_json_string` copy potentially sensitive values into
caller-owned buffers. Wrap the complete destination in
`cloud_sdk_sanitization::SecretBuffer` so it is volatile-cleared after
transport use, including on early returns. The SDK cannot erase source strings
or downstream copies it does not own.

### TSIG Policy

The hardened API supports only HMAC-SHA256. HMAC-MD5 is prohibited and
HMAC-SHA1 is intentionally excluded. TSIG secrets must use canonical padded
Base64 and decode to at least 32 bytes. Generate them with a CSPRNG, share them
only with the intended peer, and rotate them periodically; representation
validation cannot establish entropy.

`ZoneFile`, `TsigKey`, `TsigCredentials`, and request structures containing
them intentionally omit ordinary equality. Use a reviewed constant-time
mechanism if external secret comparison is required. RFC 8945 defines the
[algorithm requirements](https://www.rfc-editor.org/rfc/rfc8945.html#section-6)
and [shared-secret requirements](https://www.rfc-editor.org/rfc/rfc8945.html#section-8).

### RRSet Validation Policy

The SDK validates names, supported RR types, TTLs, record counts and
uniqueness, control and bidi characters, paths, and JSON escaping. It does not
normalize every record type's complete RDATA grammar. Callers remain
responsible for values accepted by Hetzner's
[DNS record type documentation](https://docs.hetzner.com/networking/dns/record-types/overview/).

Uniqueness uses exact value bytes because RR-type-neutral handling cannot
case-fold domain names without changing case-sensitive records such as `TXT`.
Canonicalize domain-name values before construction when semantic,
case-insensitive uniqueness is required. The optional Serde wrapper enforces a
1 MiB JSON bound before serialization; transports must retain an independently
reviewed body limit.
