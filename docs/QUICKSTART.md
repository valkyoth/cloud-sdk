# Provider-Neutral Quickstart

`cloud-sdk` separates provider request models from transport execution. The
default graph performs no I/O, selects no runtime, stores no token, and remains
usable in `no_std` environments.

## Install

```toml
[dependencies]
cloud-sdk = "0.39.0"
```

Provider-specific request models are separate dependencies. For Hetzner:

```toml
[dependencies]
cloud-sdk = "0.39.0"
cloud-sdk-hetzner = "0.32.0"
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
[`MIGRATION_0.35.0.md`](MIGRATION_0.35.0.md).
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

See [`MIGRATION_0.36.0.md`](MIGRATION_0.36.0.md) for reserved ownership,
duplicate handling, limits, response metadata, and adapter changes.
See [`MIGRATION_0.38.0.md`](MIGRATION_0.38.0.md) for mandatory response
cleanup, protected request identifiers, and complete checked decode workspace
ownership.
See [`MIGRATION_0.39.0.md`](MIGRATION_0.39.0.md) for transactional request
encoding, guarded preparation storage, and named capacity profiles.

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
[`MIGRATION_0.33.0.md`](MIGRATION_0.33.0.md).

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

See [`MIGRATION_0.32.0.md`](MIGRATION_0.32.0.md) for direct
`ProviderService` construction and migration from the removed closed enums.

## Bind Endpoint Trust

`EndpointPolicy` admits fixed, finite official-set, provider-derived regional,
and explicitly acknowledged custom destinations. Provider operations carry
the policy into `PreparedRequest`; execution checks it before credentials are
sent. See [`MIGRATION_0.34.0.md`](MIGRATION_0.34.0.md).

## Select A Transport

- Use `cloud-sdk-testkit` for deterministic blocking and async unit tests.
- Implement `BlockingTransport` or `AsyncTransport` for a platform-native
  transport.
- Enable `cloud-sdk-reqwest/blocking-rustls`,
  `blocking-rustls-webpki-roots`, `blocking-rustls-fips`, or `async-rustls`
  when the supported native reqwest boundary fits the target.

Both transport traits send through `&self`. Thread-safe implementations can be
shared under caller-selected concurrency limits without a mutex held across I/O
or `.await`; implementations that are not `Sync` remain sequential. The SDK
does not create tasks, queues, retries, sleeps, or an executor.

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

`PreparedRequest::execute_blocking` and `execute_async` verify endpoint policy
before sending and lend no more than the policy's admitted response capacity
through a sealed `ResponseWriter`. A cleanup-owning `ResponseBuffer`
volatile-clears the complete caller buffer before admission and on every exit;
an optional `ResponseStorageSanitizer` can add platform cleanup without
replacing the mandatory core clear. They return
`CheckedResponseGuard` only after status, body shape, initialized length, and
validated response content type pass. Borrowed decoding is closure-scoped;
owned decoding clears body, temporary metadata, request identifiers, cursor or
provider-link staging, and decoder scratch before returning. Execution never
retries, sleeps, schedules work, or selects a clock.

## Continue

- [Hetzner workflow examples](HETZNER_EXAMPLES.md)
- [Security recipes](SECURITY_RECIPES.md)
- [Platform support](PLATFORM_SUPPORT.md)
- [Live smoke testing](LIVE_SMOKE_TESTING.md)
