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

The crate is an unreleased `1.1.0` candidate. Seven discovery, three catalog
and five version operations, four download/statistics operations and six public
account/ownership operations have checked blocking, local-async and Send-async
execution. Commit 20 is accepted; Commit 21 is ready for incremental pentest.
The `RegistryClient` facade executes eight personal mutation operations,
including both API-token and consumed secret-path variants, in all three modes.
Three token-management operations use the same checked execution boundary.
Two settings PATCH operations also use explicit permits and checked postconditions.
Owner additions/removals use consumed consent, conservative acknowledgements
and an optional local removal preflight.
Cargo yank/unyank uses bodyless single-attempt mutations, checked acknowledgements
and explicit version-state read-back.
Publishing adds validated metadata, exact binary framing, single-use authority
and bounded streaming through bundled blocking/local-async/Send-async adapters,
or custom adapters implementing the neutral streaming contracts.
Trusted publishing adds GitHub/GitLab configuration management, unverified OIDC
preflight and exchange, and protected temporary-token revocation.
Authentication preparation, endpoint, query and response foundations
are available. The blocking/local/Send facade, official bundled constructors and
anonymous artifact streaming have three-mode fixture coverage for all 51 API
operations. Independent Cargo wire fixtures and local qualification pass;
Commit 21 local qualification passed; its pentest and GitHub acceptance remain open. This is not yet a
release-qualified crates.io provider.

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
| Catalog operations | web/Cargo search, named and literal-new metadata, includes and checked continuation |
| Version operations | seek-paged versions, exact detail, dependencies, deprecated empty authors and JSON README location |
| Download operations | JSON archive location, crate/version count windows and bounded reverse dependencies |
| Public accounts and owners | user lookup with linked accounts, user statistics, team lookup and combined/user/team owner lists |
| Personal workflows | follow/unfollow, invitation accept/decline, token acceptance, email confirmation/resend, single-setting user updates and legacy notifications; explicit permits and three-mode unified execution |
| Token management | lookup by ID, explicit revoke-by-ID and self-revocation; protected scope/expiry metadata and single-use permits |
| Crate/version settings | trusted-publishing-only policy, yank state and explicit message replacement/clearing; three-mode unified execution |
| Ownership mutations | Cargo-compatible additions/removals, explicit namespaces, destructive confirmation and optional self/last-owner preflight; three-mode unified execution |
| Cargo yank/unyank | exact bodyless DELETE/PUT, consumed consent, checked acknowledgements and explicit state observation; three-mode unified execution |
| Cargo publish | bounded metadata, exact little-endian framing, borrowed/streaming archives, API or temporary token consent and checked warnings; opt-in bundled blocking/local-async/Send-async upload |
| Trusted publishing | GitHub/GitLab list/create/delete, assertion exchange and temporary-token revocation; local deadline/crate restrictions, not a JWT authenticator; three-mode unified execution |
| Artifact streaming | opt-in bundled static-origin live body sources and SHA-256; caller-supplied transactional sink remains required |
| Unified execution | blocking, local-async and Send-async typed reads and permits, including secret-path personal operations, plus streaming publish methods accepting a source; 51/51 three-mode fixtures pass, incremental pentest pending |

See the [Commit 20 implementation ledger](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_UNIFIED_CLIENT.md)
for exact remaining gates. Do not treat these foundations as full-provider qualification.
Public ownership reads do not grant mutation authority.
The complete 51-operation scope is maintained in the
[crates.io source lock](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_SOURCE_LOCK.md).

## Official Blocking Client

Enable `blocking-rustls` for the opt-in bundled transport. The default crate
still has no network dependency. Mutation calls take the existing consumed
permits, not unconfirmed requests. A schedule error asks the caller to schedule
a later attempt; the client never sleeps or retries a mutation automatically.

```rust,no_run
# #[cfg(feature = "blocking-rustls")]
# fn example() -> Result<(), Box<dyn std::error::Error>> {
use cloud_sdk_cratesio::{
    bundled::{RequestTimeouts, production_blocking},
    client::{RegistryBuffers, RegistryClient},
    discovery::DiscoveryRequest,
    wire::IdentifyingUserAgent,
};
use std::time::Duration;

let identity = IdentifyingUserAgent::new("my-tool/1 (ops@example.org)")?;
let timeouts = RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(5))?;
let transport = production_blocking(identity, timeouts)?;
let client = RegistryClient::production(&transport, identity, 65_536)?;
let mut response = vec![0; 65_536];
let mut headers = [0; 1024];
let metadata = client.execute(DiscoveryRequest::site_metadata(), RegistryBuffers {
    credential: &mut [],
    body: &mut [],
    response: &mut response,
    headers: &mut headers,
})?;
# let _ = metadata;
# Ok(())
# }
```

