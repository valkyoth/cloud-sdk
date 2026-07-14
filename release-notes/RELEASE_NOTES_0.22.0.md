# cloud-sdk 0.22.0 Release Notes

Status: implementation stop reached; pentest required.

## Overview

`0.22.0` adds isolated coverage-guided fuzzing and deterministic adversarial
regressions around the SDK's allocation-free builders, validators, state
machines, and bounded response decoders. The fuzz toolchain is not part of any
published crate or supported stable Rust dependency graph.

No provider operation, transport behavior, default feature, or production
dependency is added by this release.

## Fuzz Coverage

Six libFuzzer targets cover:

- fixed-buffer decimal, percent, and atomic JSON writers;
- origin-form paths, query validation, ordering, and encoding;
- labels, selectors, DNS names, endpoint paths, and record JSON;
- pagination metadata, traversal locks, bounds, and non-mutation;
- action progress, polling policy, terminal states, and non-mutation;
- bounded action, error, and pagination response envelopes.

Named committed seeds are synthetic valid and invalid inputs derived from
source-locked examples and policy boundaries. Smoke runs copy those inputs to
temporary writable corpora. Generated corpus entries and crash artifacts are
ignored and rejected if tracked.

The harness uses `nightly-2026-07-13`, `cargo-fuzz 0.13.2`, and
`libfuzzer-sys 0.4.13`. It lives in the excluded, non-published `fuzz/` package
with an independent lockfile, Cargo Deny/RustSec checks, and SPDX SBOM.
Because cargo-sbom omits build and development dependencies, the repository
completes all three SPDX documents from locked Cargo metadata and independently
rejects any missing, duplicate, ambiguous, unexpected, or stale package set.

## Deterministic Regressions

- Every undersized output capacity is checked for atomic JSON-string failure
  without partial writes or length mutation.
- Malformed UTF-8, trailing input, wrong envelope shapes, duplicate metadata,
  numeric overflow, oversized fields, and forbidden control characters fail
  closed.
- Deep additive unknown fields are ignored safely without changing decoded
  action state; bounded shallow unknown fields remain forward compatible.

The [fuzzing guide](../docs/FUZZING.md) documents installation, bounded CI
replay, longer campaigns, exact crash replay, minimization, secret handling,
and conversion of confirmed defects into owning-crate regression tests.

## Version Plan

- `cloud-sdk` publishes code/release-evidence version `0.22.0`.
- `cloud-sdk-hetzner` publishes response-test code version `0.19.0`.
- `cloud-sdk-reqwest` publishes dependency-only patch `0.15.4`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.8`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.15.4`.
- Retired and provider-scoped helper packages remain rejected and unpublished.

## Verification

- `scripts/checks.sh`
- `scripts/check_fuzz_harness.sh --build`
- `scripts/check_fuzz_harness.sh --smoke`
- `cargo test --workspace --all-features`
- `scripts/check_platform_matrix.sh --all`
- `scripts/check_rust_version_matrix.sh`
- `scripts/check_hetzner_upstream.sh --local-only`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_sbom_freshness.sh`
- `scripts/release_crates.py --dry-run --yes --version 0.22.0`
- workspace, downstream fixture, and fuzz Cargo Deny checks;
- workspace, downstream fixture, and fuzz RustSec audits;
- `scripts/release_0_22_gate.sh` after pentest evidence is committed.

## Pentest Stop

Pentest the exact committed implementation state. A no-findings result is valid
evidence and does not require a redundant retest. Findings require remediation,
deterministic regression coverage, and another pentest pass before tagging.
