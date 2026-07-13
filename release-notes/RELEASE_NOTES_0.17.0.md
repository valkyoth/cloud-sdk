# cloud-sdk 0.17.0 Release Notes

Status: implementation in progress; pentest and retest required before tagging.

## Overview

`0.17.0` adds a runtime-neutral async transport contract to the no_std core,
an async deterministic testkit implementation, and an optional hardened async
reqwest/rustls adapter. No transport, allocator, TLS stack, or async runtime
enters any default provider or facade graph.

## Version Plan

- `cloud-sdk` publishes its code release as `0.17.0`.
- `cloud-sdk-reqwest` publishes its code release as `0.14.0`.
- `cloud-sdk-testkit` publishes its code release as `0.14.0`.
- `cloud-sdk-hetzner` publishes dependency-only patch `0.15.2`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.3`.
- Retired and provider-scoped helper packages remain rejected and unpublished.

## Verification

- `scripts/checks.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_rust_version_matrix.sh`
- `scripts/release_0_17_gate.sh`
- `cargo deny check`
- `cargo audit`
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