## Official Async Client

Enable `async-rustls` for bundled Tokio-backed transport, or `async` for a
caller-provided raw executor implementing the explicit authorization contract.
`execute_async` returns a Send future;
`execute_local` also accepts non-Send executors. Both install cleanup guards
before returning the future, share the process rate gate, and never retry or
sleep implicitly. Poll bundled transports inside a Tokio runtime.

```rust,no_run
# #[cfg(feature = "async-rustls")]
# async fn example(token: &cloud_sdk_cratesio::credentials::ApiToken)
#     -> Result<(), Box<dyn std::error::Error>> {
use cloud_sdk_cratesio::{
    bundled::{RequestTimeouts, production_async},
    catalog::CatalogRequest,
    client::{RegistryBuffers, RegistryClient},
    query::Parameter,
    wire::IdentifyingUserAgent,
};
use std::time::Duration;

let identity = IdentifyingUserAgent::new("my-tool/1 (ops@example.org)")?;
let transport = production_async(identity,
    RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(5))?)?;
let client = RegistryClient::production(&transport, identity, 65_536)?;
let parameters = [Parameter::Following];
let mut credential = [0; 1024];
let mut response = vec![0; 65_536];
let mut headers = [0; 1024];
let followed = client.catalog_with_token_async(
    CatalogRequest::list(&parameters)?, token,
    RegistryBuffers {
        credential: &mut credential, body: &mut [],
        response: &mut response, headers: &mut headers,
    },
).await?;
# let _ = followed;
# Ok(())
# }
```

Mutation execution takes the same consumed permits as the blocking facade.
An error or cancellation may follow a committed provider mutation; reconcile
the outcome before explicitly authorizing another attempt.

## Trusted Publishing Intent

```rust
# #[cfg(feature = "alloc")] {
use cloud_sdk_cratesio::{identifiers::CrateName,
    trusted_publishing::{Publisher, PublisherConfig, ExchangePolicy}};
let config = PublisherConfig::new(
    Publisher::GitHub, CrateName::new("example")?, "example-org",
    "example", "release.yml", Some("production"),
)?;
// Supply fresh trusted Unix time in the integration, not assertion claims.
let policy = ExchangePolicy::new(config, "crates.io", 1_790_400_000, 600)?;
// Consume a protected OidcAssertion with TrustedPublishingPermit::confirm_exchange.
// TrustedPublishingClient sends it once, in JSON, without Authorization.
# let _ = policy;
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

The registry verifies OIDC signatures and authorization. Local preflight only
rejects obvious issuer, audience, workflow and time mismatches. The response
does not carry scope or expiry; `TemporaryToken` applies a caller-selected local
deadline (at most 30 minutes from policy creation) and intended crate restriction.
`None` for environment deliberately permits any environment upstream. Revoke
with fresh explicit consent after use; failures never trigger automatic retries.
See the [trusted publishing policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_TRUSTED_PUBLISHING_POLICY.md)
for the conservative input profile, clock and trusted-adapter requirements.

## Publish Intent

```rust
# #[cfg(feature = "alloc")] {
use cloud_sdk::transport::StreamLimits;
use cloud_sdk_cratesio::publishing::{PublishMetadata, PublishRequest};
let json = br#"{"name":"example","vers":"1.0.0","deps":[],"features":{},"authors":[],"keywords":[],"categories":[],"license":"MIT"}"#;
let metadata = PublishMetadata::from_json(json)?;
let limits = StreamLimits::new(1_048_576, 4096, 4096, 8192, 2)?;
let request = PublishRequest::new(metadata, 1024, limits)?;
assert_eq!(request.archive_length(), 1024);
assert!(!request.permits_automatic_retry());
// Supply an actual 1024-byte .crate archive, then explicitly confirm_api(&token)
// or confirm_trusted(&temporary_token) for a checked publish client.
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

