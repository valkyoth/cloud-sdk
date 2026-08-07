# v0.59.0 Dependency Review

Date: 2026-08-07

Scope: provider-neutral OVHcloud cursor and schema-header conformance after
published v0.55.0.

## Result

No third-party Rust or Python package, feature, build script, native code,
network stack, runtime, allocator requirement, logger, clock, filesystem
service, random source, or serializer enters the workspace for v0.59.

The new core types reuse `cloud-sdk-sanitization`, bounded response headers,
opaque cursor storage, and fixed-buffer decimal writers already admitted by
the workspace. Source-binding and regression checks use only the Python
standard library and the existing normalized provider-drift model.

## Package Versions

| Package | Published | v0.59 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.59.0` | code | No |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.0` | dependency | No |
| `cloud-sdk-reqwest` | `0.32.4` | `0.32.4` | dependency | No |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | No |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.0` | dependency | No |

## Root Lockfile Changes Since v0.55

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.59.0` | Workspace facade source version only. |
| `regex-automata` | `0.4.16` | `0.4.18` | Compatible transitive regex engine maintenance update. |

## Required Verification

- exact source-bound pagination and schema-header fixtures;
- absent-next termination, duplicate, control, non-ASCII, oversize, and
  sensitivity rejection;
- exact cursor cycle, cleanup, and request round-trip tests;
- canonical schema parsing, reviewed-major drift, and scratch cleanup tests;
- unchanged default dependency, feature, `no_std`, and platform boundaries;
- Cargo Deny, RustSec, SBOM freshness, documentation links, and release metadata;
- incremental pentest from signed v0.58.0 through the v0.59.0 candidate; and
- `scripts/release_0_59_gate.sh` after pentest evidence is committed.
