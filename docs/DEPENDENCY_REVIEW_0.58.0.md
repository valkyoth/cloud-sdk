# v0.58.0 Dependency Review

Date: 2026-08-06

Scope: provider-neutral OVHcloud authority and OAuth conformance after
published v0.55.0.

## Result

No third-party Rust or Python package, feature, build script, native code,
network stack, runtime, allocator requirement, logger, clock, filesystem
service, random source, or serializer enters the workspace for v0.58.

The new core contracts are allocation-free `no_std` data types. Expiring
credential storage reuses the admitted reqwest adapter, standard-library
synchronization, and existing first-party sanitization boundary. The
source-binding checker uses only the Python standard library and already
reviewed provider-drift lock.

## Package Versions

| Package | Published | v0.58 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.58.0` | code | No |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.0` | dependency | No |
| `cloud-sdk-reqwest` | `0.32.4` | `0.32.4` | code | No |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | No |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.0` | dependency | No |

## Root Lockfile Changes Since v0.55

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.58.0` | Workspace facade source version only. |

## Required Verification

- exact source-bound regional authority and OAuth response-shape fixtures;
- alias, cross-region, duplicate, redirect, and downgrade rejection;
- refresh-window, rollback, expiry, overflow, stale-race, and cleanup tests;
- blocking and async atomic rotation parity;
- unchanged default dependency, feature, `no_std`, and platform boundaries;
- Cargo Deny, RustSec, SBOM freshness, documentation links, and release metadata;
- incremental pentest from signed v0.57.0 through the v0.58.0 candidate; and
- `scripts/release_0_58_gate.sh` after pentest evidence is committed.
