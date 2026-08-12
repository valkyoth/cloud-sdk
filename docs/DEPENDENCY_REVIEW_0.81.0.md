# v0.81.0 Dependency Review

Status: implementation stop; pentest required.

v0.81 adds no third-party package, feature activation, build script, native
component, network stack, runtime, filesystem, clock, cryptography, unsafe
code, or new supply-chain edge. Subnet operations reuse existing protected
storage, bounded Robot forms, strict JSON parsing, permits, and core IP types.

The source checker uses only Python's standard library. The response fuzz
target reuses the existing fuzz dependency graph.

## Root Lockfile Changes

| Package | Previous | v0.81 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.81.0` | Advance the facade source identity only. |
| `ovhcloud-v2-probe` | `0.80.0` | `0.81.0` | Advance the excluded workspace probe with the workspace version. |

The fuzz and isolated test lockfiles advance only the exact local `cloud-sdk`
path identity from 0.80.0 to 0.81.0. External package versions, checksums,
features, and sources do not change.

## Independent Versions

| Package | Published | v0.81 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.81.0` | source identity | no |
| `cloud-sdk-hetzner` | `0.43.0` | `0.43.0` | accumulated code | no |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.1` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.3` | unchanged | no |

The release plan selects no package. Publication remains deferred to v0.85.0.
