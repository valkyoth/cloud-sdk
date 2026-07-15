# cloud-sdk 0.30.0 Release Notes

Status: implementation complete; pentest and final release checks remain
required before tagging.

Release date: 2026-07-15

## Overview

`0.30.0` completes allocation-free prepared requests for all 208
source-locked non-deprecated Hetzner Cloud, DNS, and Console Storage
operations. Provider request models now produce a complete method, target,
bounded JSON body, official endpoint identity, operation metadata, and
response policy without selecting a transport or runtime.

The default workspace remains `no_std`, transport-free, runtime-free, and free
of new third-party dependencies.

## Prepared Hetzner Operations

- Endpoint adapters cover all 208 active source-locked operations and reject
  the 13 deprecated operations retained only for drift monitoring.
- Body adapters cover all 91 active operations that declare an OpenAPI request
  body, including compute, networking, firewall, load balancer, DNS, security,
  and Console Storage families.
- List filters, pagination, sorting, metrics queries, zonefile imports,
  sensitive values, reusable action bodies, and empty-body operations retain
  their existing validated request models.
- Official Cloud and Console Storage service identities are fixed to HTTPS,
  port 443, `/v1`, and their exact provider host.

## Security Properties

- Preparation clears complete caller-owned target and body buffers before use
  and again after any path, query, body, pairing, or capacity failure.
- Endpoint, query, and body components carry source-locked operation keys;
  mismatched combinations fail before request bytes are admitted.
- Request metadata distinguishes read-only, mutation, destructive, retry, and
  cost-bearing behavior without permissive defaults.
- Complete DNS and firewall replacement, credential reset, protection-change,
  and other state-removal operations require destructive approval. Enabling
  billed server backups requires cost approval.
- Sensitive user data, private keys, TSIG material, Storage Box passwords,
  zonefiles, and record values use controlled JSON-string writers.
- Response policies bind exact success status, content type, body presence,
  and an 8 MiB maximum for JSON responses; no-content responses forbid body
  and content type.

## Coverage Evidence

- `docs/PREPARED_BODY_OPERATIONS.txt` locks the 91 active upstream operations
  that declare request bodies.
- `scripts/check_prepared_operation_coverage.py` derives operation keys only
  from Rust items parsed by an isolated, locked `syn` checker. The endpoint
  macro accepts only explicit pattern-to-string-literal mappings. Adapter
  macros must be unqualified top-level items, while manual implementations
  must use canonical `crate::prepared` trait paths. All five module-scope macro
  definitions are structurally source-locked in their reviewed roots, and
  module-scope invocations are allowlisted. The complete canonical module chain
  from `lib.rs` through `prepared.rs` is checked before evidence files receive
  an exact unconditional `mod name;` declaration and regular `name.rs` file
  pairing. Counted macros, implementations, methods, and match arms must be
  unattributed. Constants, nested comments, raw strings, file/item `cfg`,
  `cfg_attr`, parent-edge substitutions, orphaned or redirected modules,
  procedural erasure, namespaced or generated-shadow adapters, duplicate or
  modified definitions, inline fake traits, discarded/helper expressions,
  unknown keys, ambiguous mappings, missing adapters, and deprecated evidence
  are rejected.
- Mutation tests prove those structural checks and malformed duplicate body
  locks fail closed. The module/file bijection and normal Rust checks together
  prove the admitted source participates in the compiled provider crate.
- Golden tests cover exact firewall, load-balancer, DNS, and Console Storage
  requests, destructive/cost metadata, mismatch rejection, and insufficient
  buffer cleanup.

## Independent Crate Versions

- `cloud-sdk` publishes metadata release `0.30.0`.
- `cloud-sdk-hetzner` publishes code release `0.23.0`.
- `cloud-sdk-reqwest` publishes dependency-only patch `0.20.1`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.16`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.18.1`.

No retired provider-specific helper crate is present in the publish plan.

## Migration

See [`docs/MIGRATION_0.30.0.md`](../docs/MIGRATION_0.30.0.md) for prepared
operation construction, buffer ownership, reusable action components, and the
explicit firewall apply/remove intent.

## Security Review

The v0.30 pentest is required at the implementation stop. Its permanent report
will be added only after the review and any required retest pass.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test --workspace --all-features`
- `scripts/checks.sh`
- `scripts/check_prepared_operation_coverage.py`
- `scripts/test-prepared-operation-coverage.py`
- `scripts/release_crates.py --check`
- `scripts/check_sbom_freshness.sh`
- `scripts/release_0_30_gate.sh`
- `scripts/release_crates.py --dry-run --yes --version 0.30.0`

## Release Gate

```text
v0.30.0 implementation stop reached. Run the pentest for this exact commit.
Tag only after the permanent PASS report, clean local release gate, and GitHub
CI pass for the final release commit.
```
