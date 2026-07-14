# cloud-sdk Implementation Plan

Status: planning document

Workspace name: `cloud-sdk`

Primary crates: `cloud-sdk` and `cloud-sdk-hetzner`

1.0 target: a serious production-ready provider-neutral `cloud-sdk` foundation
and Hetzner Cloud/DNS provider with no_std request/response domains, complete
claimed Cloud API endpoint coverage, explicit transport boundaries, safe token
handling, pagination, action polling, rate-limit policy, high-quality tests,
and security-gated release evidence.

Post-1.0 target: Robot Webservice support exposed through
`cloud-sdk-hetzner`, likely starting at `v1.1.0`, after its separate
authentication, form-encoding, response, rate-limit, and dedicated-server
operation model is source-locked.

## Core Position

This workspace is not a generated monolith and must not hide networking, TLS,
async runtime, clocks, filesystem access, or token storage behind default
features. The provider-neutral `cloud-sdk` crate stays small. Provider crates
such as `cloud-sdk-hetzner` model safe API domains and request policy in
`no_std`. Concrete transport implementations are optional adapter crates with
explicit trust and dependency review.

## Non-Negotiable Engineering Rules

- Rust stable `1.97.0`, edition 2024, workspace resolver `3`.
- MSRV is Rust `1.90.0`; compatibility must be checked through `1.97.0`.
- Latest crate and tool versions are checked before dependency or tooling edits.
- Hetzner API behavior is implemented from current official documentation or a
  pinned official OpenAPI/spec source, never from memory.
- First-party crates are `#![no_std]` by default and do not depend on network,
  filesystem, clock, TLS, async runtime, process, or secret-storage code unless
  an adapter crate explicitly admits that dependency.
- Provider crate `cloud-sdk-hetzner` owns Hetzner endpoint models in focused
  modules.
- Third-party crates require review, current-version checks, license checks,
  feature review, and tests before admission.
- First-party crates use `#![forbid(unsafe_code)]`.
- Normal `.rs` files must stay below 500 lines.
- Security documentation, release notes, and test evidence are release
  requirements.

## Workspace Shape

- `cloud-sdk`: provider-neutral no_std SDK foundation for shared provider,
  API-family, method, and blocking/runtime-neutral async transport contracts.
- `cloud-sdk-hetzner`: Hetzner no_std provider crate. Endpoint models live under
  `src/cloud`, `src/dns`, `src/security`, and `src/storage`, with shared
  request, response, pagination, label, rate-limit, and action domains in
  top-level source files.
- `cloud-sdk-reqwest`: provider-neutral no_std boundary by default, with
  reviewed blocking and async reqwest/rustls transports behind non-default
  features. Provider crates never depend on it directly.
- `cloud-sdk-testkit`: provider-neutral no_std ordered mock transport, bounded
  response fixtures, pagination/action/rate-limit metadata, and adversarial
  response corpus. Future releases may add live-test gating helpers.
- `cloud-sdk-sanitization`: provider-neutral volatile caller-buffer cleanup and
  guarded secret buffers using the reviewed first-party `sanitization` crate.

Future providers add one `cloud-sdk-{provider}` crate. Provider API families
stay as internal modules, while reusable transports, test infrastructure,
serialization, and secret handling extend the provider-neutral boundaries.
Release automation rejects nested provider packages.

## Source Discipline

The official API reference is <https://docs.hetzner.cloud/reference/cloud>.
Before implementing endpoint behavior, create a pinned source record in
`docs/SPEC_LOCK.md` and update `docs/API_MATRIX.md` with the exact endpoint,
method, path, request, response, error, pagination, sorting, and action
semantics being claimed.

If current docs or the OpenAPI source disagree with the plan, implementation
stops until the difference is documented and versioned.

