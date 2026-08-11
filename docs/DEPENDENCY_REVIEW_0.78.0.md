# v0.78.0 Dependency Review

Status: implementation complete; pentest required.

v0.78 adds no third-party package, feature activation, build script, native
component, network stack, runtime, filesystem, clock, or unsafe code.

Robot request preparation reuses the existing provider-neutral operation,
transport, buffer, and sanitization boundaries. The neutral sanitization crate
now re-exports the already admitted `sanitization::SecretBoxBytes` type under
`alloc`; it adds no package or feature edge. Owned server models use that
stable, clear-on-drop allocation plus the existing opt-in strict JSON parser
and protected string storage under `serde`. The committed source-contract
checker uses only the Python standard library and repository fixtures.

## Root Lockfile Changes

| Package | Previous | v0.78 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.77.0` | `0.78.0` | Advance the facade source identity and add reusable form media-type constants. |
| `ovhcloud-v2-probe` | `0.77.0` | `0.78.0` | Advance the unpublished workspace probe with the workspace version. |

Isolated lockfiles advance only their exact local `cloud-sdk` path identity.
External package versions, checksums, features, and sources do not change.

## Independent Versions

| Package | Published | v0.78 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.78.0` | metadata/code | no |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | accumulated code | no |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged | no |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | accumulated code | no |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged | no |

The release plan selects no package. Publication remains deferred to v0.80.0.
