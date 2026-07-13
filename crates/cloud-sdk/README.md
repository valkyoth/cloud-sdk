<p align="center">
  <b>no_std-first multi-provider cloud SDK for Rust.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">Crates.io</a>
  |
  <a href="https://docs.rs/cloud-sdk">Docs.rs</a>
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

# cloud-sdk

`cloud-sdk` is a `no_std`-first Rust workspace for cloud provider SDKs. The
first provider crate is `cloud-sdk-hetzner`, covering the Hetzner Cloud and DNS
APIs. The default crates have no network client, TLS stack, async runtime,
filesystem, clock, or secret-storage dependency. Transport and serialization
remain explicit boundaries: v0.15 defines a no-network contract and testkit,
v0.16 adds an opt-in blocking rustls adapter, v0.17 adds an executor-neutral
async contract and optional Tokio-backed adapter, and v0.14 added narrowly
reviewed no_std Serde and caller-buffer sanitization. v0.18 adds explicit
pagination and action polling, while v0.19 adds an ignored opt-in live smoke
harness. None changes the default provider graph.

The project target is a serious production-ready `cloud-sdk` foundation and
Hetzner provider at `1.0.0`, reached through small reviewed releases with test,
security, dependency, and release evidence. Future provider crates can follow
the same pattern, especially focused cloud and SaaS providers such as
`cloud-sdk-scaleway` or `cloud-sdk-ovh`.

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

Status: `v0.19.0` live smoke harness implementation stop reached;
pentest required.
The latest published release is `v0.18.0`.

Implemented now:

- Rust workspace pinned to stable `1.97.0`.
- MSRV policy for Rust `1.90.0` through `1.97.0`.
- Edition 2024 and workspace resolver `3`.
- `cloud-sdk` provider-neutral crate.
- `cloud-sdk-hetzner` provider crate with focused internal modules.
- Initial Hetzner API surface partition for Cloud, DNS, security, and Storage
  Box resources.
- Provider-neutral blocking and runtime-neutral async transport contracts, plus
  a no_std deterministic mock testkit implementing both contracts.
- Provider-neutral bounded pagination and action polling state machines with
  caller-owned page fetching, parsing, delay, timeout, and cancellation policy.
- Validated rate-limit metadata propagation through blocking, async, and mock
  transports without hidden retries.
- Optional hardened provider-neutral blocking and async reqwest/rustls
  transports, plus admitted guarded caller-buffer sanitization.
- Opt-in read-only Hetzner catalog smoke harness with fixed provider origin,
  a root-sealed build-before-credential workflow, private token-file input,
  bounded responses, redacted diagnostics, and twelve offline security-policy
  tests; live network execution remains ignored by default.
- Local checks for formatting, linting, tests, no_std policy, modularity, and
  file length.
- MIT OR Apache-2.0 license.
- Security, implementation, release, modularity, supply-chain, and threat-model
  docs.
- Official Hetzner Cloud/DNS and Storage Box spec source lock for `v0.2.0`.
- Complete source-derived API matrix with 221 operations, owner modules,
  pagination, sorting, action behavior, deprecation status, and implementation
  status.
- Local upstream lock validation for the pinned Hetzner spec URLs and hashes.
- Hetzner API drift detection for added, removed, and changed operations and
  component schemas.
- Core Hetzner request/response policy domains for endpoint paths, base URL
  selection, endpoint group base mapping, bounded query parameters,
  fixed-buffer percent encoding, labels, pagination, sorting, action status,
  API errors, and rate-limit metadata.
- Read-only Hetzner catalog request primitives for locations, pricing, server
  types, load balancer types, ISOs, and public images.
- Hetzner security request primitives for SSH key CRUD, certificate CRUD, and
  certificate retry action endpoints.
- Hetzner server request primitives for server CRUD, metrics, and server action
  endpoint paths.
- Hetzner server-adjacent request primitives for images, placement groups,
  primary IPs, and their v0.7 action paths.
- Hetzner storage/IP request primitives for volumes, floating IPs, and their
  v0.8 action paths.
- Hetzner Storage Box request primitives for boxes, box types, snapshots,
  subaccounts, Storage Box actions, and subaccount actions.
