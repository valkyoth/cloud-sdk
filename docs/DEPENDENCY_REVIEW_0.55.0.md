# v0.55.0 Dependency Review

Date: 2026-08-04

Scope: cumulative public checkpoint from v0.50.0 through v0.55.0.

## Result

No new third-party package, feature, build script, native code, network stack,
runtime, allocator requirement, logger, clock, filesystem access, random
source, or serializer enters the workspace for v0.55.

The dynamic testkit implementation uses only `core`, existing `cloud-sdk`
transport contracts, caller-owned atomic slots, and borrowed fixtures. The
default testkit graph remains the same bounded `no_std` graph enforced by
`scripts/check_testkit_boundary.sh`.

## Independent Package Versions

| Package | Previous published | v0.55 source | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.55.0` | cumulative code release |
| `cloud-sdk-hetzner` | `0.38.0` | `0.39.0` | cumulative provider integration code |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.4` | dependency-only core update |
| `cloud-sdk-sanitization` | `0.17.0` | `0.18.0` | fail-closed cleanup fixture assurance |
| `cloud-sdk-testkit` | `0.28.2` | `0.29.0` | cumulative fixture and dynamic scenario code |

The publication manifest contains all five changed package trees. The
sanitization release changes test assurance only; its public API, runtime code,
dependency graph, and feature graph are unchanged.

## Root Lockfile Changes Since v0.54

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.54.0` | `0.55.0` | Workspace package version and dynamic-checkpoint documentation. |
| `cloud-sdk-hetzner` | `0.38.0` | `0.39.0` | Cumulative provider code version assignment. |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.4` | Dependency-only core range update. |
| `cloud-sdk-testkit` | `0.28.2` | `0.29.0` | Dynamic scenario implementation. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.18.0` | Workspace-wide fail-closed fixture remediation. |

## Required Verification

- default and all-feature workspace compilation and tests;
- Rust 1.92.0 through pinned Rust 1.97.1 compatibility;
- testkit exhaustion, non-consumption, cancellation, recording, script, stream,
  and injected-failure tests;
- unchanged default dependency and platform boundaries;
- package archives, README doctests, fuzz compilation, and file-length policy;
- Cargo Deny, RustSec, SBOM freshness, source-lock drift, and release metadata;
- cumulative pentest from exact v0.50.0 through the final v0.55.0 candidate;
- `scripts/release_0_55_gate.sh` after pentest evidence is committed.
