<p align="center">
  <b>Security-first, no_std crates.io API provider for cloud-sdk.</b><br>
  Provider-owned identities and bounded API domains on a transport-neutral foundation.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk-cratesio">Crates.io</a>
  |
  <a href="https://docs.rs/cloud-sdk-cratesio">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk">cloud-sdk</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/cratesio-commit-plan.md">Implementation Plan</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/SECURITY.md">Security</a>
</div>

# cloud-sdk-cratesio

`cloud-sdk-cratesio` is the crates.io provider crate for the main
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) project. It owns crates.io
provider identities, API models, request preparation, checked response
decoding, authentication rules, and high-level workflows while reusing the
provider-neutral execution contracts from `cloud-sdk`.

The crate is an unreleased `1.1.0` candidate. Seven anonymous discovery
operations now have typed requests, complete success models and checked
blocking, local-async and Send-async execution. Commit 8 passed its incremental
pentest and remediation retest; its evidence checkpoint awaits GitHub approval.
Authentication preparation, endpoint, query and response foundations
are available, but authenticated clients and the other API workflows remain
assigned to later checkpoints. This is not yet a complete crates.io provider.

## Current Boundary

| Area | State |
| --- | --- |
| Provider identity | `crates-io` |
| Service identity | `registry` |
| Default features | empty |
| Default target | `no_std` |
| Official endpoints | production API, staging API, and anonymous static downloads |
| Request targets | bounded `/api/v1/` API and `/crates/` static-download forms |
| Redirects | atomic production source proof plus atomic credential-free download execution |
| Custom API endpoints | HTTPS plus explicit trusted-operator acknowledgement |
| Credentials | five protected token kinds, origin-bound contexts, scoped adapter material (`alloc`) |
| JSON wire admission | exact status, bounded complete JSON, Cargo error detection and cleanup (`alloc`) |
| Request policy | identifying user agent and a process-wide gate across blocking and async discovery |
| Identifiers and queries | bounded public identifiers, operation-specific values and atomic percent encoding |
| Pagination | checked meta links, legacy `more`, and explicit traversal limits |
| Discovery operations | categories, category slugs, keywords, site metadata and complete front-page summary |
| Other API operations | deferred to their source-locked implementation commits |

The public modules reserve ownership without claiming executable coverage:
`catalog`, `accounts`, `ownership`, `publishing`, and `trusted_publishing`.
The complete 51-operation scope is maintained in the
[crates.io source lock](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_SOURCE_LOCK.md).

## Discovery Example

| API operation | Request/target | Success/error decode | Blocking/local/Send async |
| --- | --- | --- | --- |
| Category list and detail | Implemented | Implemented | Implemented |
| Category slugs | Implemented | Implemented | Implemented |
| Keyword list and detail | Implemented | Implemented | Implemented |
| Site metadata | Implemented | Implemented | Implemented |
| Front-page summary | Implemented | Implemented | Implemented |

Enable `blocking` and supply a trusted anonymous raw executor configured for
`https://crates.io` with the same identifying user-agent. The provider does
not choose a TLS stack or runtime for you:

```rust
# #[cfg(feature = "blocking")]
# {
use cloud_sdk::transport::{BlockingRawHttpExecutor, BoundTransport, BoundUserAgent};
use cloud_sdk_cratesio::{
    discovery::{DiscoveryClient, DiscoveryRequest, DiscoveryResponse},
    wire::IdentifyingUserAgent,
};

fn inspect<T>(executor: &T) -> Result<DiscoveryResponse, Box<dyn std::error::Error>>
where
    T: BlockingRawHttpExecutor + BoundTransport + BoundUserAgent,
    T::Error: 'static,
{
    let identity = IdentifyingUserAgent::new("inventory/1.0 (ops@example.org)")?;
    let client = DiscoveryClient::production(executor, identity, 65_536)?;
    let mut body = vec![0; 65_536];
    let mut headers = [0; 512];
    Ok(client.execute(DiscoveryRequest::site_metadata(), &mut body, &mut headers)?)
}
# }
```

