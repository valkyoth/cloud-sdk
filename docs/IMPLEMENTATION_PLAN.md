# cloud-sdk Implementation Plan

Status: planning document

Workspace name: `cloud-sdk`

Primary crates: `cloud-sdk` and `cloud-sdk-hetzner`

1.0 target: a serious production-ready provider-neutral `cloud-sdk` foundation
and complete Hetzner Cloud, DNS, Console Storage Box, and Robot provider with
no_std request/response domains, complete claimed non-deprecated endpoint
coverage, explicit transport boundaries, safe credential handling,
pagination, action polling, rate-limit policy, high-quality tests, and
security-gated release evidence.

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
- `cloud-sdk-sanitization`: provider-neutral volatile caller-buffer cleanup,
  borrowed guards, and opt-in owned UTF-8 secret storage using the reviewed
  first-party `sanitization` crate.

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
Robot is part of the 1.0 Hetzner claim. A narrow wire-protocol source lock and
credential-free conformance fixture land in `v0.42.0` before neutral freeze
because Robot uses a different base URL, HTTP Basic Auth, repeated form fields,
errors, quotas, maintenance responses, and empty bodies. The complete active
operation inventory is source-locked separately in `v0.74.0` before Robot
operation implementation. Deprecated Robot Storage Box operations are
excluded; their supported replacement is the Console Storage Box API already
owned by `cloud-sdk-hetzner::storage`.

## API Coverage Tracks

1. Foundation: workspace, policy, endpoint group map, API matrix, source lock.
2. Core protocol: methods, paths, query parameters, labels, label selectors,
   pagination, sorting, error envelopes, rate-limit metadata, and action states.
3. Catalog/read-only resources: locations, pricing, server types, load balancer
   types, ISOs, image list/get.
4. Security resources: SSH keys and certificates with redaction and validation,
   including all non-deprecated certificate action queries.
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
10. Endpoint completion: global action lookup/list requests, certificate action
   lookup/list requests, and a zero-planned-non-deprecated matrix gate.
11. Integration evidence: mock transport, recorded fixtures, and a live-test
   harness with separate credential-free staging, privileged root sealing, and
   authenticated open-descriptor execution phases.
12. Neutral wire and isolation kernel: extensible identities, complete HTTP
   methods, endpoint-policy algebra, canonical path/query handling, bounded
   headers, response-buffer provenance, bounded lifecycle cleanup, atomic encoders,
   raw execution, authentication policies, and complete Hetzner migration in
   `v0.32.0` through `v0.43.0`.
13. Control-plane and execution contracts: pagination, quota, retry,
   idempotency, local async, streaming, incremental decoding, typed operations,
   enforceable permits, a provider-generic client kernel, pure workflow
   drivers, diagnostics, testkit scenarios, and generic drift evidence in
   `v0.44.0` through `v0.56.0`.
14. Multi-provider proof and neutral freeze: an unpublished OVHcloud API v2
   source lock and conformance probe in `v0.57.0` through `v0.61.0`, followed
   by the neutral API freeze in `v0.62.0`. The probe covers source-locked
   geographic API/token authority pairs, OAuth2 expiry and rotation,
   validation-only schema overrides, cursor headers, and task/event resources.
   Freeze also requires the narrow Robot wire fixture from `v0.42.0`; this is
   protocol evidence, not the complete Robot API inventory.
15. Complete pre-Robot Hetzner models and bindings: Cloud, actions, metrics,
   DNS, security, Console Storage Box, RFC3339, exact decimals, and exact typed
   associations in `v0.63.0` through `v0.68.0`.
16. Complete pre-Robot Hetzner clients: provider-generic client integration and
   typed Cloud, DNS, security, and Console Storage Box workflows in `v0.69.0`
   through `v0.73.0`.
17. Robot Webservice implementation: source lock, atomic forms, credentials,
   lockout, errors, quotas, every active resource family, ordering, and scoped
   billable mutations in `v0.74.0` through `v0.93.0`.
18. Robot integration and live evidence: blocking, Send-async, local-async,
   pager/action workflows, complete mock coverage, and carefully gated
   read-only live evidence in `v0.94.0` and `v0.95.0`.
