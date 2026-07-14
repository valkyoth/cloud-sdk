# cloud-sdk 0.26.0 Release Notes

Release date: 2026-07-14

## Summary

`0.26.0` completes every source-locked non-deprecated Hetzner Cloud, DNS, and
Storage Box request operation. The provider now covers 208/208 active
operations while deliberately omitting 13 deprecated operations. Default
provider and facade builds remain transport-free, allocation-free, and
`no_std`.

## Global Actions

`cloud_sdk_hetzner::actions` now exposes explicit `GET /actions` and
`GET /actions/{id}` request models. The list request requires at least one ID,
matching the upstream removal of arbitrary project-wide action listing. It
encodes repeated `id` parameters in caller order and caps each request at 128
IDs. `ActionId` now enforces the OpenAPI integer maximum of
9,007,199,254,740,991 in addition to rejecting zero.

All paths and queries write into caller-owned buffers. Undersized buffers,
empty ID lists, excessive ID lists, invalid IDs, and invalid constructed paths
fail explicitly without allocation.

## Certificate Actions

The certificate module now covers:

- `GET /certificates/actions`;
- `GET /certificates/actions/{id}`; and
- `GET /certificates/{id}/actions`.

Global and per-certificate list requests are distinct types. Only the global
request can carry repeated action-ID filters, preventing an unsupported query
combination from being attached to the certificate-local endpoint. Both list
forms support pagination and bounded repeated status and sort values. Query
ordering is deterministic: IDs, pagination, sorts, then statuses.

The deprecated resource-local
`GET /certificates/{id}/actions/{action_id}` operation remains unavailable.
Certificate retry continues through the existing non-deprecated request model.

## Complete Matrix Gate

The five formerly planned rows are marked `implemented-v0.26`. A new checked
gate parses the operations table and fails unless every non-deprecated row has
an implemented status. It also rejects missing, malformed, or structurally
changed tables. Regression fixtures prove planned and deferred active
operations cannot pass.

The current result is 208 implemented non-deprecated operations and 13
deferred deprecated operations across all 221 source-locked operations.

## Documentation And Examples

The provider README now reports complete request-operation coverage. A new
compile-checked example builds global and certificate action paths and queries
without performing network operations. The API matrix remains the canonical
operation-level coverage record.

## Independent Crate Versions

- `cloud-sdk` publishes metadata release `0.26.0`.
- `cloud-sdk-hetzner` publishes code release `0.20.0`.
- `cloud-sdk-reqwest` publishes dependency-only patch `0.17.2`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.12`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.15.8`.

No retired provider-specific helper crate is published.

## Verification

- `cargo test -p cloud-sdk-hetzner --lib actions::`
- `cargo test -p cloud-sdk-hetzner --lib security::certificates::actions::tests`
- `cargo check -p cloud-sdk-hetzner --example actions`
- `scripts/check_api_matrix_coverage.py`
- `scripts/test-api-matrix-coverage.py`
- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/check_platform_matrix.sh --all`
- `scripts/check_rust_version_matrix.sh`
- `scripts/check_fuzz_harness.sh --build`
- `scripts/check_fuzz_harness.sh --smoke`
- `scripts/check_sbom_freshness.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/release_crates.py --dry-run --yes --version 0.26.0`
- `scripts/release_0_26_gate.sh` after pentest evidence is committed.

## Security Review

Pentest is required for the exact implementation commit before release
finalization. Tagging remains blocked until the report, retest when applicable,
full release checks, GitHub CI, and CodeQL default setup are green.