Under `async`, `execute_local` admits non-Send executors and
`execute_async` returns a Send future for a shared Sync executor. Every call
checks origin, identifying user-agent, exact status, media type, response bounds
and schema. Response/header buffers clear on return, error or cancellation,
including an unpolled future. No credentials, retries, redirects or sleeps are
added. The process-wide rate gate can return `ScheduleError::Wait`; callers
decide when to try again and must coordinate shared egress across processes.

Models retain every source-required field, explicit nullability and checked
timestamps. Unknown values are bounded; returned descriptions, links and banner
text are untrusted data, not HTML or routing authority. Pagination reports
`LimitReached` when more data exists beyond the SDK's numbered-page ceiling.
See the [discovery contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_DISCOVERY_POLICY.md)
for all bounds, source fixtures and execution guarantees.

## Endpoint Example

Official routing remains allocation-free and transport-neutral:

```rust
use cloud_sdk_cratesio::endpoint::{
    ApiRequestTarget, OfficialCratesIoEndpoint,
};

let endpoint = OfficialCratesIoEndpoint::production_api();
assert_eq!(endpoint.base_url(), "https://crates.io");

let target = ApiRequestTarget::new("/api/v1/crates?q=serde");
assert!(target.is_ok());
```

Custom API origins require
`CustomEndpointAcknowledgement::trusted_operator_configuration()` and an
already validated HTTPS `EndpointIdentity`. Values must never come from tenant,
request, webhook, or other attacker-controlled input. Static download
redirects accept only the exact `https://static.crates.io` authority, correlate
the crate and version with the source API target, and can be followed only
through a raw executor using an SDK-created bodyless `GET` with empty headers.
Source proof creation also dispatches through the exact production-bound raw
executor using an SDK-created bodyless `GET`, empty headers, and an exact
response policy after validating the exact version-download route; invalid
generic API targets never reach the executor. Callers cannot combine an
unrelated response with a separately verified transport. The redirect does not expose endpoint or target pieces
that could be reused by an authenticated request. The complete contract is
documented in the [crates.io endpoint policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_ENDPOINT_POLICY.md).

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps the provider allocation-free and `no_std`. |
| `alloc` | no | Enables protected credentials, checked JSON admission and discovery models; still `no_std`. |
| `serde` | no | Enables the Serde boundary for later serialization; discovery decoding reuses core JSON events. |
| `std` | no | Enables `alloc` and the shared monotonic API gate. |
| `blocking` | no | Enables checked anonymous blocking discovery; no transport dependency is added. |
| `async` | no | Enables checked local/Send async discovery; no runtime or transport dependency is added. |

Networking and TLS remain opt-in provider-neutral concerns. This crate does
not depend on `cloud-sdk-reqwest`, an async runtime, a TLS implementation, a
filesystem, or an external clock package. The explicit `std` gate reads a
monotonic clock; discovery execution also reads wall time for HTTP-date delays.

## Identity Example

```rust
use cloud_sdk::{ProviderMarker, ServiceMarker};
use cloud_sdk_cratesio::{
    CRATES_IO_PROVIDER_ID, CratesIo, REGISTRY_SERVICE_ID, RegistryService,
};

assert_eq!(CratesIo::ID, CRATES_IO_PROVIDER_ID);
assert_eq!(RegistryService::ID, REGISTRY_SERVICE_ID);
assert_eq!(
    <<RegistryService as ServiceMarker>::Provider as ProviderMarker>::ID,
    CRATES_IO_PROVIDER_ID,
);
```

## Credential Example

Enable `alloc` to ingest mutable credential buffers. Supply token bytes from
trusted caller-owned secret storage, not a string literal or repository file:

