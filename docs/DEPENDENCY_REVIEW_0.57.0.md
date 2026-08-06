# v0.57.0 Dependency Review

Date: 2026-08-06

Scope: unpublished OVHcloud API v2 source-lock probe after published v0.55.0.

## Result

No third-party Rust or Python package, feature, build script, native code,
network stack, runtime, allocator requirement, logger, clock, filesystem
service, random source, or serializer enters the workspace for v0.57.

The excluded probe uses the Python standard library and the provider-neutral
drift engine admitted in v0.56. Remote retrieval is credential-free, resolves
once, connects only to validated global addresses with original-host TLS
verification, caps unique destinations at eight under one connection deadline,
restores the normal HTTP I/O timeout after TLS, and remains exact-URL,
redirect-denying, and bounded.

## Package Versions

| Package | Published | v0.57 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.57.0` | release-tooling metadata | No |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.0` | unchanged | No |
| `cloud-sdk-reqwest` | `0.32.4` | `0.32.4` | unchanged | No |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | No |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.0` | unchanged | No |

## Root Lockfile Changes Since v0.55

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.57.0` | Workspace facade source version only. |

## Required Verification

- exact source URL, digest, byte-length, authority, and schema fingerprints;
- strict duplicate-rejecting adapter and malformed-source regressions;
- exact eight-operation inventory and production/read-only enforcement;
- authority, OAuth, schema, cursor, task, and event source reproducibility;
- Cargo metadata, release-plan, publish-order, and manifest exclusions;
- unchanged default dependency, feature, `no_std`, and platform boundaries;
- Cargo Deny, RustSec, SBOM freshness, documentation links, and release metadata;
- incremental pentest from signed v0.56.0 through the v0.57.0 candidate; and
- `scripts/release_0_57_gate.sh` after pentest evidence is committed.
