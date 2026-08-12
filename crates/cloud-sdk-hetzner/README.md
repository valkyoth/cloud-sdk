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
cloud-sdk = "0.80.0"
cloud-sdk-hetzner = "0.43.0"
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

## Provider Identity

Hetzner owns its provider and service markers:

| Marker | Canonical ID |
| --- | --- |
| `Hetzner` | `hetzner` |
| `CloudService` | `cloud` |
| `DnsService` | `dns` |
| `SecurityService` | `security` |
| `StorageService` | `storage` |
| `RobotService` | `robot` |

Prepared Cloud and Console Storage operations bind the appropriate marker and
official endpoint automatically. Callers comparing metadata can use
`HETZNER_PROVIDER_ID`, `CLOUD_SERVICE_ID`, `DNS_SERVICE_ID`,
`SECURITY_SERVICE_ID`, `STORAGE_SERVICE_ID`, and `ROBOT_SERVICE_ID`. These IDs
are routing
metadata; exact endpoint verification remains a separate mandatory credential
boundary.

## Workflow Examples

Compile-checked read-only, mutation, pagination, action polling, DNS, and
Storage Box examples are indexed in the
[Hetzner workflow guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/HETZNER_EXAMPLES.md).
Security-sensitive transport decisions are covered by the
[security recipes](https://github.com/valkyoth/cloud-sdk/blob/main/docs/SECURITY_RECIPES.md).
The compile-checked
[`cloud_client` example](https://github.com/valkyoth/cloud-sdk/blob/main/crates/cloud-sdk-hetzner/examples/cloud_client.rs)
shows official construction, bounded caller-owned storage, a named Cloud read,
and typed response handling against the deterministic testkit transport.
The matching
[`dns_client` example](https://github.com/valkyoth/cloud-sdk/blob/main/crates/cloud-sdk-hetzner/examples/dns_client.rs)
shows numbered DNS pagination through the named official client path.
The
[`security_client` example](https://github.com/valkyoth/cloud-sdk/blob/main/crates/cloud-sdk-hetzner/examples/security_client.rs)
shows certificate pagination through the named official Security client path.
The
[`storage_client` example](https://github.com/valkyoth/cloud-sdk/blob/main/crates/cloud-sdk-hetzner/examples/storage_client.rs)
shows numbered Console Storage pagination through the named official Storage
client path.

Robot server reads and renames use only the canonical positive server number.
The deprecated IPv4 path aliases are intentionally unavailable. The `alloc`
feature provides stable protected request identities; request preparation does
not allocate after identity construction. Strict owned success models require
`serde`, which includes `alloc`:

```rust
# #[cfg(feature = "alloc")]
# fn main() -> Result<(), Box<dyn core::error::Error>> {
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk_hetzner::robot::{RobotServerGetRequest, RobotServerNumber};

let number = RobotServerNumber::new(321)?;
let request = RobotServerGetRequest::new(number);
let mut target = [0_u8; 64];
let mut body = [0_u8; 1];
let prepared = request.prepare(PreparationStorage::new(&mut target, &mut body))?;

assert_eq!(prepared.transport_request().target().as_str(), "/server/321");
# Ok(())
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

`RobotServerListRequest`, `RobotServerGetRequest`, and
`RobotServerUpdateRequest::rename` bind the official Robot origin, service and
Basic-auth scope, form media type, explicit operation impact, retry policy,
and checked `200`/JSON response policy. Their decode methods consume a
`CheckedResponseGuard`, clear response storage, reject a mismatched server
number, and return bounded `RobotServerList` or `RobotServer` models.
Operationally sensitive IDs, addresses, subnets, dates, states, cancellation,
and capability flags are non-`Copy`, stable-allocation-backed values with
redacted diagnostics. Moving an SDK model transfers allocation metadata rather
than copying classified bytes to another inline location. The strict decoder
retains numbers and Booleans in protected lexical/fixed storage, parses Robot
topology and dates through bounded clear-on-drop scratch, and writes request
identity paths directly from protected decimal bytes. Protected Boolean flags
copy directly into their final stable allocation, and protected parser
allocation failures remain distinct from malformed provider data. Inspect IDs,
addresses, subnets, and dates through the documented closure-scoped accessors;
any scalar copy retained by caller code is outside the SDK cleanup boundary.

Robot cancellation support covers all nine server, IP, and subnet get,
create, and revoke operations. Create requests require an explicit immediate
or calendar-date schedule; server requests also require explicit location
reservation intent. Create and revoke metadata is destructive and never
automatically retryable. Responses bind identity to the request, reject
contradictory dates and state, require mutation acknowledgement to match the
requested schedule/reason/reservation intent and reservation availability,
and preserve the official
IP/subnet date-field spelling inconsistency without accepting both spellings at
once. Server revoke has an empty success response; IP and subnet revoke return
and validate inactive cancellation models.

Robot IP management covers list, detail, traffic-policy update, and
separate-MAC get, generate, and delete. Requests admit only canonical protected
addresses, non-empty partial traffic forms, and canonical lowercase EUI-48
responses. Checked decoding binds optional server filters and exact resource
identity, rejects duplicate or oversized lists and inconsistent network fields,
and verifies requested threshold and nullable-MAC outcomes. Traffic and MAC
mutations use request-bound direct/shared permits; MAC generation and deletion
are never automatically retried.

Robot subnet management covers list, detail, traffic-policy update, and
subnet-MAC get, explicit assignment, and default restoration. Responses admit
the documented nullable server assignment and host-bits-set route identities,
while enforcing canonical address spelling, family-valid masks, gateway
membership, bounded duplicate-free inventories, and bounded canonical
address-to-MAC choices. The mathematical network and IPv4 broadcast boundary
are derived accessors rather than assumed route identities. Traffic updates
and explicit MAC assignment use sensitive forms and strong-digest permits;
MAC assignment and restoration are never automatically retried.
Default restoration requests can only be constructed by consuming checked
subnet and MAC snapshots, their bounded observation window, and an external
per-subnet mutation-lock lease. The assigned server address selects the
expected default MAC. All evidence is bound into a strong digest, is checked
at permit entry and immediately before transport dispatch, and bounds the
permit lifetime. Async checks run on first poll using the generic permit
check's clock sample. DELETE success must preserve that mapping and return that
exact MAC. Exact fingerprints are rejected for this sensitive evidence.
The SDK verifies lock identity, resource, and expiry; callers must obtain the
lease from a lock service that serializes every mutation of the subnet. Each
subnet request also exposes operation-specific failure
decoding, so documented `404` and `500` codes are typed without admitting a
code for the wrong operation.

Traffic policy and update types are non-`Copy`, redact `Debug`, return borrowed
aggregate views, and clear their owned scalar storage on drop. Scalar values
explicitly returned by accessors become caller-owned copies.

Robot reset management covers capability list, checked detail, and disruptive
execution. The finite reset types are `sw`, `hw`, `power`, `power_long`, and
`man`; unknown or duplicate capabilities fail closed. Execution has no public
server-number-only constructor: callers must execute an authenticated detail
preflight and select an advertised capability from its 30-second
`AuthorizedRobotReset`. Raw-decoded `RobotReset` values are inspectable but
non-authorizing. The evidence binds the transport's opaque credential lineage,
both addresses, server number, capability, observation, and expiry into the
strong digest, then rechecks credential and expiry immediately before dispatch.
Its sensitive form is destructive,
non-idempotent, never automatically retried, and requires a request-bound
strong-digest permit. Success binds IPv4, IPv6 network, any returned server
number, and the exact requested reset type. The official action example omits
`server_number` despite the output table listing it, so only that field is
narrowly optional. Success bodies are independently capped at 2 MiB for lists,
4 KiB for details, and 2 KiB for actions.

```rust
# #[cfg(feature = "serde")]
# fn main() -> Result<(), Box<dyn core::error::Error>> {
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{
    RobotCancellationSchedule, RobotIpAddress, RobotIpCancellationCreateRequest,
};

let ip = RobotIpAddress::new("192.0.2.10")?;
let request = RobotIpCancellationCreateRequest::new(
    ip,
    RobotCancellationSchedule::Immediate,
);
let mut target = [0_u8; 96];
let mut body = [0_u8; 64];
let prepared = request.prepare_bound(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(
    prepared.as_untyped().transport_request().method(),
    cloud_sdk::Method::Post,
);
# Ok(())
# }
# #[cfg(not(feature = "serde"))]
# fn main() {}
```

Cancellation create and revoke requests are destructive. Move their
`PreparedCancellation` into `CancellationPlanConfirmation` and execute a
`CancellationDestructivePermit` or `CancellationSharedDestructivePermit`
attempt. POST bodies are sensitive and must use
`build_cancellation_plan_digest`; exact canonical fingerprint construction
fails closed for POST. Bodyless revoke requests may use the exact canonical or
strong-digest builder. These request-bound wrappers return
`CheckedCancellation` directly across blocking, Send-async, and local-async
execution. Keep caller-owned reconciliation after uncertain delivery; v0.79
does not provide the Robot high-level client.

Robot error responses use a separate strict decoder. Pass only an admitted
transport response; unknown statuses, unknown codes, duplicate keys, and
invalid content types fail closed:

```rust,no_run
#[cfg(feature = "serde")]
use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};
#[cfg(feature = "serde")]
use cloud_sdk_hetzner::robot::{RobotDecodeError, RobotFailure, decode_robot_failure};

#[cfg(feature = "serde")]
fn classify_robot_error(
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let failure = decode_robot_failure(response, workspace)?;
    assert!(!failure.allows_automatic_retry());
    Ok(failure)
}
```

Authentication rejection is structurally distinct and always has
`RobotRetryDisposition::Never`. Quota, maintenance, and explicitly supplied
transport failures still require caller-owned retry policy.

Robot POST bodies use a separate bounded form codec. Ordered duplicate names
remain duplicate wire fields, and the returned guard clears the complete
caller buffer when it leaves scope:

```rust
use cloud_sdk_hetzner::robot::{RobotForm, RobotFormField};

let fields = [
    RobotFormField::public("server[]", "192.0.2.10")?,
    RobotFormField::public("server[]", "192.0.2.11")?,
    RobotFormField::sensitive("password", "example-only-secret")?,
];
let form = RobotForm::new(&fields)?;
let mut output = [0_u8; 256];
{
    let encoded = form.encode(&mut output)?;
    assert_eq!(
        encoded.as_bytes(),
        b"server%5B%5D=192.0.2.10&server%5B%5D=192.0.2.11&password=example-only-secret",
    );
    // Send while `encoded` owns the mutable output borrow.
}
assert!(output.iter().all(|byte| *byte == 0));
# Ok::<(), Box<dyn core::error::Error>>(())
```

The codec performs exact checked preflight, preserves output on validation or
capacity failure, wipes stale tail bytes before an admitted write, and uses
standard form rules: spaces become `+`, while literal `+`, `&`, `=`, brackets,
controls, and non-ASCII UTF-8 bytes are percent encoded. Field names require a
nonempty identifier root followed only by complete bracketed components;
`server[]` remains valid while malformed nesting fails before encoding. The
codec does not send a request or own source secrets. With `alloc`, use the
protected Robot-only credential owner for Basic authentication material:

```rust
# #[cfg(feature = "alloc")]
# fn main() -> Result<(), Box<dyn core::error::Error>> {
use cloud_sdk::authentication::CredentialReconfirmation;
use cloud_sdk_hetzner::robot::RobotCredentials;

let mut username = b"robot-user".to_vec();
let mut password = b"example-only-secret".to_vec();
let credentials = RobotCredentials::from_mut_bytes(&mut username, &mut password)?;
assert!(username.iter().all(|byte| *byte == 0));
assert!(password.iter().all(|byte| *byte == 0));

let attempt = credentials.begin_attempt()?;
credentials.try_with_attempt(&attempt, |username, password| {
    // Encode and send only through an exact official Robot endpoint here.
    assert_eq!(username, "robot-user");
    assert_eq!(password, "example-only-secret");
})?;

// An authentication rejection closes this generation globally.
credentials.reject_attempt(&attempt)?;
let _next_generation = credentials.reconfirm(
    CredentialReconfirmation::acknowledge_same_credentials(),
)?;
# Ok(())
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

Only newly supplied credentials or explicit caller reconfirmation reopen a
rejected generation. Retry, polling, and client policy must never create that
decision. The type does not build a Basic header or send a request.

Use compile-time operation associations when endpoint, query, body, response,
and safety policy must retain one nominal operation identity:

```rust
use cloud_sdk::operation::PreparationStorageGuard;
use cloud_sdk_hetzner::actions::{ActionEndpoint, ActionId};
use cloud_sdk_hetzner::association::AssociatedOperation;
use cloud_sdk_hetzner::association::operations::GetAction;

let id = ActionId::new(7).ok_or("invalid action ID")?;
let operation = AssociatedOperation::<GetAction, _>::endpoint(
    ActionEndpoint::Get(id),
)?;
let mut target = [0_u8; 64];
let mut body = [0_u8; 1];
let mut storage = PreparationStorageGuard::new(&mut target, &mut body);
let prepared = operation.prepare_typed_guarded(&mut storage)?;

assert_eq!(prepared.association().operation_id().as_str(), "get_action");
# Ok::<(), Box<dyn core::error::Error>>(())
```

`EndpointFor<O, _>`, `QueryFor<O, _>`, and `BodyFor<O, _>` cannot be combined
across different operation markers. `Prepared<O>` retains the exact service,
official endpoint, authentication scope, request and response policy,
pagination, quota, retry, streaming, response/error, and permit associations.
See the
[operation association guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/OPERATION_ASSOCIATIONS.md).
The generated
[complete binding manifest](https://github.com/valkyoth/cloud-sdk/blob/main/docs/TYPED_OPERATION_BINDINGS.tsv)
makes all 208 active operation contracts reviewable without reading generated
Rust source.

With `serde`, `HetznerClient::cloud` exposes named blocking, `Send` async, and
local-async methods for all 139 active Cloud operations. Read-only calls use a
`ClientWorkspaceLease` and return a fully checked, owned response. Mutation,
destructive, and cost-bearing calls expose named preparation plus execution
that accepts only the matching plan-confirm permit attempt. The client selects
no retry policy, runtime, clock, transport, or secret store.

Only `NoPermit` read-only markers expose direct typed execution. Mutation,
destructive, and cost-bearing markers use `AssociatedPlanConfirmation`, an
associated exact or strong-digest fingerprint, and the corresponding
`Associated*Permit`. Successful permit execution returns
`AssociatedCheckedResponse<O>`, preserving endpoint-derived response identity
through typed decoding. Explicit type erasure does not bypass the neutral
runtime permit boundary, but it discards this provider identity binding.
The neutral layer also requires `GET` or `HEAD` for direct read-only execution;
all other wire methods remain permit-gated independently of provider metadata.
See the
[execution permit guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/EXECUTION_PERMITS.md).

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
assert_eq!(
    prepared.transport_request().headers().get("accept")
        .map(|header| header.value().as_str()),
    Some("application/json"),
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

Secret-bearing operations need successful-path cleanup after transport use.
Guard both complete preparation buffers through the core API:

```rust
use cloud_sdk::operation::{PreparationStorageGuard, PrepareOperation};
use cloud_sdk_hetzner::storage::storage_boxes::{
    StorageBoxCreateRequest, StorageBoxLocation, StorageBoxName,
    StorageBoxPassword, StorageBoxTypeRef,
};
let name = StorageBoxName::new("backup")?;
let location = StorageBoxLocation::new("fsn1")?;
let box_type = StorageBoxTypeRef::new("bx20")?;
let password = StorageBoxPassword::new("example-only-not-a-real-secret")?;
let operation = StorageBoxCreateRequest::new(name, location, box_type, password);
let mut target = [0_u8; 128];
let mut body_bytes = [0_u8; 512];
{
    let mut storage = PreparationStorageGuard::new(&mut target, &mut body_bytes);
    let prepared = storage.prepare(&operation)?;
    assert!(!prepared.transport_request().body().is_empty());
    // Send while `prepared` borrows `storage`, then leave this scope.
}
assert!(target.iter().all(|byte| *byte == 0));
assert!(body_bytes.iter().all(|byte| *byte == 0));
# Ok::<(), Box<dyn core::error::Error>>(())
```

Every `prepare` call clears the guard's complete target and body buffers before
writing. Reusing the guard cannot retain a longer earlier request in the tail
of a shorter later request.

Use `official_endpoint_policy(expected_base)` with a policy-aware transport
constructor. Prepared operations carry that same provider-owned fixed policy.
For an existing custom transport, `verify_official_endpoint` fails closed
unless scheme, host, effective port, and base path exactly match the selected
official Cloud or Storage API endpoint. `verify_any_official_endpoint` checks
the bounded two-endpoint set for provider-wide diagnostics, not operation
execution.

## Request Operation Coverage

The current release has complete prepared-request coverage for all 208
source-locked non-deprecated Cloud, DNS, and Storage Box operations. Each
prepared operation binds its method, target, bounded body, response policy,
safety and retry classification, cost intent, exact provider service,
authentication scope, raw response policy, and official endpoint.

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
| Success response models | Complete checked envelopes for all 208 operations; source-complete ordinary Cloud resources, DNS zones and RRSets, zonefiles, actions, metrics, composites, pricing, locations, certificates, SSH keys, and Console Storage Boxes, types, snapshots, subaccounts, and folders; operation-branded typed execution guards decode through `decode_associated_checked_response` | Current |
| Error response models | Complete checked typed API error decoding for all active operations | Current |
| End-to-end client | Complete named workflows for all 208 active Cloud, DNS, Security, and Console Storage Box operations; custom-endpoint execution remains unavailable | Current |

Thirteen deprecated operations remain deliberately unavailable. A checked
release gate prevents non-deprecated request operations from returning to a
planned or deferred state. See the
[API matrix](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_MATRIX.md)
for operation-level request status and the
[release plan](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_PLAN.md)
for prepared-request, serialization, response, and client milestones.
The construction, storage, and trust boundaries are described in the
[Hetzner client guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/HETZNER_CLIENT.md).
Upstream source monitoring and lock-refresh decisions follow the
[API drift maintenance runbook](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_DRIFT_MAINTENANCE.md).
The separate Robot Webservice exposes its bounded form codec, exact official
endpoint identity, Robot service marker, protected credentials, lockout-aware
owned attempt generation, strict typed protocol errors, three server
list/get/rename operations, all nine cancellation operations, and all six IP
and separate-MAC operations. It does not yet expose the later Robot endpoint
families or a high-level Robot client. Its
complete source lock records 89
active operations and excludes all 16 deprecated Storage Box operations. See the
[Robot source-lock contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/ROBOT_WIRE_SOURCE_LOCK.md).
The latest Robot additions and source migration are described in the
[v0.80 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.80.0.md).
Breaking v0.27 constructor and custom-endpoint changes are listed in the
[migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.27.0.md).
Shared transport and credential lifecycle changes are listed in the
[v0.29 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.29.0.md).
The complete method domain and explicit operation metadata migration are listed
in the
[v0.33 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.33.0.md).
Endpoint trust-policy migration is listed in the
[v0.34 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.34.0.md).
Canonical request-target migration is listed in the
[v0.35 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.35.0.md).
Prepared request-header migration is listed in the
[v0.36 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.36.0.md).
Response provenance migration is listed in the
[v0.37 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.37.0.md).
Mandatory response cleanup migration is listed in the
[v0.38 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.38.0.md).
Transactional request encoding and preparation cleanup are listed in the
[v0.39 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.39.0.md).
Bearer authentication scope and generation-safe refresh are listed in the
[v0.41 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.41.0.md).
Basic authentication and canonical signing input additions are listed in the
[v0.42 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.42.0.md).
The authenticated raw-wire migration is listed in the
[v0.43 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.43.0.md).
The numbered pagination migration and provider-neutral strategy family are
listed in the
[v0.44 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.44.0.md).
Provider-owned quota decoding and pure delay policy are described in the
[v0.45 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.45.0.md)
and the
[quota and retry guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/QUOTA_AND_RETRY.md).
Exact replay identity and source-locked retry classes are described in the
[v0.46 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.46.0.md)
and the
[retry and idempotency guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RETRY_AND_IDEMPOTENCY.md).
Local `!Send` prepared execution and cancellation policy are described in the
[v0.47 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.47.0.md)
and the
[local async guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/LOCAL_ASYNC.md).

## Optional Serde Boundary

Enable Serde explicitly; it is never part of the default graph:

```toml
[dependencies]
cloud-sdk-hetzner = { version = "0.43.0", features = ["serde"] }
```

The feature admits serde_json with `default-features = false` and `alloc` only
for the public Serde request/envelope APIs. Checked SSH-key responses reuse
exact `base64-ng` and admit exact `md-5` plus `sha2`, all without defaults;
none enters the default provider graph. A private bounded parser validates the
exact supported algorithm set and RFC 4253 structure directly inside one
cleanup-owned decoded allocation. Checked responses never route decoded
string values through serde_json heap scratch storage. The decoder consumes a
cleanup-owning `ResponseBuffer`
together with its exact `PreparedRequest`, applies the prepared
status/content-type/body policy, rejects duplicate or malformed JSON, and
returns validated typed success or API errors only after response storage is
cleared.
Checked resource responses are source-complete. SSH public keys receive
complete OpenSSH/RFC 4253 parsing,
legacy provider fingerprints are bound to the parsed key, and callers receive
an SDK-computed SHA-256 fingerprint for identity comparisons. The bounded
parser tree and its volatile-clearing string storage remain private:

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

Decode only through the prepared request that produced the response. This
example adds a platform hook; core cleanup remains mandatory without one:

```rust
# #[cfg(feature = "serde")]
# fn main() -> Result<(), Box<dyn core::error::Error>> {
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{
    HeaderSensitivity, ResponseBuffer, ResponseMetadata, ResponseStorageSanitizer,
    StatusCode,
};
use cloud_sdk_hetzner::cloud::servers::placement_groups::{
    PlacementGroupEndpoint, PlacementGroupId,
};
use cloud_sdk_hetzner::serde::{CloudResourceKind, HetznerSuccess, decode_response};

let endpoint = PlacementGroupEndpoint::Get(
    PlacementGroupId::new(42).ok_or("invalid ID")?,
);
let mut target = [0_u8; 64];
let mut body = [];
let prepared = endpoint.prepare(PreparationStorage::new(&mut target, &mut body))?;
let response_body = br#"{"placement_group":{"id":42,"name":"group-1","labels":{},"type":"spread","created":"2026-08-08T00:00:00Z","servers":[]}}"#;
let mut response_storage = [0_u8; 128];
let mut response_header_storage =
    [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
let mut response = ResponseBuffer::with_additive_sanitizer(
    &mut response_storage,
    128,
    &mut response_header_storage,
    &Sanitizer,
);
{
    let mut attempt = response.writer().begin_attempt()?;
    let output = attempt.body_mut()?;
    let output = output
        .get_mut(..response_body.len())
        .ok_or("response buffer is too small")?;
    output.copy_from_slice(response_body);
    attempt.headers_mut()?.try_push(
        "content-type",
        b"application/json",
        HeaderSensitivity::Public,
    )?;
    attempt.commit(
        StatusCode::OK,
        response_body.len(),
        ResponseMetadata::EMPTY,
    )?;
}
let decoded = decode_response(prepared, response)?;

let HetznerSuccess::CloudResource(group) = decoded.success() else {
    return Err("unexpected response family".into());
};
assert_eq!(group.kind(), CloudResourceKind::PlacementGroup);
assert_eq!(group.name(), Some("group-1"));

struct Sanitizer;
impl ResponseStorageSanitizer for Sanitizer {
    fn sanitize_response_storage(&self, storage: &mut [u8]) {
        cloud_sdk_sanitization::sanitize_bytes(storage);
    }
}
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
from the first decoded byte, including escaped strings and parser/model error
paths. Provider and action error messages use the same protected closure-access
model. A shared 65,536-node budget bounds aggregate JSON structure allocation.
Secret-bearing response models do not implement `Clone`. Ordinary Cloud
resource and pricing trees also redact all values from `Debug` and deliberately
omit infallible `Clone`; use their `try_clone()` methods when an owned copy is
required. Names, labels, addresses, topology, and unknown future fields remain
available only through explicit accessors and should be treated as operationally
sensitive by callers. `ResponseBuffer` clears the complete original transport
storage before decoding returns.

### Incremental JSON

Large provider responses can be validated without constructing the private
full JSON tree. `IncrementalJsonDecoder` accepts arbitrary chunks, emits
borrowed events, rejects duplicate decoded keys, and enforces explicit input,
depth, token, field, string, number, and exponent limits. Only `finish()` can
return `Complete`; visitor-requested `Stopped` is not complete-document
validation. See the
[incremental decoding guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/INCREMENTAL_DECODING.md)
for the full contract and the compile-checked
[`incremental_json` example](https://github.com/valkyoth/cloud-sdk/blob/main/crates/cloud-sdk-hetzner/examples/incremental_json.rs).

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
use cloud_sdk::pagination::{
    NumberedPagination, PaginationBudget, PaginationLimits, SnapshotPolicy,
};
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
let Ok(limits) = PaginationLimits::new(2, 50, 128) else { return };
let Ok(first) = cloud_sdk::pagination::PageNumber::new(1) else { return };
let budget = PaginationBudget::new(limits, SnapshotPolicy::Forbidden);
let Ok(mut pagination) = NumberedPagination::new(
    first,
    u64::from(metadata.per_page().get()),
    budget,
) else {
    return;
};
let Ok(boundary) = pagination.observe(metadata.as_core(), 1, None, None) else {
    return;
};

assert!(boundary.is_terminal());
assert_eq!(metadata.total_entries(), Some(1));
# }
# #[cfg(not(feature = "serde"))]
# fn main() {}
```

Pass the legacy `rate_limit()` compatibility view and optional snapshot values
exposed by the checked decode boundary as the third and fourth `observe`
arguments. Retain `quota()` separately for multi-bucket and `Retry-After`
policy. The caller remains
responsible for decoding the resource array and reporting its exact entry
count. See the provider-neutral
[pagination guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/PAGINATION_STRATEGIES.md)
for cursor, offset, marker, and provider-link strategies.

## Quota And Retry Example

Provider quota decoding is independent of transport and performs no retry or
sleep:

```rust
use cloud_sdk::rate_limit::WallClockTimestamp;
use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};
use cloud_sdk_hetzner::rate_limit::HetznerQuota;

let mut storage = [0_u8; 8_192];
let mut headers = ResponseHeaders::new(&mut storage);
headers.try_push("ratelimit-limit", b"3600", HeaderSensitivity::Public)?;
headers.try_push("ratelimit-remaining", b"0", HeaderSensitivity::Public)?;
headers.try_push("ratelimit-reset", b"42", HeaderSensitivity::Public)?;
headers.try_push("retry-after", b"10", HeaderSensitivity::Public)?;

let quota = HetznerQuota::decode(&headers, WallClockTimestamp::new(1))?;
assert_eq!(quota.buckets().len(), 1);
let policy_buckets = quota.to_quota_buckets()?;
assert_eq!(policy_buckets.len(), 1);
assert!(quota.retry_after().is_some());
# Ok::<(), Box<dyn core::error::Error>>(())
```

Choose stale-time, conflict, and maximum-delay policy with the provider-neutral
types documented in the
[quota guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/QUOTA_AND_RETRY.md).
The caller still owns operation eligibility, attempt limits, clocks, deadlines,
sleeping, and cancellation.

## Action Polling Example

```rust
# #[cfg(feature = "serde")]
# fn main() {
use cloud_sdk::action_polling::{
    ActionPollLimits, ActionPollStep, ActionPoller, ExponentialBackoff,
    PollControl, PollRequestStep, ProgressObservation, ProgressPolicy,
    ProviderTimeObservation,
};
use cloud_sdk::retry::{MonotonicDuration, MonotonicInstant};
use cloud_sdk_hetzner::serde::ActionEnvelope;

let body = br#"{"action":{
    "id":42,"command":"create_server","status":"running","progress":25,
    "started":"2026-07-13T12:00:00Z","finished":null,
    "resources":[],"error":null
}}"#;
let Ok(envelope) = serde_json::from_slice::<ActionEnvelope<'_>>(body) else {
    return;
};
let Ok(limits) = ActionPollLimits::new(
    60,
    MonotonicDuration::new(8_000),
    MonotonicDuration::new(120_000),
    MonotonicDuration::new(300_000),
) else { return };
let mut poller = ActionPoller::new(
    limits,
    ProgressPolicy::Nondecreasing,
    MonotonicInstant::new(0),
);
let Ok(mut backoff) = ExponentialBackoff::new(
    MonotonicDuration::new(2_000),
    MonotonicDuration::new(8_000),
    2,
) else { return };
assert_eq!(
    poller.next_request(PollControl::Continue, MonotonicInstant::new(0)),
    Ok(PollRequestStep::Request),
);
let step = poller.observe(
    envelope.action().polling_update(),
    ProgressObservation::Percent(envelope.action().progress()),
    None,
    ProviderTimeObservation::default(),
    MonotonicInstant::new(10),
    &mut backoff,
);

assert_eq!(
    step,
    Ok(ActionPollStep::Delay(MonotonicDuration::new(2_000))),
);
# }
# #[cfg(not(feature = "serde"))]
# fn main() {}
```

For an `error` action, the step is `ActionPollStep::Failed` and carries the
validated optional Hetzner error response. Hard observations, delay, and
monotonic elapsed budgets are selected before the first request. The SDK never
reads a clock, sleeps, retries, or owns an executor.

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

### Response Header Policy

Every prepared Hetzner operation admits `content-type`, protected
`x-request-id`, and the complete `ratelimit-limit`, `ratelimit-remaining`, and
`ratelimit-reset` set. A protected request-ID policy without matching raw
header admission fails during preparation. Quota decoding remains
provider-owned; incomplete sets remain visible for strict rejection.

The source-locked Hetzner API contracts consumed by this crate do not require
repeated response fields. The transport therefore rejects every duplicate
response-header name, including repeated `Set-Cookie`, rather than combining
values with field-specific rules. An upstream change that introduces a
multi-value response header fails closed with `InvalidResponseHeaders` and must
be reviewed before the provider contract is updated.

### TSIG Policy

The hardened API supports only HMAC-SHA256. HMAC-MD5 is prohibited and
HMAC-SHA1 is intentionally excluded. TSIG secrets must use canonical padded
Base64 and decode to at least 32 bytes. Generate them with a CSPRNG, share them
only with the intended peer, and rotate them periodically; representation
validation cannot establish entropy.

`ZoneFile`, `TsigKey`, `TsigCredentials`, returned `PrimaryNameserver`/`Zone`
models, and checked-success structures containing them intentionally omit
ordinary equality. Use a reviewed constant-time mechanism if external secret
comparison is required. RFC 8945 defines the
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
