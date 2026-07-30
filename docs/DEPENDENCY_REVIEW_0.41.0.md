# v0.41.0 Dependency Review

Date: 2026-07-30

Scope: bearer authentication scope, rotation, refresh, and adapter-header
cleanup.

## Result

No new third-party dependency is admitted. No dependency enters a default
feature graph. The direct optional `http` dependency moves from `1.4.2` to
`1.5.0`; upstream adds the RFC 10008 `QUERY` constant and fixes empty
path/query construction and URI maximum-length enforcement. `cloud-sdk`
continues to own its finite method domain and does not automatically admit the
new method.

Core authentication uses only existing provider-neutral identity and endpoint
types. The optional reqwest adapter reuses its admitted `bytes`, reqwest,
rustls, synchronization, and `cloud-sdk-sanitization` graph.

`bytes::Bytes::from_owner` ties each authorization header to a cleanup-owning
allocation. The owner volatile-clears its complete byte allocation after the
last header clone drops. Token source and retired-token cleanup continue
through the existing sanitization boundary.

Live crates.io metadata on 2026-07-30 confirmed all selected stable direct
dependencies are current. Rustls `0.24.0-dev.1` remains a prerelease and is not
selected. `http 1.5.0` declares Rust 1.57, below the workspace MSRV.

## Boundary

- Default core, provider, reqwest, sanitization, and testkit graphs remain
  network-free and runtime-free.
- Authentication adds no clock, acquisition client, OAuth implementation,
  executor, filesystem, random source, or secret store.
- Raw executors remain credential-free.
- Blocking, async, deterministic-root, and FIPS authenticated modes share the
  same scope and generation policy.
- Unavoidable reqwest, TLS, allocator, kernel, and remote credential copies
  remain documented operational exclusions.

## Required Verification

- `scripts/check_bearer_authentication.sh`
- reqwest, deterministic-root, FIPS, default/no_std, platform, MSRV, package,
  deny, audit, and SBOM checks
- `scripts/release_0_41_gate.sh` after pentest evidence is committed
