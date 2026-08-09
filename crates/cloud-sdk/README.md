<p align="center">
  <b>no_std-first provider-neutral cloud SDK foundation for Rust.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">Crates.io</a>
  |
  <a href="https://docs.rs/cloud-sdk">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/PLATFORM_SUPPORT.md">Platforms</a>
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

# cloud-sdk

`cloud-sdk` is a `no_std`-first Rust foundation for building secure, portable
SDKs for cloud services. Provider APIs use shared, provider-neutral contracts
while retaining ownership of their request models, response models, endpoint
rules, and errors.

The workspace keeps networking, TLS, async runtimes, serialization,
filesystem access, clocks, and secret storage outside the default dependency
graph. These capabilities are explicit optional boundaries so applications
can select the transport, runtime, trust policy, and platform integration that
fit their environment. The project emphasizes validated inputs, bounded
memory use, caller-controlled behavior, cross-platform compatibility,
security review, and reproducible release evidence.

## Development Status

All `0.x` versions are development releases. Their APIs may change and they
may still contain defects or incomplete behavior before `1.0.0`; review the
documented operation and provider limits before using them with real
infrastructure.

Every pre-1.0 version receives the complete automated release gate, an
incremental pentest against the preceding tag, GitHub CI and CodeQL, release
notes, permanent pentest evidence, and a normal signed `v0.x.x` tag. Crates.io
publication occurs every fifth minor version, so the next public checkpoint is
`v0.70.0`. Intervening tags are fully reviewed but are not separately published.
A material security or compatibility need can trigger an earlier publication.

## Cost And Production Warning

Cloud APIs can create, modify, and delete billable resources. This SDK is built
with careful review, tests, security gates, and release checks, but no SDK can
guarantee that it is free from mistakes or that every provider-side API behavior
is risk-free.

Before running code against a real cloud account, review the exact operations,
inputs, permissions, and provider pricing yourself. You are responsible for the
infrastructure actions you execute and for any costs, downtime, data loss, or
configuration changes caused by those actions. If you find an SDK mistake,
please report it so it can be fixed.

## Current Status

Completed milestones and upcoming work are tracked in the
[release roadmap](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_PLAN.md).
Published and planned versions for each independently versioned crate are
listed in the
[crate version matrix](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATE_VERSION_MATRIX.md).

Current releases provide provider-neutral contracts and provider-owned,
validated request and response building blocks. Before the high-level client is
stabilized, the roadmap now hardens extensible provider identities, HTTP
metadata, authentication, pagination, local async, bounded decoding, and typed
operation contracts, then validates them with an unpublished OVHcloud API v2
architecture probe, a narrow credential-free Robot wire fixture, and
full-fidelity Hetzner vertical slices before the neutral API freeze.

The latest published checkpoint is `v0.65.0`, and `v0.68.0` is the latest
signed internal milestone. v0.69 adds service-typed official and explicitly
trusted custom Hetzner client construction, shared read-only execution through
the checked decoder, and named complete-workspace capacity profiles. The
v0.69 implementation stop is ready for incremental pentest; cumulative
crates.io publication remains deferred to v0.70.0.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.92.0` |
| Pinned toolchain | Rust `1.97.1` |
| Default target | `no_std` |
| Default runtime dependencies | only the first-party cleanup boundary and admitted `sanitization` primitive; provider crates remain transport-free |
| Unsafe policy | first-party crates use `#![forbid(unsafe_code)]` |
| Default features | empty |
| Network defaults | none |
| Secret storage defaults | none |
| Release evidence | full gates, SBOM, and incremental pentest for every tag; crates.io publication every fifth pre-1.0 minor or earlier when required |
| Platform support | explicit tiers and targets in [`docs/PLATFORM_SUPPORT.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/PLATFORM_SUPPORT.md) |
| Crate versions | tracked in [`docs/CRATE_VERSION_MATRIX.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATE_VERSION_MATRIX.md) |
| 1.0 target | serious production-ready foundation plus complete Hetzner Cloud, DNS, Console Storage Box, and Robot provider |

## Provider Roadmap