- Hetzner Firewall request primitives for CRUD, resource application, and
  validated rule replacement.
- Hetzner Network request primitives for CRUD, routes, subnets, range changes,
  and protection actions.
- Hetzner Load Balancer request primitives for CRUD, metrics, services,
  targets, network attachment, reverse DNS, protection, algorithms, type
  changes, and public-interface actions.
- Hetzner DNS Zone request primitives for CRUD, zonefile import/export,
  primary nameservers, TTL and protection actions, and action listing.
- Hetzner DNS RRSet request primitives for CRUD, list filtering, TTL and
  protection actions, and bounded record mutations.
- Optional no_std Serde boundary for size-checked RRSet request bodies and
  validated shared action/error response envelopes.

Not implemented yet:

- No provider-level client that executes typed Hetzner request models end to
  end; v0.17 exposes reviewed provider-neutral blocking and async transports.
- No token storage or secret manager integration.
- No broad request/response serialization outside the reviewed RRSet and
  shared response boundary.
- No automatic pagination stream; the explicit cursor requires callers to
  fetch and decode each page.
- No automatic retries or sleeps; the action poller delegates delay, timeout,
  and cancellation decisions to caller policy.
- No generated response model.
- No destructive live Hetzner tests; v0.19 includes only explicitly enabled
  read-only catalog probes.
- No non-Hetzner providers yet. Smaller focused cloud and SaaS providers such
  as Scaleway and OVH are better future fits than hyperscaler-scale APIs, but
  no non-Hetzner provider is a 1.0 deliverable.
- No Robot Webservice support. Robot is planned after the Hetzner Cloud/DNS
  provider reaches `1.0.0`, likely as a `1.1.0` track exposed through
  `cloud-sdk-hetzner`.

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
| Crate versions | tracked in [`docs/CRATE_VERSION_MATRIX.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATE_VERSION_MATRIX.md) |
| 1.0 target | serious production-ready provider-neutral foundation plus Hetzner provider |

## Provider Roadmap

| Provider | Target Version | Crate |
| --- | --- | --- |
| [`Hetzner Cloud`](https://www.hetzner.com/) | 1.0.0 | [`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner) |
| [`Hetzner Robot`](https://www.hetzner.com/) | 1.1.0 | planned in `cloud-sdk-hetzner` |

## Install

```toml
[dependencies]
cloud-sdk = "0.19.0"
cloud-sdk-hetzner = "0.17.0"
```

## Provider-Neutral Example

```rust
use cloud_sdk::{ApiFamily, Method, Provider};

let provider = Provider::Hetzner;
let family = ApiFamily::Cloud;
let method = Method::Get;

assert_eq!(provider, Provider::Hetzner);
assert_eq!(family, ApiFamily::Cloud);
assert_eq!(method, Method::Get);
```

## Transport Contract Example

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{RequestTarget, TransportRequest};

let Ok(target) = RequestTarget::new("/servers?page=1") else {
    return;
};
let request = TransportRequest::new(Method::Get, target);

assert_eq!(request.method(), Method::Get);
assert_eq!(request.target().as_str(), "/servers?page=1");
assert!(request.body().is_empty());
```

The core contracts perform no I/O and select no executor. Use
`cloud-sdk-testkit` for deterministic blocking or async tests, or opt into
`cloud-sdk-reqwest/blocking-rustls` or `async-rustls` for HTTPS.

## Optional Blocking Transport

```toml
[dependencies]
cloud-sdk = "0.19.0"
cloud-sdk-reqwest = { version = "0.15.1", features = ["blocking-rustls"] }
```

```rust,ignore
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, RequestTarget, TransportRequest};
use cloud_sdk_reqwest::blocking::{
    BearerToken, BlockingClientBuilder, HttpsEndpoint, RequestTimeouts,
    UserAgent,
};

let Ok(endpoint) = HttpsEndpoint::new("https://api.hetzner.cloud/v1") else { return };
let Ok(token) = BearerToken::new("replace-with-scoped-token") else { return };
let Ok(user_agent) = UserAgent::new("my-service/1.0") else { return };
let Ok(timeouts) = RequestTimeouts::new(
    Duration::from_secs(30),
    Duration::from_secs(10),
) else { return };
let Ok(mut client) = BlockingClientBuilder::new(endpoint, token, user_agent, timeouts).build()
else { return };