Metadata parsing and intents need `alloc`; `PublishClient` needs `blocking` or `async`.
The adapter must stream exactly one authenticated exchange with the declared
length. No archive is built or inspected, and a successful acknowledgement does
not prove index propagation. The immutable source bytes remain caller-owned.
See the [publish contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_PUBLISH_POLICY.md)
for validation limits, adapter obligations and cleanup boundaries.

### Bundled Publication

The bundled facade accepts an explicit permit and an already packaged source.
`publish` needs `blocking-rustls`; `publish_async` and `publish_local` need
`async-rustls`. The body region is reusable upload scratch, not an archive copy.

```rust,no_run
# #[cfg(feature = "blocking-rustls")]
fn publish_package(
    token: &cloud_sdk_cratesio::credentials::ApiToken,
    metadata: &[u8],
    archive: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    use cloud_sdk::transport::StreamLimits;
    use cloud_sdk_cratesio::{bundled, client::{RegistryBuffers, RegistryClient},
        publishing::{PublishMetadata, PublishRequest, SlicePackage},
        wire::IdentifyingUserAgent};
    let identity = IdentifyingUserAgent::new("my-registry-tool/1 (ops@example.org)")?;
    let timeouts = bundled::RequestTimeouts::new(
        std::time::Duration::from_secs(60), std::time::Duration::from_secs(5))?;
    let transport = bundled::production_blocking(identity, timeouts)?;
    let client = RegistryClient::production(&transport, identity, 65_536)?;
    let limits = StreamLimits::new(16_777_216, 4096, 8192, 65_536, 2)?;
    let permit = PublishRequest::new(PublishMetadata::from_json(metadata)?,
        u64::try_from(archive.len())?, limits)?.confirm_api(token);
    let mut source = SlicePackage::new(archive);
    let mut credential = [0; 1024];
    let mut body = [0; 4096];
    let mut response = vec![0; 65_536];
    let mut headers = [0; 4096];
    let _acknowledgement = client.publish(permit, &mut source, RegistryBuffers {
        credential: &mut credential, body: &mut body,
        response: &mut response, headers: &mut headers,
    })?;
    Ok(())
}
# fn main() {}
```

An error or cancellation can follow a remote mutation. Do not automatically
retry publication; successful acknowledgement does not prove index propagation.
The source must cooperate with execution: a blocking callback or an async poll
that never returns cannot be preempted by the transport deadline.

## Cargo Yank Intent

```rust
# #[cfg(feature = "alloc")] {
use cloud_sdk_cratesio::{identifiers::{CrateName, Version}, publishing::YankRequest};
let intent = YankRequest::yank(CrateName::new("example")?, Version::new("1.2.3")?);
let mut path = [0; 256];
assert_eq!(intent.write_target(&mut path)?.as_str(), "/api/v1/crates/example/1.2.3/yank");
assert!(!intent.operation().permits_automatic_retry());
// Execution requires intent.confirm(&token) and a trusted credential adapter.
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

An `ok: true` response is only an acknowledgement. `YankAcknowledgement` provides
an anonymous `verification_request()` for the same exact crate/version and a
`decode_observed_state()` decoder; inspect `matches_requested_state()` rather
than assuming index propagation or replaying after an ambiguous error. The
read-back must use `verification_endpoint()`. Cargo yank/unyank clears an existing
yank message; use the separate settings API for explicit message edits. See the
[yank contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_YANK_POLICY.md).

## Ownership Intent

```rust
# #[cfg(feature = "alloc")] {
use cloud_sdk_cratesio::{identifiers::CrateName,
    ownership::{OwnerSelector, OwnerChangeRequest}};