| Provider or role | Target | Crate or status |
| --- | --- | --- |
| [`Hetzner Cloud`](https://www.hetzner.com/) | `1.0.0` | [`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner) |
| [`Hetzner Robot`](https://robot.hetzner.com/doc/webservice/en.html) | `1.0.0` | pre-1.0 milestones in `cloud-sdk-hetzner` |
| [`OVHcloud API v2`](https://docs.ovhcloud.com/en/guides/manage-and-operate/api/apiv2/) architecture probe | `0.57.0 - 0.61.0` | unpublished conformance fixture; neutral freeze follows in `0.62.0` |
| [`Scaleway`](https://www.scaleway.com/en/developers/api/) | `1.1.0 - 1.6.0` | planned `cloud-sdk-scaleway`; stable GA APIs first |
| [`DigitalOcean`](https://docs.digitalocean.com/reference/api/reference/public-apis/) | `1.7.0 - 1.12.0` | planned `cloud-sdk-digitalocean` |
| [`OVHcloud`](https://docs.ovhcloud.com/en/) full provider | later post-1.0 | planned `cloud-sdk-ovhcloud` after a dedicated v1/v2 and product-scope plan |

The probe exists to test the shared architecture against a materially different
API. Published providers receive their own source lock, threat model, API
matrix, release plan, tests, and pentest gates.

## Rust Version Support

The minimum supported Rust version is Rust `1.92.0`. Development uses the
pinned stable Rust `1.97.1` until the toolchain policy is updated.

v0.52.0 raised the MSRV from Rust 1.90 to 1.92 because Rust 1.90 and 1.91 hit
[compiler issue #100013](https://github.com/rust-lang/rust/issues/100013) when
proving the client kernel's explicit `Send` future. The project retains the
cross-thread async guarantee instead of weakening it for those two compilers.

| Rust | Local Evidence |
| --- | --- |
| `1.92.0 - 1.96.1` | `cargo +<version> check --workspace --all-features` for every supported compiler |
| `1.97.0` | `cargo +1.97.0 check --workspace --all-features` |
| `1.97.1` | `scripts/checks.sh` |

Portable and native platform evidence is documented in
[`docs/PLATFORM_SUPPORT.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/PLATFORM_SUPPORT.md).

## Install

```toml
[dependencies]
cloud-sdk = "0.65.0"
cloud-sdk-hetzner = "0.40.0"
```

## cloud-sdk Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps the crate allocation-free and `no_std`. |
| `alloc` | no | Enables APIs that require the Rust `alloc` crate. |
| `std` | no | Enables `alloc` and standard-library integration. |

Docs.rs builds this crate with all features so every public optional API is
visible. Applications should enable only the features they use.

## Guides

- [Provider-neutral quickstart](https://github.com/valkyoth/cloud-sdk/blob/main/docs/QUICKSTART.md)
- [Hetzner workflow examples](https://github.com/valkyoth/cloud-sdk/blob/main/docs/HETZNER_EXAMPLES.md)
- [Hetzner live smoke testing](https://github.com/valkyoth/cloud-sdk/blob/main/docs/LIVE_SMOKE_TESTING.md)
- [Security recipes](https://github.com/valkyoth/cloud-sdk/blob/main/docs/SECURITY_RECIPES.md)
- [Raw bounded HTTP executor](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RAW_HTTP_EXECUTOR.md)
- [Bearer and Basic authentication policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/AUTHENTICATION_POLICY.md)
- [Canonical signing input policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/SIGNING_INPUT_POLICY.md)
- [Robot wire source lock](https://github.com/valkyoth/cloud-sdk/blob/main/docs/ROBOT_WIRE_SOURCE_LOCK.md)
- [Pagination strategies](https://github.com/valkyoth/cloud-sdk/blob/main/docs/PAGINATION_STRATEGIES.md)
- [Schema version validation](https://github.com/valkyoth/cloud-sdk/blob/main/docs/SCHEMA_VERSION_VALIDATION.md)
- [Bounded asynchronous resources](https://github.com/valkyoth/cloud-sdk/blob/main/docs/ASYNC_RESOURCES.md)
- [Quota and retry policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/QUOTA_AND_RETRY.md)
- [Retry and idempotency policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RETRY_AND_IDEMPOTENCY.md)
- [Plan-confirm execution permits](https://github.com/valkyoth/cloud-sdk/blob/main/docs/EXECUTION_PERMITS.md)
- [Provider-generic client kernel](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CLIENT_KERNEL.md)
- [Hetzner client foundation](https://github.com/valkyoth/cloud-sdk/blob/main/docs/HETZNER_CLIENT.md)
- [Pager and action workflow drivers](https://github.com/valkyoth/cloud-sdk/blob/main/docs/WORKFLOW_DRIVERS.md)
- [Payload-free diagnostics](https://github.com/valkyoth/cloud-sdk/blob/main/docs/DIAGNOSTICS.md)
- [Dynamic testkit scenarios](https://github.com/valkyoth/cloud-sdk/blob/main/docs/DYNAMIC_TESTKIT.md)
- [Local async contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/LOCAL_ASYNC.md)
- [Streaming transport contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/STREAMING.md)
- [Release runbook](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_RUNBOOK.md)
- [Versioning and error policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/VERSIONING_POLICY.md)
- [Provider-generic drift evidence](https://github.com/valkyoth/cloud-sdk/blob/main/docs/PROVIDER_DRIFT.md)
- [Migrating to v0.29](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.29.0.md)
- [Migrating to v0.30](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.30.0.md)
- [Migrating to v0.31](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.31.0.md)
- [Migrating to v0.32](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.32.0.md)
- [Migrating to v0.33](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.33.0.md)
- [Migrating to v0.34](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.34.0.md)
- [Migrating to v0.35](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.35.0.md)
- [Migrating to v0.36](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.36.0.md)
- [Migrating to v0.37](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.37.0.md)
- [Migrating to v0.38](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.38.0.md)
- [Migrating to v0.39](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.39.0.md)
- [Migrating to v0.40](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.40.0.md)
- [Migrating to v0.41](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.41.0.md)
- [Migrating to v0.42](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.42.0.md)
- [Migrating to v0.43](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.43.0.md)
- [Migrating to v0.44](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.44.0.md)
- [Migrating to v0.45](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.45.0.md)
- [Migrating to v0.46](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.46.0.md)
- [Migrating to v0.47](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.47.0.md)
- [Migrating to v0.48](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.48.0.md)
- [Migrating to v0.49](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.49.0.md)
- [Migrating to v0.50](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.50.0.md)
- [Migrating source users to v0.51](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.51.0.md)
- [Migrating source users to v0.52](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.52.0.md)
- [Migrating source users to v0.53](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.53.0.md)
- [Migrating source users to v0.54](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.54.0.md)
- [Migrating to v0.55](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.55.0.md)
- [Migrating source users to v0.56](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.56.0.md)
- [Migrating source users to v0.57](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.57.0.md)
- [Migrating source users to v0.58](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.58.0.md)
- [Migrating source users to v0.59](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.59.0.md)
- [Migrating to v0.60](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.60.0.md)
- [Migrating source users to v0.61](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.61.0.md)
- [Migrating source users to v0.62](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.62.0.md)
- [Migrating source users to v0.63](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.63.0.md)
- [Migrating source users to v0.64](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.64.0.md)
- [Migrating to v0.65](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.65.0.md)
- [Migrating source users to v0.66](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.66.0.md)
- [Migrating source users to v0.67](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.67.0.md)
- [Migrating source users to v0.68](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.68.0.md)
- [Migrating source users to v0.69](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.69.0.md)
- [Compile-time Hetzner operation associations](https://github.com/valkyoth/cloud-sdk/blob/main/docs/OPERATION_ASSOCIATIONS.md)
- [Incremental provider decoding](https://github.com/valkyoth/cloud-sdk/blob/main/docs/INCREMENTAL_DECODING.md)
- [Deprecated endpoint policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/DEPRECATED_ENDPOINT_POLICY.md)

## Provider-Neutral Quickstart

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{RequestTarget, TransportRequest};

let Ok(target) = RequestTarget::new("/resources?page=1") else {
    return;
};
let request = TransportRequest::new(Method::Get, target);

assert_eq!(request.method(), Method::Get);
assert_eq!(request.target().as_str(), "/resources?page=1");
assert!(request.body().is_empty());
```

Bound concurrent typed execution with caller-owned storage:

```rust
use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};

let pool = ClientWorkspacePool::<4>::new()?;
let mut target = [0_u8; 1024];
let mut request_body = [0_u8; 4096];
let mut response_body = [0_u8; 8192];
let mut response_headers = [0_u8; 8192];
let workspace = ClientWorkspace::new(
    &mut target,
    &mut request_body,
    &mut response_body,
    &mut response_headers,
);
let lease = pool.try_acquire(workspace)?;
assert_eq!(lease.capacities(), (1024, 4096, 8192, 8192));
drop(lease);
assert!(target.iter().all(|byte| *byte == 0));
# Ok::<(), Box<dyn core::error::Error>>(())
```

Provider crates implement the typed preparation and checked decoder. The
kernel owns no allocation, queue, executor, clock, retry loop, or network
stack; see the [client kernel contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CLIENT_KERNEL.md).

Add provider-owned headers as one validated bounded block:

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{
    ContentType, MediaType, RequestHeader, RequestHeaders, RequestTarget,
    TransportRequest,
};

let target = RequestTarget::new("/resources")?;
let entries = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::JSON),
];
let headers = RequestHeaders::new(&entries)?;
let request = TransportRequest::new(Method::Post, target)
    .with_headers(headers)
    .with_body(br#"{"name":"example"}"#);

assert_eq!(request.headers().encoded_len(), 58);
# Ok::<(), Box<dyn core::error::Error>>(())
```

Request header names, values, count, and aggregate encoded bytes are bounded.
Duplicate names and caller-owned authority, framing, proxy, hop-by-hop, and
`Authorization` headers fail closed. Header values are always redacted from
`Debug`; sensitivity markers additionally instruct adapters to protect their
temporary values.

Build separate components when query presence or dialect is security-relevant:

```rust
use cloud_sdk::transport::{
    CanonicalQuery, RequestPath, RequestQuery, RequestTarget,
};

let path = RequestPath::new("/resources")?;
let query = CanonicalQuery::new("name=test%20service&page=1")?;
let mut storage = [0_u8; 128];
let target = RequestTarget::assemble(
    path,
    RequestQuery::Canonical(query),
    &mut storage,
)?;

assert_eq!(target.path(), path);
assert_eq!(target.query_bytes(), Some(query.as_str().as_bytes()));
# Ok::<(), Box<dyn core::error::Error>>(())
```

`RequestQuery::Absent` differs from a present empty query. Canonical spaces use
`%20`; form-style `+` is admitted only through the separate `FormQuery` type.
Pair iteration preserves exact order, duplicate keys, and `key` versus `key=`.
Assembly initializes only `output[..target.len()]`; never consume the untouched
scratch-buffer tail, which may contain bytes from an earlier use.

Known methods include GET, POST, PUT, DELETE, PATCH, HEAD, and origin-form
OPTIONS. Provider crates can define a finite extension without changing core:

```rust
use cloud_sdk::Method;

const PURGE: Method = match Method::extension("PURGE") {
    Ok(method) => method,
    Err(_) => panic!("invalid provider method"),
};

assert_eq!(PURGE.as_str(), "PURGE");
```

Extension tokens are static, allocation-free, bounded to 32 bytes, uppercase,
and cannot alias known methods. CONNECT, TRACE, `OPTIONS *`, protocol upgrade,
and tunnelling are outside the current transport contract. Retry, destructive,
and cost behavior always comes from explicit operation metadata, never the
method.

### Canonical Signing Inputs

Provider signing policy can bind exact request bytes without making core own a
clock, nonce generator, key, or algorithm:

```rust
use cloud_sdk::authentication::{
    CanonicalSigningInput, RequestBodyHasher, SigningAlgorithm, SigningContext,
    SigningDigestAlgorithm, SigningFreshness, SigningHeaders, SigningKeyId,
    SigningNonce, UnixTime,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointScheme, RequestHeaders, RequestTarget,
    TransportRequest,
};
use cloud_sdk::{Method, ProviderId, ServiceId};

fn prepare<H: RequestBodyHasher>(hasher: &H) {
    let Ok(target) = RequestTarget::new("/resources?page=1") else { return };
    let entries = [];
    let Ok(headers) = RequestHeaders::new(&entries) else { return };
    let request =
        TransportRequest::new(Method::Get, target).with_headers(headers);
    let Ok(endpoint) = EndpointIdentity::new(
        EndpointScheme::Https,
        "api.example.test",
        443,
        "/v1",
    ) else { return };
    let Ok(provider) = ProviderId::new("example") else { return };
    let Ok(service) = ServiceId::new("compute") else { return };
    let Ok(key_id) = SigningKeyId::new("production-key-1") else { return };
    let Ok(digest_algorithm) = SigningDigestAlgorithm::new("sha256") else {
        return;
    };
    let Ok(algorithm) = SigningAlgorithm::new("provider-algorithm") else {
        return;
    };
    let context = SigningContext::new(
        provider,
        service,
        endpoint,
        key_id,
        digest_algorithm,
        algorithm,
    );
    let Ok(nonce) = SigningNonce::new(b"caller-generated-nonce") else {
        return;
    };
    let freshness =
        SigningFreshness::new(nonce, UnixTime::from_seconds(1_700_000_000));
    let Ok(selected) = SigningHeaders::new(&entries) else { return };
    let mut digest = [0_u8; 128];
    let mut storage = [0_u8; 1024];
    let Ok(canonical) = CanonicalSigningInput::new_hashed(
        request,
        context,
        selected,
        freshness,
        hasher,
        &mut digest,
        &mut storage,
    ) else { return };

    assert!(canonical.as_bytes().starts_with(b"cloud-sdk-signing-v2\0"));
}
```

The constructor hashes the retained request body and clears digest scratch.
Signing returns a bounded `SignedRequest` that retains the same request and
clears signature storage on drop. Provider code must still select reviewed
digest, signing, replay, nonce, timestamp, key, and verification policy.

### Provider-Owned Identity

Provider crates define their own bounded identities without changing a central
`cloud-sdk` enum:

```rust
use cloud_sdk::{
    ProviderId, ProviderMarker, ServiceId, ServiceMarker, provider_id,
    service_id,
};

enum ExampleProvider {}

impl ProviderMarker for ExampleProvider {
    const ID: ProviderId = provider_id!("example");
}

enum ComputeService {}

impl ServiceMarker for ComputeService {
    type Provider = ExampleProvider;
    const ID: ServiceId = service_id!("compute");
}

assert_eq!(ExampleProvider::ID.as_str(), "example");
assert_eq!(ComputeService::ID.as_str(), "compute");
```

IDs are allocation-free, at most 63 bytes, and accept lowercase ASCII letters,
digits, and single internal hyphens. A service marker always names its owning
provider.

### Endpoint Trust Policy

Provider services bind credentials to an explicit endpoint policy:

```rust
use cloud_sdk::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme,
};

let endpoint = EndpointIdentity::new(
    EndpointScheme::Https,
    "api.example.invalid",
    443,
    "/v1",
)?;
let policy = EndpointPolicy::fixed(endpoint);
assert_eq!(policy.verify(endpoint), Ok(()));
# Ok::<(), Box<dyn core::error::Error>>(())
```

Policies represent one fixed endpoint, a bounded finite official set, a
provider-derived regional endpoint, or an explicitly acknowledged custom
credential destination. DNS names must already be canonical lowercase ASCII;
internationalized names use lowercase A-label form. IPv6 is bracketed and
compared by address bits. Userinfo, zone identifiers, trailing DNS dots,
percent-encoded hosts, and Unicode host input are rejected. DNS resolution,
resolved-address filtering, and network egress controls remain optional
transport or deployment policy and are not performed by `cloud-sdk`.

The core contracts perform no I/O and select no executor. Use
`cloud-sdk-testkit` for deterministic blocking or async tests, or opt into
`cloud-sdk-reqwest/blocking-rustls`, `blocking-rustls-webpki-roots`,
`blocking-rustls-fips`, or `async-rustls` for HTTPS.

### Prepared Request Policy

Provider operations can bind request execution to explicit impact, retry,
cost, endpoint, and response rules without selecting a transport:

```rust
use cloud_sdk::operation::{
    CostIntent, OperationImpact, OperationMetadata, RequestIdPolicy,
    RequestSemantics, RetryEligibility,
};

# fn main() -> Result<(), cloud_sdk::operation::OperationMetadataError> {
let metadata = OperationMetadata::new(
    OperationImpact::Mutation,
    RequestSemantics::Idempotent,
    RetryEligibility::ExplicitPolicy,
    CostIntent::MayIncurCost,
    RequestIdPolicy::Protected,
)?;

assert_eq!(metadata.impact(), OperationImpact::Mutation);
assert_eq!(
    metadata.retry_eligibility(),
    RetryEligibility::ExplicitPolicy,
);
# Ok(())
# }
```

`PrepareOperation` writes a validated target and body into caller-owned
storage and returns one `PreparedRequest`. Read-only blocking and async
execution verify
the provider-owned endpoint policy before sending, lend only the response policy's admitted
capacity through a sealed `ResponseWriter`, and return a
`CheckedResponseGuard` only after status, body, and content type pass. The
provided transports wrap each writer use in a transactional `ResponseAttempt`;
failed, timed-out, unwound, or cancelled attempts clear complete body and
header storage before reuse. Blocking custom transports acquire the same guard
through `ResponseWriter::begin_attempt`; all async implementations receive
non-committing staging instead. The checked response guard owns
mandatory volatile cleanup of the complete caller buffer plus its
header, request-ID, cursor/link, and decoder-scratch workspace; borrowed
decoding is closure-scoped and owned decoding clears storage before returning.
Optional `ResponseStorageSanitizer` implementations may add platform cleanup,
but cannot replace or weaken the core clear. The SDK still performs no
automatic retry or scheduling.

State-changing prepared requests cannot execute directly. Mutation,
destructive, and cost-bearing operations require a non-copyable
plan-confirm permit bound to the exact request, endpoint, account/tenant,
expiry, replay policy, and any observed price ceiling. See
[Plan-Confirm Execution Permits](https://github.com/valkyoth/cloud-sdk/blob/main/docs/EXECUTION_PERMITS.md).
Direct read-only execution additionally requires `GET` or `HEAD`; contradictory
read-only metadata is rejected during preparation and cannot bypass the method
check through type erasure.

Use `PreparationStorageGuard` when request buffers may contain secrets. The
prepared request borrows the guard, so safe Rust keeps cleanup ownership alive
through transport use. Every preparation attempt and the guard's drop
volatile-clear both complete buffers, including unused tails. Named
`EMBEDDED`, `DEFAULT`, and `LARGE` capacity profiles make storage policy
explicit. Enabling `alloc` adds fallible `OwnedPreparationStorage` convenience
without changing the default allocation-free graph.

## Optional Blocking Transport

```toml
[dependencies]
cloud-sdk = "0.65.0"
cloud-sdk-reqwest = { version = "0.34.0", features = ["blocking-rustls"] }
```

The production builder is HTTPS-only, requires explicit bounded timeouts and a
user agent, uses rustls with TLS 1.2 minimum, and disables redirects, retries,
proxies, referer generation, and response decompression. It forces HTTP/1 and
the system resolver even if another dependency enables reqwest HTTP/2 or
Hickory DNS. The caller owns credential generation, scope, rotation,
revocation, and cleanup of immutable secret sources. Mutable and guarded
constructors clear their complete source buffers. Type-separated bearer and
Basic clients support caller-bounded concurrency. Bearer clients additionally
support atomic rotation without holding credential locks across I/O. Every
authenticated send requires complete provider or operation-owned scope; see
the [authentication policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/AUTHENTICATION_POLICY.md).

See the complete, compile-checked
[`cloud-sdk-reqwest` blocking example](https://docs.rs/cloud-sdk-reqwest/latest/cloud_sdk_reqwest/#blocking-example)
for client construction and request execution.

### Optional Deterministic Root Snapshot

Use a source-pinned Mozilla root snapshot instead of host trust-store contents
when deterministic public WebPKI roots are required:

```toml
[dependencies]
cloud-sdk = "0.65.0"
cloud-sdk-reqwest = { version = "0.34.0", features = ["blocking-rustls-webpki-roots"] }
```

The blocking API is unchanged. This feature excludes host-added enterprise
roots from trust decisions and updates roots only when `webpki-roots` is
reviewed and upgraded. It does not provide certificate revocation checking,
certificate pinning, private PKI support, or FIPS status. When combined with
`blocking-rustls-fips`, the explicit FIPS roots-and-CRLs policy wins.

### Optional Blocking FIPS Transport

Applications that require the reviewed FIPS path must select the dedicated
feature instead of relying on dependency feature unification:

```toml
[dependencies]
cloud-sdk = "0.65.0"
cloud-sdk-reqwest = { version = "0.34.0", features = ["blocking-rustls-fips"] }
```

Client construction explicitly selects rustls' AWS-LC FIPS provider and fails
unless both `CryptoProvider::fips()` and `ClientConfig::fips()` report true. It
also requires a `FipsTlsPolicy` with deployment-managed trust roots and
complete, current CRLs; unknown or expired revocation status fails closed. The
feature alone does not make an application or deployment FIPS compliant: the
caller must satisfy the AWS-LC security policy, approved operating-environment,
build, entropy, deployment, and operational requirements. The full policy
example is in the
[reqwest crate README](https://crates.io/crates/cloud-sdk-reqwest). See also the
[FIPS dependency admission](https://github.com/valkyoth/cloud-sdk/blob/main/docs/dependency-admission-reqwest-fips.md).

## Optional Async Transport

```toml
[dependencies]
cloud-sdk = "0.65.0"
cloud-sdk-reqwest = { version = "0.34.0", features = ["async-rustls"] }
```

The async adapter requires an active Tokio executor because reqwest uses Tokio
internally; the core trait and testkit remain executor-neutral. Responses are
buffered only up to caller capacity and copied after complete success. Timeout,
read failure, overflow, or cancellation leaves the caller buffer cleared.
The shared-reference contract does not spawn tasks or select a concurrency
limit; callers own task lifetimes, bounds, cancellation, and executor policy.
See the complete, compile-checked
[`cloud-sdk-reqwest` async example](https://docs.rs/cloud-sdk-reqwest/latest/cloud_sdk_reqwest/#async-example)
for client construction and request execution.

## Async Transport

`LocalAsyncTransport`, `LocalAsyncAuthenticatedTransport`, and
`LocalAsyncRawHttpExecutor` support `!Send` browser-WASM, embedded, and
single-threaded futures without selecting an executor. Existing `Send` async
implementations automatically satisfy the local contracts. Prepared requests,
provider links, and retry permits expose `execute_local_async`.

All async implementations receive non-committing `AsyncResponseStaging` and
return `ResponseCompletion`. Cross-thread callers use `drive_async`,
`drive_async_authenticated`, or `drive_async_raw`; local callers use the
corresponding `drive_local` functions. These SDK-owned drivers commit only
after `Ready(Ok)`.

Dropping any async driver future rolls back partial response state, but request
delivery is conservatively `PossiblySent`. See the
[local async contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/LOCAL_ASYNC.md).

## Streaming Transport

Opt-in streaming contracts support finite uploads, finite downloads, and
caller-cancelled event streams without adding allocation, networking, or an
executor to core. Every attempt fixes hard byte, chunk-size, chunk-count,
observation, and consecutive zero-progress limits plus declared or
executor-owned framing and transactional or direct sink behavior.

```rust
use cloud_sdk::transport::{
    StreamAttempt, StreamFraming, StreamKind, StreamLimits, StreamOutcome,
    StreamPolicy, StreamSinkMode,
};

let Ok(limits) = StreamLimits::new(8, 4, 2, 4, 1) else { return };
let Ok(policy) = StreamPolicy::new(
    StreamKind::FiniteDownload,
    StreamFraming::Declared(4),
    StreamSinkMode::Transactional,
    limits,
) else { return };
let mut outcome = StreamOutcome::new();
let mut attempt = StreamAttempt::new(policy, &mut outcome);
assert!(attempt.begin_source_observation().is_ok());
assert!(attempt.begin_chunk(4).is_ok());
assert!(attempt.begin_sink_observation().is_ok());
assert!(attempt.advance(4).is_ok());
assert!(attempt.begin_source_observation().is_ok());
assert!(attempt.finish().is_ok());
assert!(attempt.commit_sink().is_ok());
```

Blocking, Send-async, and local-async drivers use complete caller-owned scratch
storage, clear it on every exit, count only bytes actually accepted by the
sink, and never retry. Sink-attempt state becomes conservative before external
I/O, and async drivers force a cooperative yield after bounded ready work. See
the
[streaming contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/STREAMING.md).

## Numbered Pagination Example

```rust
use cloud_sdk::pagination::{
    NumberedPageMetadata, NumberedPageObservation, NumberedPagination,
    PageNumber, PagerControl, PagerDriver, PagerStep, PaginationBudget,
    PaginationLimits, SnapshotPolicy,
};

# fn main() -> Result<(), Box<dyn core::error::Error>> {
let first = PageNumber::new(1)?;
let second = PageNumber::new(2)?;
let limits = PaginationLimits::new(3, 30, 128)?;
let budget = PaginationBudget::new(limits, SnapshotPolicy::Forbidden);
let strategy = NumberedPagination::new(first, 25, budget)?;
let mut pager = PagerDriver::new(strategy);

assert_eq!(
    pager.next_request(PagerControl::Continue)?,
    PagerStep::Request(first),
);
let metadata = NumberedPageMetadata::new(
    first,
    25,
    None,
    Some(second),
    Some(second),
    Some(30),
)?;
let boundary = pager.observe(NumberedPageObservation::new(
    metadata, 25, None, None,
))?;

assert!(!boundary.is_terminal());
assert_eq!(
    pager.next_request(PagerControl::Continue)?,
    PagerStep::Request(second),
);
# Ok(())
# }
```

The driver admits exactly one request before accepting its response. The caller
fetches and decodes that page, then supplies one grouped observation. Request,
item, opaque-state, snapshot, and traversal metadata limits fail closed;
cancellation is explicit and the driver has no transport or executor. Cursor,
offset, marker, and operation-bound provider-link strategies are documented in
the [pagination guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/PAGINATION_STRATEGIES.md).

## Quota Decision Example

```rust
use cloud_sdk::rate_limit::{
    DelayConflictPolicy, DelaySeconds, ExcessDelayPolicy, PastTimestampPolicy,
    QuotaBucket, QuotaBucketId, QuotaBuckets, QuotaDelayPolicy, QuotaReset,
    WallClockTimestamp, decide_delay,
};

# fn main() -> Result<(), Box<dyn core::error::Error>> {
let mut buckets = QuotaBuckets::new();
let bucket = QuotaBucket::new(
    QuotaBucketId::new(b"provider-hourly")?,
    100,
    0,
    QuotaReset::After(DelaySeconds::new(30)),
)?;
buckets.try_push(bucket)?;
let policy = QuotaDelayPolicy::new(
    DelaySeconds::new(300),
    PastTimestampPolicy::Reject,
    ExcessDelayPolicy::Reject,
    DelayConflictPolicy::RetryAfterPrecedence,
);
let decision = decide_delay(
    &buckets,
    None,
    WallClockTimestamp::new(1_000),
    None,
    policy,
)?;
assert_eq!(decision.map(|value| value.delay().get()), Some(30));
# Ok(())
# }
```

The result is data only. The caller owns clocks, sleeping, retry eligibility,
attempt counts, deadlines, and cancellation. See the
[quota and retry guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/QUOTA_AND_RETRY.md).

## Retry Budget Example

```rust
use cloud_sdk::retry::{MaxAttempts, MonotonicDuration, RetryPolicy};

# fn main() -> Result<(), Box<dyn core::error::Error>> {
let policy = RetryPolicy::new(
    MaxAttempts::new(3)?,
    MonotonicDuration::new(30_000),
    MonotonicDuration::new(120_000),
);
assert_eq!(policy.max_attempts().get(), 3);
assert_eq!(policy.max_cumulative_delay().get(), 30_000);
# Ok(())
# }
```

One non-cloneable controller owns the total attempts and requested-delay
budget. Private retry subjects bind complete prepared policy to exact wire
fingerprints. One-use permits exclusively borrow controller clock state,
recheck the hard deadline after caller-owned sleep, and execute their exact
request directly. Retries also require prepared-body replayability, provider
operation metadata, delivery phase, and a fresh fingerprint-bound intent for
mutations. The complete flow and provider policy table are in
the [retry and idempotency guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RETRY_AND_IDEMPOTENCY.md).

## Action Polling Example

```rust
use cloud_sdk::action_polling::{
    ActionPollLimits, ActionPollStep, ActionPoller, ActionUpdate,
    ExponentialBackoff, PollControl, PollRequestStep, ProgressObservation,
    ProgressPolicy, ProviderTimeObservation,
};
use cloud_sdk::retry::{MonotonicDuration, MonotonicInstant};

# fn main() -> Result<(), Box<dyn core::error::Error>> {
let limits = ActionPollLimits::new(
    60,
    MonotonicDuration::new(8_000),
    MonotonicDuration::new(120_000),
    MonotonicDuration::new(300_000),
)?;
let mut poller = ActionPoller::new(
    limits,
    ProgressPolicy::Nondecreasing,
    MonotonicInstant::new(0),
);
let mut backoff = ExponentialBackoff::new(
    MonotonicDuration::new(2_000),
    MonotonicDuration::new(8_000),
    2,
)?;

assert_eq!(
    poller.next_request(PollControl::Continue, MonotonicInstant::new(0))?,
    PollRequestStep::Request,
);
let running = poller.observe(
    ActionUpdate::<()>::Running,
    ProgressObservation::Percent(25),
    None,
    ProviderTimeObservation::default(),
    MonotonicInstant::new(10),
    &mut backoff,
)?;
assert_eq!(
    running,
    ActionPollStep::Delay(MonotonicDuration::new(2_000)),
);
# Ok(())
# }
```

Provider failures are returned as `ActionPollStep::Failed(E)`. The driver
enforces request/response sequencing, nonzero bounded backoff, an unconditional
observation limit, cumulative delay, and monotonic elapsed time. `PollControl`
owns cancellation separately from backoff. Provider wall-clock timestamps are
typed telemetry only and cannot extend local budgets. The SDK owns no clock,
executor, sleep, or transport.

## Fixed Buffer Example

```rust
use cloud_sdk::buffer::write_query_u64;

# fn main() -> Result<(), ()> {
let mut output = [0u8; 8];
let mut len = 0;
let mut first = true;
write_query_u64(&mut output, &mut len, &mut first, "page", 0, ())?;

let query = output
    .get(..len)
    .and_then(|bytes| core::str::from_utf8(bytes).ok());
assert_eq!(query, Some("page=0"));
# Ok(())
# }
```

## JSON String Example

```rust
use cloud_sdk::buffer::write_json_string;

# fn main() -> Result<(), ()> {
let mut output = [0u8; 48];
let mut len = 0;
write_json_string(&mut output, &mut len, "line\n\"quoted\"", ())?;

let value = output
    .get(..len)
    .and_then(|bytes| core::str::from_utf8(bytes).ok());
assert_eq!(value, Some("\"line\\n\\\"quoted\\\"\""));
# Ok(())
# }
```

## Workspace Crates

| Crate | Default `std`? | Purpose |
| --- | --- | --- |
| [`cloud-sdk`](https://crates.io/crates/cloud-sdk) | no | Provider-neutral domains, prepared operations, checked responses, and bounded streaming policy. |
| [`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner) | no | Hetzner provider APIs and provider-specific documentation. |
| [`cloud-sdk-reqwest`](https://crates.io/crates/cloud-sdk-reqwest) | no | Provider-neutral optional blocking and async reqwest/rustls transports; transport-free by default. |
| [`cloud-sdk-testkit`](https://crates.io/crates/cloud-sdk-testkit) | no | Provider-neutral mock transports, stream fixtures, prepared-request records, response fixtures, and adversarial corpus. |
| [`cloud-sdk-sanitization`](https://crates.io/crates/cloud-sdk-sanitization) | no | Provider-neutral volatile cleanup plus bounded fallible protected UTF-8 growth. |

Each provider has one primary crate for its APIs and documentation. Reusable
transport, testing, and secret-handling capabilities remain provider-neutral.

## Provider Documentation

Provider-specific API coverage and maintenance procedures live outside this
provider-neutral README. For Hetzner, see the
[`cloud-sdk-hetzner` crate](https://crates.io/crates/cloud-sdk-hetzner), the
[API matrix](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_MATRIX.md),
and the
[source-lock policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/SPEC_LOCK.md),
and the
[API drift maintenance runbook](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_DRIFT_MAINTENANCE.md).

## Development Checks

Run `scripts/checks.sh` for the maintained local check suite. The complete
pentest, CI, release-gate, tagging, and publication process is documented in
the [release runbook](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_RUNBOOK.md).
