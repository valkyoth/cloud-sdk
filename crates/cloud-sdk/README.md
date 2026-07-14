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
validated request and response building blocks. They do not yet provide
high-level provider clients that combine request construction, transport, and
response decoding end to end.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.90.0` |
| Pinned toolchain | Rust `1.97.0` |
| Default target | `no_std` |
| Default runtime dependencies | none in `cloud-sdk`; provider crates remain transport-free by default |
| Unsafe policy | first-party crates use `#![forbid(unsafe_code)]` |
| Default features | empty |
| Network defaults | none |
| Secret storage defaults | none |
| Release evidence | local gates, dependency policy, SBOM, pentest report before tags |
| Platform support | explicit tiers and targets in [`docs/PLATFORM_SUPPORT.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/PLATFORM_SUPPORT.md) |
| Crate versions | tracked in [`docs/CRATE_VERSION_MATRIX.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATE_VERSION_MATRIX.md) |
| 1.0 target | serious production-ready foundation plus complete Hetzner Cloud, DNS, Console Storage Box, and Robot provider |

## Provider Roadmap

| Provider | Target Version | Crate |
| --- | --- | --- |
| [`Hetzner Cloud`](https://www.hetzner.com/) | 1.0.0 | [`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner) |
| [`Hetzner Robot`](https://robot.hetzner.com/doc/webservice/en.html) | 1.0.0 | pre-1.0 milestones in `cloud-sdk-hetzner` |

## Rust Version Support

The minimum supported Rust version is Rust `1.90.0`. Development uses the
pinned stable Rust `1.97.0` until the toolchain policy is updated.

| Rust | Local Evidence |
| --- | --- |
| `1.90.0 - 1.96.1` | `cargo +<version> check --workspace --all-features` for every supported compiler |
| `1.97.0` | `scripts/checks.sh` |

Portable and native platform evidence is documented in
[`docs/PLATFORM_SUPPORT.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/PLATFORM_SUPPORT.md).

## Install

```toml
[dependencies]
cloud-sdk = "0.25.0"
cloud-sdk-hetzner = "0.19.3"
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
- [Release runbook](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_RUNBOOK.md)

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

The core contracts perform no I/O and select no executor. Use
`cloud-sdk-testkit` for deterministic blocking or async tests, or opt into
`cloud-sdk-reqwest/blocking-rustls`, `blocking-rustls-webpki-roots`,
`blocking-rustls-fips`, or `async-rustls` for HTTPS.

## Optional Blocking Transport

```toml
[dependencies]
cloud-sdk = "0.25.0"
cloud-sdk-reqwest = { version = "0.17.1", features = ["blocking-rustls"] }
```

The production builder is HTTPS-only, requires explicit bounded timeouts and a
user agent, uses rustls with TLS 1.2 minimum, and disables redirects, retries,
proxies, referer generation, and response decompression. It forces HTTP/1 and
the system resolver even if another dependency enables reqwest HTTP/2 or
Hickory DNS. The caller owns token generation, scope, rotation, revocation,
and cleanup of the original secret; the adapter clears only its own token and
request-body storage.

See the complete, compile-checked
[`cloud-sdk-reqwest` blocking example](https://docs.rs/cloud-sdk-reqwest/latest/cloud_sdk_reqwest/#blocking-example)
for client construction and request execution.

### Optional Deterministic Root Snapshot

Use a source-pinned Mozilla root snapshot instead of host trust-store contents
when deterministic public WebPKI roots are required:

```toml
[dependencies]
cloud-sdk = "0.25.0"
cloud-sdk-reqwest = { version = "0.17.1", features = ["blocking-rustls-webpki-roots"] }
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
cloud-sdk = "0.25.0"
cloud-sdk-reqwest = { version = "0.17.1", features = ["blocking-rustls-fips"] }
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
cloud-sdk = "0.25.0"
cloud-sdk-reqwest = { version = "0.17.1", features = ["async-rustls"] }
```

The async adapter requires an active Tokio executor because reqwest uses Tokio
internally; the core trait and testkit remain executor-neutral. Responses are
buffered only up to caller capacity and copied after complete success. Timeout,
read failure, overflow, or cancellation leaves the caller buffer cleared.
See the complete, compile-checked
[`cloud-sdk-reqwest` async example](https://docs.rs/cloud-sdk-reqwest/latest/cloud_sdk_reqwest/#async-example)
for client construction and request execution.

## Pagination Cursor Example

```rust
use cloud_sdk::pagination::{
    PageLimit, PageMetadata, PageNumber, PaginationCursor,
};

# fn main() -> Result<(), cloud_sdk::pagination::PaginationError> {
let first = PageNumber::new(1)?;
let second = PageNumber::new(2)?;
let limit = PageLimit::new(10)?;
let mut cursor = PaginationCursor::new(first, 25, limit)?;

assert_eq!(cursor.next_page()?, first);
let metadata = PageMetadata::new(
    first,
    25,
    None,
    Some(second),
    Some(second),
    Some(30),
)?;
let boundary = cursor.observe(metadata, 25, None)?;

assert!(!boundary.is_terminal());
assert_eq!(cursor.next_page()?, second);
# Ok(())
# }
```

The caller fetches and decodes each requested page, then passes only validated
metadata and the decoded entry count to the cursor. Empty non-terminal pages,
repeated pages, contradictory navigation, entries above `per_page`, mismatch
with a supplied total, page-size or traversal-total changes, and the caller's
hard page limit fail closed. Restart the traversal when provider metadata
changes. Each accepted boundary preserves transport rate-limit metadata.

## Action Polling Example

```rust
use core::time::Duration;
use cloud_sdk::action_polling::{
    ActionPollStep, ActionPoller, ActionUpdate, PollContext, PollDecision,
    PollPolicy,
};

struct FixedDelay;

impl PollPolicy for FixedDelay {
    type Error = ();

    fn decide(&mut self, _context: PollContext) -> Result<PollDecision, Self::Error> {
        Ok(PollDecision::Delay(Duration::from_secs(2)))
    }
}

let mut poller = ActionPoller::new();
let mut policy = FixedDelay;
let running = poller.observe(
    ActionUpdate::<()>::Running,
    25,
    None,
    &mut policy,
);
assert_eq!(running, Ok(ActionPollStep::Delay(Duration::from_secs(2))));

let complete = poller.observe(
    ActionUpdate::<()>::Success,
    100,
    None,
    &mut policy,
);
assert_eq!(complete, Ok(ActionPollStep::Complete));
```

Provider failures are returned as `ActionPollStep::Failed(E)` without being
discarded. Running observations invoke caller policy, which must explicitly
choose a nonzero delay, cancellation, or timeout; the SDK owns no clock,
executor, sleep, retry count, or deadline.

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
| [`cloud-sdk`](https://crates.io/crates/cloud-sdk) | no | Provider-neutral domains and shared SDK foundation. |
| [`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner) | no | Hetzner provider APIs and provider-specific documentation. |
| [`cloud-sdk-reqwest`](https://crates.io/crates/cloud-sdk-reqwest) | no | Provider-neutral optional blocking and async reqwest/rustls transports; transport-free by default. |
| [`cloud-sdk-testkit`](https://crates.io/crates/cloud-sdk-testkit) | no | Provider-neutral blocking/async mock transport, response metadata fixtures, and adversarial corpus. |
| [`cloud-sdk-sanitization`](https://crates.io/crates/cloud-sdk-sanitization) | no | Provider-neutral volatile caller-buffer cleanup and guarded secret buffers. |

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