let owners = [OwnerSelector::new("crates.io:example-user")?];
let request = OwnerChangeRequest::add(CrateName::new("example")?, &owners)?;
let mut scratch = [0; 512];
request.with_json_body(&mut scratch, |body| {
    assert_eq!(body, br#"{"users":["crates.io:example-user"]}"#);
})?;
// request.confirm_add(&api_token)? authorizes one trusted-adapter call.
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

Removal uses `confirm_removal` or `confirm_removal_after_preflight`, never an
addition permit. Acknowledgements do not prove invitation acceptance or echo
the crate identity. See the [ownership contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_OWNERSHIP_POLICY.md)
for namespace, snapshot and concurrent-change boundaries.

## Settings Intent

```rust
# #[cfg(feature = "alloc")] {
use cloud_sdk_cratesio::{identifiers::{CrateName, Version},
    settings::{SettingsRequest, YankMessage}};
let patch = SettingsRequest::version(
    CrateName::new("example")?, Version::new("1.0.0")?,
    Some(true), YankMessage::Set("Use the corrected release"),
)?;
let mut scratch = [0; 1024];
patch.with_json_body(&mut scratch, |body| assert!(!body.is_empty()))?;
assert!(scratch.iter().all(|byte| *byte == 0));
// Explicit patch.confirm(&api_token) is required before SettingsClient::execute.
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

`YankMessage::Clear` explicitly removes a message. There is no preserve-message
variant: upstream omission also clears it. Settings do not edit descriptions,
URLs or archived state. No automatic retries or compare-and-swap protection is
claimed; see the [settings contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_SETTINGS_POLICY.md).

## Token Management Intent

```rust
# #[cfg(feature = "alloc")] {
use cloud_sdk_cratesio::{accounts::tokens::TokenPermit,
    credentials::ApiToken, identifiers::NumericId};

fn inspect(credential: &ApiToken, id: NumericId) {
    let permit = TokenPermit::inspect(id, credential);
    assert!(!permit.operation().is_destructive());
}
fn confirm_retirement(old_credential: &ApiToken) {
    let permit = TokenPermit::confirm_revoke_current(old_credential);
    assert!(permit.operation().is_destructive());
    // TokenClient::execute consumes this permit with the trusted adapter.
}
# }
```

Provision and validate a replacement out of band before revoking the old token.
Revocation is never automatic; an execution error may follow a successful
upstream mutation. Scope metadata is not authority, and lookup does not prove
that a token remains usable. See the
[token contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_TOKEN_POLICY.md)
for exact response semantics, limits, cleanup and adapter requirements.

## Personal Workflow Intent

```rust
# #[cfg(feature = "alloc")] {
use cloud_sdk_cratesio::{accounts::personal::PersonalRequest,
    credentials::ApiToken, identifiers::CrateName};
fn follow_intent(token: &ApiToken) -> Result<(), Box<dyn std::error::Error>> {
    let permit = PersonalRequest::follow(CrateName::new("serde")?).confirm(token);
    // Pass this consumed permit to PersonalClient::execute with a trusted adapter.
    assert_eq!(permit.operation().operation_name(), "follow_crate");
    Ok(())
}
# }
```

`PersonalClient` requires `blocking`. Its adapter must use the exact fixed
origin, method, target, body and response policy, apply authorization as
sensitive, and disable cookies, redirects and retries. It is not interchangeable
with the anonymous raw executor. Email/invitation path tokens are consumed;
their targets and scratch must never be logged. Provider identity and expiry
checks remain server-side. See the
[personal workflow contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_PERSONAL_POLICY.md)
for partial-update, ambiguity and deprecated-endpoint limitations.

## Public Ownership Example

Anonymous ownership inspection uses the same bound executor and response
cleanup as discovery. Returned user/team records are metadata, not permission
to modify a crate.

```rust
# #[cfg(feature = "blocking")] {
use cloud_sdk::transport::{BlockingRawHttpExecutor, BoundTransport, BoundUserAgent};
use cloud_sdk_cratesio::{
    accounts::{AccountClient, AccountRequest, AccountResponse},
    identifiers::CrateName,
    wire::IdentifyingUserAgent,
};

fn owners<T>(executor: &T) -> Result<AccountResponse, Box<dyn std::error::Error>>
where
    T: BlockingRawHttpExecutor + BoundTransport + BoundUserAgent,
    T::Error: 'static,
{
    let identity = IdentifyingUserAgent::new("inventory/1.0 (ops@example.org)")?;
    let client = AccountClient::production(executor, identity, 65_536)?;
    let request = AccountRequest::owners(CrateName::new("serde")?);
    let mut body = vec![0; 65_536];
    let mut headers = [0; 512];
    Ok(client.execute(request, &mut body, &mut headers)?)
}
# }
```

`AccountRequest::user`, `user_stats`, `team`, `user_owners` and `team_owners`
cover the other public account reads. `execute_local` and `execute_async`
provide the same checks under `async`. See the
[public account contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_ACCOUNT_POLICY.md)
for namespace rules, bounds and linked-account inclusion.

## Cargo Owner Inspection

For Cargo-compatible owner listing, explicitly choose the token-authenticated
profile instead of the anonymous website response model. The minimal Cargo
records do not infer user/team kinds or grant ownership-change authority.

```rust
# #[cfg(feature = "blocking")] {
use cloud_sdk::transport::{BlockingAuthorizedRawHttpExecutor, BoundUserAgent};
use cloud_sdk_cratesio::{
    accounts::cargo::{CargoOwners, CargoOwnersRequest},
    client::{RegistryBuffers, RegistryClient},
    credentials::ApiToken,
    identifiers::CrateName,
    wire::IdentifyingUserAgent,
};
fn cargo_owners<T>(executor: &T, token: &ApiToken)
    -> Result<CargoOwners, Box<dyn std::error::Error>>
where T: BlockingAuthorizedRawHttpExecutor + BoundUserAgent, T::Error: 'static,
{
    let identity = IdentifyingUserAgent::new("inventory/1.0 (ops@example.org)")?;
    let client = RegistryClient::production(executor, identity, 65_536)?;
    let mut credential = [0; 2048];
    let mut response = vec![0; 65_536];
    let mut headers = [0; 512];
    Ok(client.execute(CargoOwnersRequest::new(CrateName::new("serde")?, token),
        RegistryBuffers { credential: &mut credential, body: &mut [],
            response: &mut response, headers: &mut headers })?)
}
# }
```

The same request supports `execute_local` and `execute_async` under `async`.
All supplied scratch is cleared; caller-created copies remain caller-owned.

## Install

This provider remains unpublished. For the candidate examples, use a checkout
with its complete workspace, as opposed to selecting a published package:

```sh
cargo add cloud-sdk --path /path/to/cloud-sdk/crates/cloud-sdk
cargo add cloud-sdk-cratesio --path /path/to/cloud-sdk/crates/cloud-sdk-cratesio --features blocking
```

For published SDK crates, `cargo add` without a path selects registry metadata
and writes version requirements to your application's manifest. Workspace
security pins and lockfile review remain separate from these install examples.

## Download Counts Example

```rust
use cloud_sdk_cratesio::{
    downloads::DownloadRequest,
    identifiers::{CrateName, Date, Version},
    query::Parameter,
};

let parameters = [Parameter::BeforeDate(Date::new("2026-09-01")?)];
let request = DownloadRequest::version_counts(
    CrateName::new("serde")?, Version::new("1.0.0")?, &parameters,
)?;
let mut storage = [0; 256];
assert_eq!(request.write_target(&mut storage)?.as_str(),
    "/api/v1/crates/serde/1.0.0/downloads?before_date=2026-09-01");
# Ok::<(), Box<dyn core::error::Error>>(())
```

Use `DownloadClient::production` with the same checked executor, identifying
user-agent and caller response buffers shown for the version client below.
Artifact streaming is separate from JSON response execution. The optional
`bundled::ArtifactTransport` supplies anonymous live streaming and
`downloads::Sha256Checksum` supplies the reviewed hash implementation. A
caller-owned transactional sink is still required; see the
[download contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_DOWNLOAD_POLICY.md).
Prefer static CDN downloads and database dumps for bulk work.

## Version Example

List versions with an explicit page size. The source supports seek pagination,
not numbered pages, and omitting `per_page` would request an unpaginated list.

```rust
use cloud_sdk_cratesio::{identifiers::CrateName, query::{Parameter, PerPage}, versions::VersionRequest};
let name = CrateName::new("serde")?;
let parameters = [Parameter::PerPage(PerPage::DEFAULT)];
let request = VersionRequest::list(name, &parameters)?;
let mut target = [0; 256];
assert_eq!(request.write_target(&mut target)?.as_str(),
    "/api/v1/crates/serde/versions?per_page=10");
# Ok::<(), Box<dyn std::error::Error>>(())
```

`versions::VersionClient` uses the same `production`/`staging` constructors and
`execute`, `execute_local`, and `execute_async` pattern as `CatalogClient`
below. Select `detail`, `dependencies`, `authors`, or `readme` with a checked
crate name and exact `identifiers::Version`. The README result is an inert URL,
never rendered HTML or permission to forward credentials. Authors is a
deprecated empty compatibility response. Requirement and target strings are
bounded metadata, not a dependency resolver or executable configuration.
See the [version contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_VERSION_POLICY.md).

## Catalog Example

Use the API for interactive discovery, not dependency resolution or bulk crawling.
Prefer Cargo's sparse index or the database dump for those workloads.

```rust
use cloud_sdk_cratesio::{
    catalog::{CatalogOperation, CatalogRequest},
    identifiers::CrateName,
    query::{Parameter, SearchQuery},
};

let params = [Parameter::Search(SearchQuery::new("serde")?)];
let search = CatalogRequest::list(&params)?;
let mut target = [0; 1024];
assert_eq!(search.write_target(&mut target)?.as_str(), "/api/v1/crates?q=serde");
let metadata = CatalogRequest::crate_metadata(CrateName::new("new")?, &[])?;
assert_eq!(metadata.operation(), CatalogOperation::NewCrate);
# Ok::<(), Box<dyn std::error::Error>>(())
```

With `blocking`, supply a trusted raw executor bound to `https://crates.io`
and the identifying user-agent configured below:

```rust
# #[cfg(feature = "blocking")]
# {
use cloud_sdk::transport::{BlockingRawHttpExecutor, BoundTransport, BoundUserAgent};
use cloud_sdk_cratesio::{
    catalog::{CatalogClient, CatalogRequest, CatalogResponse},
    identifiers::CrateName,
    query::{Include, IncludeSet, Parameter},
    wire::IdentifyingUserAgent,
};

fn metadata<T>(executor: &T) -> Result<CatalogResponse, Box<dyn std::error::Error>>
where
    T: BlockingRawHttpExecutor + BoundTransport + BoundUserAgent,
    T::Error: 'static,
{
    let identity = IdentifyingUserAgent::new("inventory/1.0 (ops@example.org)")?;
    let client = CatalogClient::production(executor, identity, 65_536)?;
    let includes = [Include::DefaultVersion];
    let params = [Parameter::Include(IncludeSet::new(&includes)?)];
    let request = CatalogRequest::crate_metadata(CrateName::new("serde")?, &params)?;
    let mut body = vec![0; 65_536];
    let mut headers = [0; 512];
    Ok(client.execute(request, &mut body, &mut headers)?)
}
# }
```

`cargo_search` selects Cargo's minimal response format; `list` selects the full
crates.io web schema. `execute_local` and `execute_async` provide the same
anonymous checks under `async`. Every call shares the discovery rate gate and
may return `ScheduleError::Wait`; it never sleeps or retries for you.
Response buffers clear on every return or cancelled/unpolled future.
Provider `Retry-After` is capped at 24 hours (`MAX_PROVIDER_DELAY`). Larger
delays reject the response with a scheduling error and impose a capped wait,
without permanently disabling the shared gate. No automatic retry occurs.

`CatalogContinuation::LimitReached` is not end-of-data. Crate links are inert
untrusted metadata, and include-expanded versions expose schema-checked protected
fields. Optional raw API-token list execution is available through the registry
facade's `catalog_with_token`, `catalog_with_token_local` and
`catalog_with_token_async` methods, or the lower-level blocking trusted callback.
These use the raw API token, not a Bearer prefix. Anonymous `following` requests reject.
See the [catalog contract](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_CATALOG_POLICY.md)
for source coverage, pagination, includes, resource limits and adapter obligations.

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
| `alloc` | no | Enables protected credentials, checked JSON admission and discovery/catalog models; still `no_std`. |
| `serde` | no | Enables the Serde boundary for later serialization; discovery decoding reuses core JSON events. |
| `std` | no | Enables `alloc` and the shared monotonic API gate. |
| `blocking` | no | Enables checked blocking clients and unified read/permit execution; no transport dependency is added. |
| `artifact-sha256` | no | Adds the reviewed no_std SHA-256 implementation for archive integrity. |
| `blocking-rustls` | no | Adds official blocking constructors, neutral authorized execution, live artifact reads and SHA-256. |
| `async-rustls` | no | Adds official Tokio-backed async constructors, live artifact reads and SHA-256. |
| `async` | no | Enables unified checked local/Send async reads and permits; no runtime or transport dependency is added. |

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
routes. Operation-bound clients, consumed mutation permits, scheduling and
bounded response admission are implemented. Bundled streaming publication is
implemented, including secret-path execution. Local integration qualification
is recorded in the Commit 20 ledger above; ongoing qualification is tracked in
[Commit 21](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_QUALIFICATION.md).
Custom adapters must
not log targets or retain unprotected secret URI copies. Bundled raw adapters
clear owned URI staging; upstream HTTP/TLS buffers and server/proxy logs remain
deployment boundaries.

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