```rust
# #[cfg(feature = "alloc")]
# {
use cloud_sdk_cratesio::credentials::{ApiToken, CredentialError, CredentialOrigin};

fn load_token(source: &mut [u8]) -> Result<ApiToken, CredentialError> {
    // The entire source is cleared on success and failure.
    ApiToken::from_mut_bytes(CredentialOrigin::Production, source)
}

fn rotate_token(token: &mut ApiToken, replacement: &mut [u8]) -> Result<(), CredentialError> {
    // Rejection retains the old credential but still clears replacement.
    token.rotate_from_mut_bytes(replacement)
}
# }
```

| Protected type | Wire placement | Allowed contexts | Local maximum |
| --- | --- | --- | --- |
| `ApiToken` | raw `Authorization`, no scheme prefix | source-locked API-token routes | 1,024 bytes |
| `TrustedPublishingToken` | `Authorization: Bearer ...` | publish or revoke that temporary token | 1,017 bytes |
| `OidcAssertion` | JSON `jwt` field | temporary token exchange | 16,384 bytes |
| `EmailConfirmationToken` | confirmation path suffix | confirm email | 512 bytes |
| `OwnerInvitationToken` | invitation path suffix | accept invitation | 512 bytes |

These are SDK resource bounds, not upstream guarantees. Header tokens use a
conservative token68 lexical profile without whitespace or a supplied scheme;
OIDC accepts three nonempty base64url segments without claiming to validate
their signature or claims. Path tokens admit unreserved ASCII only and reject
dot segments. Unsupported syntax fails rather than being normalized.

Credentials are not cloneable or serializable. Their origin is immutable;
production and staging credentials cannot be interchanged or prepared for
static downloads or custom authorities. Diagnostics redact secret text.
`clear`, rotation and drop use the admitted `sanitization` implementation.

`with_material_for_adapter` is a **trusted adapter extension point**, not an
authenticated client. It verifies the supplied transport's exact bound origin
before exposing typed method/target/header/body material inside a closure.
The caller buffer is cleared before use and on return, error or unwind; safe
code cannot return a borrow into it. A callback can deliberately copy secrets
or misuse another transport, so it must honor the supplied destination and
operation, set a single sensitive Authorization field, omit cookies, disable
redirects, and clear its own copies. OIDC JSON requires `application/json`.
Do not log raw target text: confirmation/invitation targets contain secrets.

No network calls, automatic retries, OIDC acquisition, cryptographic JWT
verification, expiry enforcement or server-side permission checks are added
here. Async authenticated execution is part of later client checkpoints.
Process abort, deliberately leaked objects, adapter/OS copies and external
storage are outside drop-cleanup guarantees. Full policy and admission evidence
are in the [credential contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_CREDENTIAL_POLICY.md).

## Response And Scheduling Examples

The response policy consumes the core's cleanup-owning committed buffer and
rejects provider errors even when HTTP status is 200:

```rust
# #[cfg(feature = "alloc")]
# {
use cloud_sdk::rate_limit::WallClockTimestamp;
use cloud_sdk::transport::{ResponseBuffer, StatusCode};
use cloud_sdk_cratesio::wire::{CratesIoWireError, JsonResponsePolicy, JsonSuccess};

fn admit<'a>(response: ResponseBuffer<'a>) -> Result<JsonSuccess<'a>, CratesIoWireError> {
    let policy = JsonResponsePolicy::new(StatusCode::OK, 65_536)?;
    // Supply trusted wall time for HTTP-date interpretation in real adapters.
    policy.admit(response, WallClockTimestamp::new(0))
}
# }
```

Apply `policy.maximum_bytes()` to the response writer before transport starts.
Discovery success decoders consume this checked boundary; other resource
clients come in later checkpoints. `JsonSuccess::visit` exposes checked events.

```rust
use core::time::Duration;
use cloud_sdk_cratesio::wire::{ApiSchedule, IdentifyingUserAgent};

let identity = IdentifyingUserAgent::new("inventory/1.0 (ops@example.org)")?;
let mut schedule = ApiSchedule::new();
assert!(schedule.try_start(Duration::ZERO).is_ok());
assert!(schedule.try_start(Duration::ZERO).is_err());
assert!(schedule.try_start(Duration::from_secs(1)).is_ok());
# Ok::<(), cloud_sdk_cratesio::wire::UserAgentError>(())
```