let Ok(target) = RequestTarget::new("/servers?page=1") else { return };
let request = TransportRequest::new(Method::Get, target);
let mut response_body = [0_u8; 65_536];
let Ok(response) = client.send(request, &mut response_body) else { return };

assert!(response.status().is_success());
```

The production builder is HTTPS-only, requires explicit bounded timeouts and a
user agent, uses rustls with TLS 1.2 minimum, and disables redirects, retries,
proxies, referer generation, and response decompression. It forces HTTP/1 and
the system resolver even if another dependency enables reqwest HTTP/2 or
Hickory DNS. The caller owns token generation, scope, rotation, revocation,
and cleanup of the original secret; the adapter clears only its own token and
request-body storage.

## Opt-In Hetzner Live Smoke Test

The repository includes an ignored read-only harness for locations, server
types, load balancer types, ISOs, public system images, and pricing. It accepts
only a private token-file path, fixes the authenticated origin to Hetzner Cloud
API v1, bounds and clears response storage, and never logs response bodies or
resource IDs.

```sh
scripts/smoke_hetzner_live.sh --check
```

Authenticated execution requires a dedicated test project, a provider token
with **Read** permission, and a root-sealed bundle prepared from a clean commit
before the token exists or is mounted. An administrator must install the staged
bundle and launcher into root-owned, non-writable system paths; authenticated
execution then hashes and executes the same open file descriptor without
invoking Cargo. The token value does not belong in a command or environment
variable. Follow
[`docs/LIVE_SMOKE_TESTING.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/LIVE_SMOKE_TESTING.md)
for private token-file setup and the manual command. Destructive execution is
not implemented in v0.19.

## Optional Async Transport

```toml
[dependencies]
cloud-sdk = "0.19.0"
cloud-sdk-reqwest = { version = "0.15.1", features = ["async-rustls"] }
```

```rust,ignore
use cloud_sdk::Method;
use cloud_sdk::transport::{AsyncTransport, RequestTarget, TransportRequest};

let Ok(target) = RequestTarget::new("/servers?page=1") else { return };
let request = TransportRequest::new(Method::Get, target);
let mut response_body = [0_u8; 65_536];
let Ok(response) = AsyncTransport::send(&mut client, request, &mut response_body).await
else { return };

assert!(response.status().is_success());
```

The async adapter requires an active Tokio executor because reqwest uses Tokio
internally; the core trait and testkit remain executor-neutral. Responses are
buffered only up to caller capacity and copied after complete success. Timeout,
read failure, overflow, or cancellation leaves the caller buffer cleared.

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
| [`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner) | no | Main Hetzner documentation and provider crate with internal `cloud`, `dns`, `security`, and `storage` modules. |
| [`cloud-sdk-reqwest`](https://crates.io/crates/cloud-sdk-reqwest) | no | Provider-neutral optional blocking and async reqwest/rustls transports; transport-free by default. |
| [`cloud-sdk-testkit`](https://crates.io/crates/cloud-sdk-testkit) | no | Provider-neutral blocking/async mock transport, response metadata fixtures, and adversarial corpus. |
| [`cloud-sdk-sanitization`](https://crates.io/crates/cloud-sdk-sanitization) | no | Provider-neutral volatile caller-buffer cleanup and guarded secret buffers. |

The workspace uses one primary crate per provider. Provider-specific API
families remain modules inside that crate; reusable transport, testkit,
serialization, and sanitization boundaries remain provider-neutral. Package
names with another scoped suffix, such as `cloud-sdk-ovh-reqwest` or
`cloud-sdk-scaleway-dns`, are rejected by release automation.

The root README documents the workspace and release process. Crate-local README
files document the crate-specific role and examples. For Hetzner-specific usage,
start with [`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner).

Hetzner endpoint modules live inside the provider crate:

```text
crates/cloud-sdk-hetzner/src/
  actions.rs
  endpoint.rs
  labels.rs
  pagination.rs
  rate_limit.rs
  request.rs
  response.rs
  cloud/catalog.rs
  cloud/
  dns/
  security/
  storage/
```

