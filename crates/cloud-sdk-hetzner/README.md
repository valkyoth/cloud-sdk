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
It source-locks the Hetzner Cloud/DNS and Storage Box OpenAPI specs, owns the
Hetzner endpoint module layout, and will receive the Hetzner request/response
models in small reviewed releases.

## Install

```toml
[dependencies]
cloud-sdk = "0.23.0"
cloud-sdk-hetzner = "0.19.1"
```

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps provider models allocation-free, transport-free, and `no_std`. |
| `alloc` | no | Enables provider APIs that require the Rust `alloc` crate. |
| `serde` | no | Enables the reviewed no_std Serde request and response boundary; also enables `alloc`. |
| `std` | no | Enables `alloc` and standard-library integration without selecting a transport. |

Docs.rs builds with all features. The default dependency graph still includes
no network client, TLS implementation, async runtime, filesystem, or clock.

## Workflow Examples

Compile-checked read-only, mutation, pagination, action polling, DNS, and
Storage Box examples are indexed in the
[Hetzner workflow guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/HETZNER_EXAMPLES.md).
Security-sensitive transport decisions are covered by the
[security recipes](https://github.com/valkyoth/cloud-sdk/blob/main/docs/SECURITY_RECIPES.md).

## Current Scope

Published package snapshots are listed on
[crates.io](https://crates.io/crates/cloud-sdk-hetzner). Repository development
status and independent crate versions are tracked in the
[crate version matrix](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATE_VERSION_MATRIX.md).
This crate remains no_std and does not itself implement HTTP transport, broad
Serde coverage outside reviewed RRSet/shared response models, token storage,
destructive live tests, automatic retries, sleeps, or page fetching. Blocking
and async HTTP implementations belong to the provider-neutral
`cloud-sdk-reqwest` crate.

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

Implemented in the published `0.5.0` line:

- SSH key list/create/get/update/delete request primitives;
- certificate list/create/get/update/delete request primitives;
- certificate retry action request primitive;
- validation for names, SSH public keys, PEM values, managed certificate domain
  names, labels, pagination, and source-locked sorting;
- redacted `Debug` output for secret-adjacent SSH public key and certificate
  PEM request values.

Implemented in the published `0.6.0` line:

- server list/create/get/update/delete request primitives;
- server metrics request primitives with time range validation;
- server action endpoint paths and request markers for all source-locked server
  actions;
- explicit DNS pointer set/reset intent for deprecated omitted `dns_ptr`
  behavior.

Implemented in the published `0.7.0` line:

- image list/get/update/delete request primitives;
- image action list/get, per-image action list, and protection action paths;
- placement group list/create/get/update/delete request primitives;
- primary IP list/create/get/update/delete request primitives;
- primary IP assign, unassign, DNS pointer, and protection action paths;
- explicit primary IP DNS pointer set/reset intent;
- no public create/update fields for removed datacenter request parameters.

Implemented in the published `0.8.0` line:

- volume list/create/get/update/delete request primitives;
- volume action list/get, per-volume action list, attach, detach, resize, and
  protection action paths;
- bounded volume size markers for the source-locked `10..=10240` GB range;
- floating IP list/create/get/update/delete request primitives;
- floating IP assign, unassign, DNS pointer, and protection action paths;
- explicit volume and floating IP server/location placement markers;
- explicit floating IP DNS pointer set/reset intent.

Implemented in the published `0.9.0` line:

- Storage Box list/create/get/update/delete and folder-list request primitives;
- Storage Box type list/get request primitives;
- Storage Box snapshot list/create/get/update/delete request primitives;
- Storage Box subaccount list/create/get/update/delete request primitives;
- Storage Box and subaccount action endpoint paths;
- redacted Storage Box password markers, bounded snapshot-plan markers, and
  conservative subaccount home-directory validation.

Implemented in the published `0.10.0` line:

- Firewall list/create/get/update/delete request primitives;
- Firewall apply/remove resource and set-rules action request primitives;
- direction-specific canonical source/destination CIDR selectors, protocols,
  ports, descriptions, rule limits, and duplicate conflict validation;
- Network list/create/get/update/delete request primitives;
- Network route, subnet, IP range, and protection action request primitives;
- canonical RFC 1918 range, route destination, private gateway, vSwitch, and
  CIDR boundary validation.

Implemented in the published `0.11.0` line:

- Load Balancer list/create/get/update/delete and metrics request primitives;
- service add/update/delete models for TCP, HTTP, and HTTPS;
- bounded health checks, sticky-session settings, certificate selection, and
  redirect behavior;
- server, label-selector, and direct-IP target add/remove models;
- network attach/detach, reverse-DNS, protection, algorithm, type-change, and
  public-interface action models;
- explicit reverse-DNS set/reset intent and deterministic multi-metric query
  construction.

Implemented in the published `0.12.0` line:

- Zone list/create/get/update/delete and zonefile export request primitives;
- global and per-Zone action lists plus global action lookup;
- zonefile import, primary nameserver replacement, deletion protection, and
  explicit TTL-change action models;
- bounded lowercase Zone names, default TTLs, zonefiles, public primary
  nameservers, hardened TSIG keys, and deterministic Zone queries;
- redacted zonefile and TSIG debug output, fixed-buffer paths, and structural
  primary/secondary Zone creation modes.

Implemented in the published `0.13.0` line:

- RRSet list/create/get/update/delete request primitives;
- RRSet protection, TTL, set-records, add-records, remove-records, and
  update-record-comments action request primitives;
- all 16 source-locked RR types, relative lowercase names, apex and wildcard
  path encoding, repeated type filters, pagination, labels, and sorting;
- mandatory explicit TTL or JSON-null inheritance for change-TTL, with
  optional TTL intent retained only where the source schema permits omission;
- bounded, debug-redacted record values/comments, `1..=50` unique-value
  mutation lists, and atomic fixed-buffer JSON-string writers.

Implemented in the published `0.14.0` line:

- opt-in `serde` feature with Serde defaults and `std` disabled;
- size-checked JSON serialization wrappers for every RRSet create, update,
  protection, TTL, set, add, remove, and comment-update body;
- validated borrowed-or-owned action and API error response envelopes;
- duplicate and missing known response fields rejected, additive unknown
  response fields ignored, and escaped JSON strings supported through `Cow`;
- 8 MiB pre-parser response input, 256 action-resource, and interpreted response
  text bounds;
- automated proof that Serde and serde_json remain outside the normal default
  dependency graph.

Implemented in the published `0.15.0` line:

- integration tests that apply the provider-neutral adversarial response
  corpus to real Hetzner Serde response parsing;
- compatibility with the provider-neutral blocking transport request and
  caller-owned response-buffer contract without adding transport dependencies.

Implemented in the published `0.16.0` line:

- strict reusable `meta.pagination` parsing with required nullable fields,
  additive-field compatibility, and validated navigation;
- source-locked page defaults and limits of 25 and 50;
- conversion from Hetzner page metadata into the provider-neutral bounded
  pagination cursor;
- conversion from validated Hetzner action responses into polling updates that
  preserve terminal provider errors.

Implemented in the published `0.17.0` line:

- ignored opt-in live smoke coverage for locations, server types, load
  balancer types, ISOs, public system images, and pricing;
- typed GET-only request construction through this provider crate and the
  hardened provider-neutral blocking reqwest/rustls transport;
- fixed official origin, root-sealed build-before-credential execution,
  private regular token-file input, bounded response storage, static redacted
  diagnostics, and source-buffer cleanup;
- offline adversarial coverage for target construction, response envelopes,
  pagination, token normalization, size bounds, symlinks, and Unix modes.

### Live Smoke Harness

The live test is ignored by default and is run from the main repository, not
by downstream crate builds. Use a dedicated Hetzner test project and a token
with **Read** permission. Prepare the sealed test executable from a clean commit
before the token exists or is mounted, then install it and its runtime into the
documented root-owned system paths. Authenticated execution validates ownership
and permissions, hashes an open descriptor, executes that same descriptor, and
never invokes Cargo. The token value is accepted only through a private file
path and must never be placed directly in shell history or an environment
variable. See the repository's
[`LIVE_SMOKE_TESTING.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/LIVE_SMOKE_TESTING.md)
for setup, execution, cleanup, and the separately disabled destructive plan.

### Sensitive Output Buffers

`ZoneFile::write_json_string`, `TsigKey::write_json_string`,
`UserData::write_json_string`, `StorageBoxPassword::write_json_string`, and
`PrivateKeyPem::write_json_string` copy potentially sensitive values into
caller-owned buffers. Wrap the entire destination in
`cloud_sdk_sanitization::SecretBuffer` before writing so it is volatile-cleared
after transport use, including on early returns. The SDK cannot erase source
strings or downstream copies it does not own; ordinary buffer writes are not a
secure-erasure mechanism.

### TSIG Policy

The hardened API supports only HMAC-SHA256. HMAC-MD5 is prohibited and
HMAC-SHA1 is intentionally excluded even though Hetzner accepts both for legacy
interoperability. TSIG secrets must decode to at least 32 bytes, use canonical
padded Base64, and should be generated with a CSPRNG, shared by only two
entities, and rotated periodically. Representation checks cannot establish
entropy.

`ZoneFile`, `TsigKey`, `TsigCredentials`, and request structures containing
them intentionally do not implement ordinary equality. Do not compare secrets
with normal string equality; use a reviewed constant-time mechanism if secret
comparison is required outside this request-building crate. See RFC 8945 for
the [algorithm requirements](https://www.rfc-editor.org/rfc/rfc8945.html#section-6),
[local policy](https://www.rfc-editor.org/rfc/rfc8945.html#section-7), and
[shared-secret requirements](https://www.rfc-editor.org/rfc/rfc8945.html#section-8).

### RRSet Validation Policy

The SDK validates RRSet names, the 16 source-locked RR types, TTL bounds,
record-list count and uniqueness, control and bidi characters, fixed-buffer
paths, and JSON escaping. It deliberately does not normalize or reinterpret
the complete RDATA grammar for every record type. Create requests use the same
conservative 50-record ceiling as mutation actions. Callers remain responsible
for supplying values accepted by Hetzner's
[DNS record type documentation](https://docs.hetzner.com/networking/dns/record-types/overview/).

Record uniqueness uses exact value bytes. The RR-type-neutral list cannot
case-fold domain-name-valued records without also corrupting the semantics of
case-sensitive RDATA such as `TXT`. Canonicalize domain-name values before
constructing records when semantic, case-insensitive uniqueness is required.
The optional Serde wrapper applies a conservative 1 MiB JSON upper bound before
serialization. Future transports must preserve an independently reviewed body
limit rather than assuming per-record bounds are sufficient.

Record values and comments expose complete JSON-string writers instead of raw
string accessors. The optional Serde implementation serializes those validated
types directly so quotes and backslashes cannot be interpolated unsafely.

## Optional Serde Boundary

Enable Serde explicitly; it is never part of the default graph:

```toml
[dependencies]
cloud-sdk-hetzner = { version = "0.19.1", features = ["serde"] }
```

`serde_json` is used below only as an example format implementation and remains
a dev dependency in this repository:

```rust
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
```

Before deserializing an untrusted response, construct
`cloud_sdk_hetzner::serde::ResponseBytes` and pass only its admitted slice to
the selected format parser. Direct parser use bypasses the SDK's 8 MiB raw
response policy.

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
let request = RrsetCreateRequest::try_new(
    zone,
    Some(name),
    Some(RrsetType::A),
    Some(records),
)?;

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

## Pagination Response Example

The optional Serde boundary can extract shared pagination metadata from any
Hetzner list response while ignoring the resource-specific fields:

```rust
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
```

Pass `TransportResponse::rate_limit()` as the final `observe` argument when a
real or mock transport supplies it. The caller remains responsible for
decoding the resource array and reporting its exact entry count.

## Action Polling Example

```rust
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
let request = SshKeyCreateRequest::try_new(Some(name), Some(public_key))?;

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
let request = ServerCreateRequest::try_new(Some(name), Some(server_type), Some(image))?;

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
let request = match NetworkCreateRequest::try_new(Some(name), Some(ip_range)) {
    Ok(request) => request,
    Err(_) => return,
};

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
let request = LoadBalancerCreateRequest::try_new(Some(name), Some(load_balancer_type))?
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
let request = ZoneCreateRequest::try_new(Some(name), Some(ZoneCreateMode::Primary))?
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