The Robot Webservice source is <https://robot.hetzner.com/doc/webservice/en.html>.
It is intentionally not part of the 1.0 Cloud/DNS claim. A later Robot track
must source-lock the Robot docs separately before adding server, IP, subnet,
reset, failover, boot, reverse DNS, traffic, SSH key, ordering, storage box,
firewall, and vSwitch operations.

## API Coverage Tracks

1. Foundation: workspace, policy, endpoint group map, API matrix, source lock.
2. Core protocol: methods, paths, query parameters, labels, label selectors,
   pagination, sorting, error envelopes, rate-limit metadata, and action states.
3. Catalog/read-only resources: locations, pricing, server types, load balancer
   types, ISOs, image list/get.
4. Security resources: SSH keys and certificates with redaction and validation.
5. Storage Box source lock: verify exact current groups and operations from the
   official source because this area was discovered from the 2026 changelog.
6. Compute resources: servers, server metrics, server actions, images,
   placement groups, primary IPs.
7. Storage/IP/network resources: volumes, floating IPs, firewalls, networks,
   load balancers, metrics, and actions.
8. DNS resources: zones, zonefiles, zone actions, RRSets, and RRSet actions.
9. Optional transport: request builder, auth injection, response parsing,
   pagination streams, action polling, retry/rate-limit policy, and a dedicated
   fail-closed FIPS blocking mode in `v0.23.0`.
10. Integration evidence: mock transport, recorded fixtures, and a live-test
   harness with separate credential-free staging, privileged root sealing, and
   authenticated open-descriptor execution phases.
11. 1.0 hardening: docs, examples, fuzzing, mutation/adversarial tests, SBOM,
   pentest, dependency audit, and platform matrix.
12. Post-1.0 Robot Webservice: separate source lock, Basic Auth policy,
   form-encoded request model, Robot-specific errors/rate limits, and
   dedicated-server operation modules exposed through the SDK.
13. Future providers: add provider crates such as `cloud-sdk-cloudflare` only
   after the provider's official API source, auth model, transport expectations,
   and test strategy are documented.

## Dependency Admission Plan

No third-party runtime dependency is admitted in `v0.1.0`.

Expected future candidates must be reviewed before use:

- `serde` admitted in `v0.14.0` with `default-features = false` behind the
  provider's optional no_std model boundary.
- `serde_json` admitted only as a dev dependency for `v0.14.0` JSON fixtures;
  production parser use still requires a transport-specific review.
- HTTP/TLS/client crates only in transport adapter crates, never in the main
  SDK default graph.
- `sanitization` admitted in `v0.14.0` with default features disabled through
  the provider-neutral `cloud-sdk-sanitization` boundary.
- `reqwest` `0.13.4` admitted in `v0.16.0` with default features disabled only
  through `cloud-sdk-reqwest/blocking-rustls`, extended to `async-rustls` in
  `v0.17.0`; the facade and provider default graphs remain transport-free.
- `bytes` `1.12.1` admitted in `v0.17.0` only for sanitized owned async request
  bodies; Tokio remains absent from core/testkit and is caller-supplied at the
  concrete async adapter execution boundary.
- `v0.18.0` adds pagination and action-polling state machines without new
  third-party dependencies, clocks, runtimes, automatic requests, or sleeps.
- `v0.19.0` uses the already admitted provider, reqwest, Serde, JSON, and
  sanitization dependencies only in an ignored integration test. No new
  third-party dependency or default-graph feature is admitted.
- `v0.20.0` adds compile and dependency-graph evidence for portable and native
  platform claims without admitting a new crate or enabling a default feature.
- `v0.21.0` adds compile-checked examples, docs.rs metadata, and documentation
  validation without admitting a new crate or enabling a default feature.
- Rustls FIPS and `aws-lc-fips-sys` are not admitted in v0.16.0. Their exact
  module version, certificate, operating environments, build chain, provider
  configuration, runtime verification, and feature graph are a dedicated
  `v0.23.0` admission.

Every admission needs a document under `docs/dependency-admission-*.md`.
