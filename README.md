<p align="center">
  <b>no_std-first multi-provider cloud SDK for Rust.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://docs.rs/cloud-sdk">Docs.rs</a>
  |
  <a href="docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="docs/threat-model.md">Threat Model</a>
  |
  <a href="SECURITY.md">Security</a>
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
filesystem, clock, or secret-storage dependency. Transport and serde support
will be admitted later behind explicit features or adapter crates.

The project target is a serious production-ready `cloud-sdk` foundation and
Hetzner provider at `1.0.0`, reached through small reviewed releases with test,
security, dependency, and release evidence. Future provider crates can follow
the same pattern, for example `cloud-sdk-cloudflare`.

## Current Status

Status: `v0.1.0` repository foundation.

Implemented now:

- Rust workspace pinned to stable `1.96.1`.
- MSRV policy for Rust `1.90.0` through `1.96.1`.
- Edition 2024 and workspace resolver `3`.
- `cloud-sdk` provider-neutral crate.
- `cloud-sdk-hetzner` provider crate with focused internal modules.
- Initial Hetzner API surface partition for Cloud, DNS, security, and Storage
  Box resources.
- Explicit placeholder crates for future Hetzner reqwest transport, testkit,
  and sanitization boundaries.
- Local checks for formatting, linting, tests, no_std policy, modularity, and
  file length.
- MIT OR Apache-2.0 license.
- Security, implementation, release, modularity, supply-chain, and threat-model
  docs.

Not implemented yet:

- No HTTP transport.
- No token storage or secret manager integration.
- No serde request/response models.
- No pagination iterator.
- No retry, rate-limit, or action polling helper.
- No generated endpoint model.
- No live Hetzner API tests.
- No non-Hetzner providers yet. `cloud-sdk-cloudflare` is a planned future
  provider pattern, not a 1.0 deliverable.
- No Robot Webservice support. Robot is planned after the Hetzner Cloud/DNS
  provider reaches `1.0.0`, likely as a `1.1.0` track exposed through
  `cloud-sdk-hetzner`.

## Trust Dashboard

| Area | Status |
| --- | --- |
| License | `MIT OR Apache-2.0` |
| MSRV | Rust `1.90.0` |
| Pinned toolchain | Rust `1.96.1` |
| Default target | `no_std` |
| Default runtime dependencies | none in `cloud-sdk`; provider crates remain transport-free by default |
| Unsafe policy | first-party crates use `#![forbid(unsafe_code)]` |
| Default features | empty |
| Network defaults | none |
| Secret storage defaults | none |
| Release evidence | local gates, dependency policy, SBOM, pentest report before tags |
| Crate versions | tracked in [`docs/CRATE_VERSION_MATRIX.md`](docs/CRATE_VERSION_MATRIX.md) |
| 1.0 target | serious production-ready provider-neutral foundation plus Hetzner provider |

## Install

```toml
[dependencies]
cloud-sdk = "0.1.0"
cloud-sdk-hetzner = "0.1.0"
```

## Workspace Crates

| Crate | Default `std`? | Purpose |
| --- | --- | --- |
| `cloud-sdk` | no | Provider-neutral domains and shared SDK foundation. |
| `cloud-sdk-hetzner` | no | Hetzner provider crate with internal `cloud`, `dns`, `security`, and `storage` modules. |
| `cloud-sdk-hetzner-reqwest` | no | Future optional reqwest transport adapter; no transport dependency admitted yet. |
| `cloud-sdk-hetzner-testkit` | no | Future mock transport, fixtures, and adversarial API response helpers. |
| `cloud-sdk-hetzner-sanitization` | no | Future optional token/secret sanitization helpers. |

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

Before any endpoint model is implemented, `v0.2.0` must verify the current
official OpenAPI/spec source and update [`docs/API_MATRIX.md`](docs/API_MATRIX.md)
with any newly discovered or changed endpoints.

## Rust Version Support

The minimum supported Rust version is Rust `1.90.0`. Development uses the
pinned stable Rust `1.96.1` until the toolchain policy is updated.

Compatibility evidence for `0.1.0`:

| Rust | Local Evidence |
| --- | --- |
| `1.90.0` | `cargo +1.90.0 check --workspace --all-features` |
| `1.91.0` | `cargo +1.91.0 check --workspace --all-features` |
| `1.92.0` | `cargo +1.92.0 check --workspace --all-features` |
| `1.93.0` | `cargo +1.93.0 check --workspace --all-features` |
| `1.94.0` | `cargo +1.94.0 check --workspace --all-features` |
| `1.95.0` | `cargo +1.95.0 check --workspace --all-features` |
| `1.96.0` | `cargo +1.96.0 check --workspace --all-features` |
| `1.96.1` | `scripts/checks.sh` |

## Checks

```bash
scripts/checks.sh
scripts/release_0_1_gate.sh
```