Share one schedule across workers and supply trusted monotonic time. Under
`std`, `OfficialApiGate` provides shared process-wide scheduling for a trusted
blocking adapter callback, requiring the validated identity. It is not an
authenticated client. Discovery execution integrates its admission across all
three execution modes. No retries or sleeps happen automatically.
See the [wire policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_WIRE_POLICY.md)
for response limits, cleanup and adapter responsibilities.

## Typed Query Example

```rust
use cloud_sdk_cratesio::query::{
    ApiPath, FixedSegment, Page, Parameter, PathSegment, PerPage, Query,
    QueryOperation, Sort,
};

let segments = [PathSegment::Fixed(FixedSegment::Categories)];
let path = ApiPath::new(&segments)?;
let parameters = [
    Parameter::Sort(Sort::Alpha),
    Parameter::Page(Page::new(1)?),
    Parameter::PerPage(PerPage::new(25)?),
];
let query = Query::new(QueryOperation::Categories, &parameters)?;
let mut output = [0; 128];
let target = query.write_target(path, &mut output)?;
assert_eq!(target.as_str(), "/api/v1/categories?page=1&per_page=25&sort=alpha");
# Ok::<(), cloud_sdk_cratesio::query::QueryError>(())
```

`identifiers` also provides crate names, exact versions, category slugs, keywords,
user/team logins, owners, numeric IDs and dates. Include and sort choices are
operation-specific; duplicate parameters and conflicting filters are rejected.

## Pagination Example

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use cloud_sdk::pagination::PaginationLimits;
use cloud_sdk_cratesio::{
    endpoint::OfficialCratesIoEndpoint,
    pagination::{Direction, PageLink, Traversal},
    query::{ApiPath, FixedSegment, Parameter, PathSegment, PerPage, Query, QueryOperation},
};

let endpoint = OfficialCratesIoEndpoint::production_api();
let segments = [PathSegment::Fixed(FixedSegment::Crates)];
let path = ApiPath::new(&segments)?;
let size = PerPage::new(10)?;
let parameters = [Parameter::PerPage(size)];
let query = Query::new(QueryOperation::Crates, &parameters)?;
let next = PageLink::new(endpoint, path, query, "?page=2&per_page=10", Direction::Next)?;
let limits = PaginationLimits::new(5, 50, 128)?;
let mut traversal = Traversal::new(limits, size);
traversal.admit(10, Some(&next))?;

let mut path_storage = [0; 128];
let mut target_storage = [0; 128];
let continuation = next.transfer_to(&mut path_storage, &mut target_storage, limits)?;
drop(continuation);
assert!(target_storage.iter().all(|byte| *byte == 0));
# Ok(())
# }
```

This example starts from a decoded meta link and item count. Resource-specific
response decoders and complete operation drivers follow in later checkpoints.
The transferred core link checks endpoint, method and operation at dispatch;
callers must still apply the rate gate to every attempt. `MetaLinks` validates
both next and previous fields, and `Traversal::admit_legacy` supports Cargo's
numbered `more` responses. See the
[request policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_REQUEST_POLICY.md)
for exact input limits and source-verification commands.

## Security And Policy

The provider will not support browser-session cookies or undocumented private
routes. Scheduling and bounded response admission are available foundations;
operation-bound clients, mutation permits and async integration remain
assigned to later checkpoints.

Direct crates.io API use must follow the service's data-access policy. Prefer
the sparse index, static downloads, RSS feeds, or database dumps when those
sources fit the task.

See the main project's
[threat model](https://github.com/valkyoth/cloud-sdk/blob/main/docs/threat-model.md),
[release governance](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_GOVERNANCE.md),
and [versioning policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/VERSIONING_POLICY.md).

## License

Licensed under either the MIT License or Apache License, Version 2.0, at your
option.
