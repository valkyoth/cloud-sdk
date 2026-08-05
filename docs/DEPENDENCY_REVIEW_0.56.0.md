# v0.56.0 Dependency Review

Date: 2026-08-05

Scope: provider-generic drift tooling and the release-process change after
published v0.55.0.

## Result

No third-party Rust or Python package, feature, build script, native code,
network stack, runtime, allocator requirement, logger, clock, filesystem
service, random source, or serializer enters the workspace for v0.56.

The drift engine uses the Python standard library and repository-owned shell
entry points. Remote retrieval uses the platform TLS verifier, rejects every
redirect, requires an exact credential-free HTTPS URL, and authenticates the
complete bounded response before invoking a reviewed provider adapter.

## Package Versions

| Package | Published | v0.56 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.56.0` | release-tooling metadata | No |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.0` | unchanged | No |
| `cloud-sdk-reqwest` | `0.32.4` | `0.32.4` | unchanged | No |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | No |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.0` | unchanged | No |

## Root Lockfile Changes Since v0.55

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.56.0` | Workspace facade source version only. |

## Required Verification

- canonical plugin, lock, observation, and diff fixtures;
- malformed, duplicate, oversized, noncanonical, symlink, and float rejection;
- exact-URL TLS retrieval, redirect denial, byte/time limits, and digest checks;
- compatibility with every existing Hetzner source-lock artifact;
- unchanged default dependency, feature, `no_std`, and platform boundaries;
- Cargo Deny, RustSec, SBOM freshness, documentation links, and release metadata;
- incremental pentest from signed v0.55.0 through the v0.56.0 candidate; and
- `scripts/release_0_56_gate.sh` after pentest evidence is committed.
