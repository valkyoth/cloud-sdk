# v0.84.0 Dependency Review

Status: implementation stop; pentest required.

v0.84 adds no third-party package, version, build script, native component,
network stack, runtime, filesystem, unsafe code, or lockfile source. The WOL
slice reuses protected Robot identities, strict JSON parsing, endpoint and
authentication policy, response guards, execution permits, and the admitted
collision-resistant plan hasher.

The source checker uses only Python's standard library. No fuzz dependency is
added in this narrow milestone.

## Root Lockfile Changes

| Package | Previous | v0.84 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.83.0` | `0.84.0` | Advance the facade source identity only. |
| `ovhcloud-v2-probe` | `0.83.0` | `0.84.0` | Advance the excluded workspace probe with the workspace version. |

The fuzz and isolated test lockfiles advance only the exact local `cloud-sdk`
path identity from 0.83.0 to 0.84.0. External package versions, checksums,
features, and sources do not change.

## Independent Versions

| Package | Published | v0.84 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.84.0` | source identity | no |
| `cloud-sdk-hetzner` | `0.43.0` | `0.43.0` | accumulated code | no |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.1` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.3` | unchanged | no |

The release plan selects no package. Publication remains deferred to v0.85.0.
