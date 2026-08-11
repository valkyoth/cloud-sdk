# v0.79.0 Dependency Review

Status: release candidate; pentest and final retest passed.

v0.79 adds no third-party package, feature activation, build script, native
component, network stack, runtime, filesystem, clock, cryptography, or unsafe
code. Cancellation values reuse the existing protected allocation boundary,
bounded Robot form codec, strict JSON parser, operation metadata, response
guard, and `core::net` parser.

The source checker uses only Python's standard library. The new fuzz target
reuses the existing fuzz graph and does not add a package or feature edge.

## Root Lockfile Changes

| Package | Previous | v0.79 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.78.0` | `0.79.0` | Advance the facade source identity only. |
| `ovhcloud-v2-probe` | `0.78.0` | `0.79.0` | Advance the excluded workspace probe with the workspace version. |

Isolated lockfiles advance only their exact local `cloud-sdk` path identity.
External package versions, checksums, features, and sources do not change.

## Independent Versions

| Package | Published | v0.79 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.79.0` | metadata | no |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | accumulated code | no |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged | no |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged | no |

The release plan selects no package. Publication remains deferred to v0.80.0.