19. Whole-platform qualification: adversarial/fuzz evidence, platform/MSRV/FIPS
   handshakes, provenance/governance review, controlled mutation evidence, and
   the final release candidate in `v0.96.0` through `v0.99.0`.
20. Future providers: publish focused provider crates only after the provider's
   official API source, auth model, transport expectations, threat model, API
   matrix, live-test policy, and release plan are documented. The publication
   order is Scaleway first, DigitalOcean second, and full OVHcloud later. The
   unpublished `v0.57.0-v0.61.0` OVHcloud v2 probe proves architecture only
   and is not a supported provider release.

## Post-1.0 Provider Direction

The next published provider is `cloud-sdk-scaleway`. `v1.1.0` selects a finite
product list and exact stable GA versions from
[Scaleway's APIs](https://www.scaleway.com/en/developers/api/). That inventory
must account for global, regional, and zonal authorities, `X-Auth-Token`, PATCH
requests, per-product schemas, and product-specific pagination/count
conventions such as `per_page`, `page_size`, `X-Total-Count`, and body
`total_count`. Only selected matrix rows enter the completeness claim; alpha,
beta, unselected GA versions, and unselected products remain excluded.

`cloud-sdk-digitalocean` follows Scaleway. `v1.7.0` selects a finite operation
inventory from an exact revision of DigitalOcean's official
[OpenAPI source](https://github.com/digitalocean/openapi) and validates the
simpler bearer-auth and `/v2` path together with same-authority link
pagination, optional error `request_id`, rate-limit metadata, `Retry-After`,
and the same bounded response and explicit retry rules used by other providers.
Spaces, metadata, OAuth applications, AI services, and every unselected
surface remain explicit exclusions.

`cloud-sdk-ovhcloud` is planned after those providers because a production
implementation needs a dedicated split for
[API v2](https://docs.ovhcloud.com/en/guides/manage-and-operate/api/apiv2/),
any required API v1 coverage, OAuth2 and retained legacy authentication,
geographic authorities, asynchronous task resources, billable ordering, and
OpenStack-based products. The `v0.57.0-v0.61.0` probe is deliberately too
small and too isolated to count as this provider implementation.

Each provider uses one primary crate. Shared transport, testkit, sanitization,
pagination, authentication primitives, and policy abstractions remain neutral
unless a provider demonstrates a genuinely different requirement.

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
- `v0.22.0` admits `libfuzzer-sys 0.4.13` only in an excluded, non-published
  fuzz package under pinned nightly tooling. Published and default graphs are
  unchanged.
- Rustls FIPS and `aws-lc-fips-sys` are not admitted in v0.16.0. Their exact
  module version, certificate, operating environments, build chain, provider
  configuration, runtime verification, and feature graph are a dedicated
  `v0.23.0` admission.
- `v0.23.0` admits rustls' optional FIPS mode with explicit provider and
  complete-client runtime verification. Current aws-lc-fips-sys binds a 3.0.x
  module whose active NIST certificate is not claimed by this repository.
- `v0.24.0` admits `webpki-roots 1.0.8` only through a non-default blocking
  feature with a complete explicit rustls configuration. It also records the
  direct dependency/tool freshness and AWS-LC native checksum/build review.
- `v0.28.0` changes transport receivers, endpoint identity, and credential
  lifecycle without admitting any new third-party dependency or default
  feature. Shared reqwest clients use only existing standard-library and
  admitted sanitization boundaries.
- `v0.29.0` adds allocation-free prepared-operation metadata, response policy,
  bounded execution, and testkit evidence without admitting any new
  third-party dependency or default feature.
- `v0.30.0` admits `syn 2.0.119` only in the excluded, non-published prepared
  coverage checker. It is absent from every SDK crate and published graph.
- `v0.32.0` through `v0.62.0` prefer first-party no_std contracts and excluded
  conformance tooling. Any parser, streaming, signing, provenance, or
  provider-probe dependency requires its own admission document before use.

Every admission needs a document under `docs/dependency-admission-*.md`.