## API Scope

The first planning pass covers the Hetzner Cloud API reference at
<https://docs.hetzner.cloud/reference/cloud>, including overview material for
authentication, query parameters, errors, actions, labels, pagination, rate
limiting, server metadata, sorting, and deprecation notices.

Robot Webservice support is intentionally post-1.0. It uses a different API
shape, authentication model, and request encoding than the Cloud/DNS API. The
planned direction is to expose it through `cloud-sdk-hetzner` after 1.0,
without letting Robot-specific behavior weaken the default Cloud SDK design.

Endpoint groups scheduled for the SDK:

| Area | Groups |
| --- | --- |
| Cross-resource | actions |
| Servers | servers, server actions, server types, images, image actions, ISOs, placement groups, primary IPs, primary IP actions |
| Storage | volumes, volume actions |
| IPs | floating IPs, floating IP actions |
| Network edge | firewalls, firewall actions, load balancers, load balancer actions, load balancer types, networks, network actions |
| DNS | zones, zone actions, zone RRSets, zone RRSet actions |
| Security | certificates, certificate actions, SSH keys |
| Storage Boxes | storage boxes, storage box actions, storage box subaccounts |
| Catalog and billing | locations, pricing |

The `v0.2.0` planning pass source-locked the official machine-readable specs:

- Cloud and DNS: <https://docs.hetzner.cloud/cloud.spec.json>
- Storage Boxes: <https://docs.hetzner.cloud/hetzner.spec.json>

[`docs/API_MATRIX.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_MATRIX.md) tracks all 221 discovered operations.
Deprecated operations remain listed for drift tracking, but are marked
`deferred-deprecated` until the SDK has an explicit compatibility policy.

Before changing endpoint models, run:

```bash
scripts/check_hetzner_api_drift.py --fetch
```

That compares the current upstream specs with the locked operation and schema
fingerprints in
[`docs/API_FINGERPRINTS.tsv`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_FINGERPRINTS.tsv)
and
[`docs/API_SCHEMA_FINGERPRINTS.tsv`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/API_SCHEMA_FINGERPRINTS.tsv).

Do not refresh lock files directly from a drift report. First review the
upstream changes and update the pinned spec hashes in the source-lock evidence.
Then refresh the fingerprints during the reviewed source-lock pass:

```bash
scripts/check_hetzner_api_drift.py --fetch --write-lock --accept-lock-refresh
```

The write path verifies fetched spec bytes against the pinned SHA-256 values
before overwriting the fingerprint files.

## Rust Version Support

The minimum supported Rust version is Rust `1.90.0`. Development uses the
pinned stable Rust `1.97.0` until the toolchain policy is updated.

Compatibility verification matrix for current main:

| Rust | Local Evidence |
| --- | --- |
| `1.90.0` | `cargo +1.90.0 check --workspace --all-features` |
| `1.91.0` | `cargo +1.91.0 check --workspace --all-features` |
| `1.92.0` | `cargo +1.92.0 check --workspace --all-features` |
| `1.93.0` | `cargo +1.93.0 check --workspace --all-features` |
| `1.94.0` | `cargo +1.94.0 check --workspace --all-features` |
| `1.95.0` | `cargo +1.95.0 check --workspace --all-features` |
| `1.96.0` | `cargo +1.96.0 check --workspace --all-features` |
| `1.96.1` | `cargo +1.96.1 check --workspace --all-features` |
| `1.97.0` | `scripts/checks.sh` |

## Checks

```bash
scripts/checks.sh
scripts/release_0_1_gate.sh
scripts/check_hetzner_upstream.sh --local-only
scripts/check_hetzner_api_drift.py --fetch
scripts/release_0_2_gate.sh
scripts/release_0_3_gate.sh
scripts/release_0_4_gate.sh
scripts/release_0_5_gate.sh
scripts/release_0_6_gate.sh
scripts/release_0_7_gate.sh
scripts/release_0_8_gate.sh
scripts/release_0_9_gate.sh
scripts/release_0_10_gate.sh
scripts/release_0_11_gate.sh
scripts/release_0_12_gate.sh
scripts/release_0_13_gate.sh
scripts/release_0_14_gate.sh
```
