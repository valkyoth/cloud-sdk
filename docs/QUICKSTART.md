# Provider-Neutral Quickstart

`cloud-sdk` separates provider request models from transport execution. The
default graph performs no I/O, selects no runtime, stores no token, and remains
usable in `no_std` environments.

## Install

```toml
[dependencies]
cloud-sdk = "0.50.0"
```

Provider-specific request models are separate dependencies. For Hetzner:

```toml
[dependencies]
cloud-sdk = "0.50.0"
cloud-sdk-hetzner = "0.38.0"
```

## Build A Transport Request

The provider-neutral request contract carries a validated origin-form target,
method, bounded ordered headers, and an optional body:

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{RequestTarget, TransportRequest};

let target = RequestTarget::new("/servers?page=1")?;
let request = TransportRequest::new(Method::Get, target);

assert_eq!(request.target().as_str(), "/servers?page=1");
# Ok::<(), cloud_sdk::transport::RequestTargetError>(())
```

When query presence or encoding dialect matters, assemble validated components
explicitly:

```rust
use cloud_sdk::transport::{
    CanonicalQuery, RequestPath, RequestQuery, RequestTarget,
};

let path = RequestPath::new("/servers")?;
let query = CanonicalQuery::new("name=test%20server&page=1")?;
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

Absent and present-empty queries are distinct. `CanonicalQuery` requires `%20`
for spaces; provider-specific form semantics use `FormQuery`. See
[`MIGRATION.md#v0350`](MIGRATION.md#v0350).
Only `output[..target.len()]` is initialized; never consume the untouched
scratch-buffer tail, which may contain bytes from an earlier use.

Headers are explicit provider policy rather than transport defaults:

```rust
use cloud_sdk::transport::{
    ContentType, MediaType, RequestHeader, RequestHeaders,
};

let entries = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::JSON),
];
let headers = RequestHeaders::new(&entries)?;
assert_eq!(headers.as_slice().len(), 2);
# Ok::<(), cloud_sdk::transport::HeaderError>(())
```

See [`MIGRATION.md#v0360`](MIGRATION.md#v0360) for reserved ownership,
duplicate handling, limits, response metadata, and adapter changes.
See [`MIGRATION.md#v0380`](MIGRATION.md#v0380) for mandatory response
cleanup, protected request identifiers, and complete checked decode workspace
ownership.
See [`MIGRATION.md#v0390`](MIGRATION.md#v0390) for transactional request
encoding, guarded preparation storage, and named capacity profiles.
See [`MIGRATION.md#v0400`](MIGRATION.md#v0400) for raw bounded HTTP
execution, delivery phases, and response-wire policy.
See [`MIGRATION.md#v0410`](MIGRATION.md#v0410) for mandatory bearer scope,
generation-safe rotation, and compare-and-swap refresh.
See [`MIGRATION.md#v0420`](MIGRATION.md#v0420) for Basic authentication and
canonical signing inputs.
See [`MIGRATION.md#v0430`](MIGRATION.md#v0430) for mandatory prepared
authentication and raw-response policy, authenticated execution, and the
complete Hetzner wire migration.
See [`MIGRATION.md#v0440`](MIGRATION.md#v0440) for distinct numbered,
offset, cursor, marker, and provider-link pagination strategies.
See [`MIGRATION.md#v0450`](MIGRATION.md#v0450) for provider-owned quota
decoding, exact `Retry-After`, and pure bounded delay decisions.
See [`MIGRATION.md#v0460`](MIGRATION.md#v0460) for canonical request
fingerprints, fresh intent binding, and single-owner retry budgets.
See [`MIGRATION.md#v0470`](MIGRATION.md#v0470) for local `!Send` futures,
explicit cancellation policy, and local prepared execution.
See [`MIGRATION.md#v0480`](MIGRATION.md#v0480) for bounded streaming policy,
source/sink contracts, replay identity, and partial-state cleanup.
See [`MIGRATION.md#v0490`](MIGRATION.md#v0490) for bounded incremental
provider decoding across arbitrary input chunks.
See [`MIGRATION.md#v0500`](MIGRATION.md#v0500) for exhaustive compile-time
Hetzner operation associations.

## Guard Preparation Storage

`PreparationStorageGuard` keeps both complete request buffers under one
volatile-clearing owner. A returned prepared request borrows the guard, so it
cannot safely outlive cleanup ownership. Each preparation attempt clears both
complete buffers before reuse, and dropping the guard clears them again:

```rust
use cloud_sdk::operation::PreparationStorageGuard;

let mut target = [0_u8; 1024];
let mut body = [0_u8; 16 * 1024];
{
    let storage = PreparationStorageGuard::new(&mut target, &mut body);
    assert_eq!(storage.capacities(), (1024, 16 * 1024));
}
assert!(target.iter().all(|byte| *byte == 0));
assert!(body.iter().all(|byte| *byte == 0));
```

Use `PreparationCapacityProfile::EMBEDDED`, `DEFAULT`, or `LARGE` to validate
caller storage. With the opt-in `alloc` feature,
`OwnedPreparationStorage::try_for_profile` allocates the same bounded regions
fallibly. No allocation is introduced by default.

Provider crates can use `Method::extension("PURGE")` for a finite static
extension. Extensions are bounded uppercase HTTP tokens; known aliases,
CONNECT, and TRACE are rejected. See
[`MIGRATION.md#v0330`](MIGRATION.md#v0330).

The complete compile-checked source is
[`provider_neutral.rs`](../crates/cloud-sdk/examples/provider_neutral.rs). Run
it with:

```sh
cargo run -p cloud-sdk --example provider_neutral
```

The complete prepared-operation contract is demonstrated in
[`prepared_request.rs`](../crates/cloud-sdk/examples/prepared_request.rs):

```sh
cargo run -p cloud-sdk --example prepared_request
```

## Define Provider-Owned Identity

Provider crates use open marker traits rather than registering variants in a
central enum:

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

See [`MIGRATION.md#v0320`](MIGRATION.md#v0320) for direct
`ProviderService` construction and migration from the removed closed enums.

## Bind Endpoint Trust

`EndpointPolicy` admits fixed, finite official-set, provider-derived regional,
and explicitly acknowledged custom destinations. Provider operations carry
the policy into `PreparedRequest`; execution checks it before credentials are
sent. See [`MIGRATION.md#v0340`](MIGRATION.md#v0340).

## Select A Transport

- Use `cloud-sdk-testkit` for deterministic blocking and async unit tests.
- Implement `BlockingTransport`, `AsyncTransport`, or `LocalAsyncTransport`
  for a platform-native transport.
- Enable `cloud-sdk-reqwest/blocking-rustls`,
  `blocking-rustls-webpki-roots`, `blocking-rustls-fips`, or `async-rustls`
  when the supported native reqwest boundary fits the target.

The reqwest crate also exposes credential-free `RawBlockingClient` and
`RawAsyncClient` executors. They consume an explicit `RawResponsePolicy`,
retain only admitted response headers, and return delivery-phased failures.
See [`RAW_HTTP_EXECUTOR.md`](RAW_HTTP_EXECUTOR.md).

All transport traits send through `&self`. Thread-safe implementations can be
shared under caller-selected concurrency limits without a mutex held across I/O
or `.await`; local implementations may return `!Send` futures and use `!Sync`
state. The SDK does not create tasks, queues, retries, sleeps, or an executor.

The FIPS blocking feature additionally requires an explicit `FipsTlsPolicy`
containing deployment-managed trust roots and complete, current CRLs. Missing,
unknown, malformed, or expired revocation state fails closed.

Provider crates do not depend on transport crates. This keeps cloud request
models portable to Linux, Windows, BSD, macOS, Android, iOS, WASM, embedded
targets, and future operating systems while allowing each application to own
its networking and runtime policy.

## Prepare And Check Operations

`PrepareOperation` turns typed provider input plus caller-owned target/body
storage into one `PreparedRequest`. The result carries an immutable endpoint
trust policy, explicit read-only/mutation/destructive impact, request semantics,
retry eligibility, cost intent, accepted statuses and media types, body shape,
and maximum response length.

For read-only metadata, `PreparedRequest::execute_blocking`, `execute_async`,
and `execute_local_async` verify endpoint policy before sending and lend no more
than the policy's admitted response capacity through a sealed `ResponseWriter`.
A cleanup-owning `ResponseBuffer`
volatile-clears the complete caller buffer before admission and on every exit;
provided transports additionally acquire a transactional `ResponseAttempt`
that clears uncommitted body and header state on failure, unwind, timeout, or
async cancellation before writer reuse. Blocking custom transports call
`ResponseWriter::begin_attempt`; async implementations receive only
`AsyncResponseStaging` and must be invoked through SDK-owned drivers.
an optional `ResponseStorageSanitizer` can add platform cleanup without
replacing the mandatory core clear. They return
`CheckedResponseGuard` only after status, body shape, initialized length, and
validated response content type pass. Borrowed decoding is closure-scoped;
owned decoding clears body, temporary metadata, request identifiers, cursor or
provider-link staging, and decoder scratch before returning. Execution never
retries, sleeps, schedules work, or selects a clock.

Mutation, destructive, and cost-bearing metadata cannot use those direct
methods. Build a versioned exact or strong-digest `PlanConfirmation`, then
consume a `MutationPermit`, `DestructivePermit`, or `CostPermit` attempt. The
permit binds the complete wire request, endpoint, scope, validity, replay
intent, caller context, and price ceiling, and fails closed after uncertain
delivery. See [`EXECUTION_PERMITS.md`](EXECUTION_PERMITS.md).

## Continue

- [Hetzner workflow examples](HETZNER_EXAMPLES.md)
- [Security recipes](SECURITY_RECIPES.md)
- [Plan-confirm execution permits](EXECUTION_PERMITS.md)
- [Platform support](PLATFORM_SUPPORT.md)
- [Live smoke testing](LIVE_SMOKE_TESTING.md)
