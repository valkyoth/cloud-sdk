# v0.77.0 Dependency Review

Status: implementation complete; pentest required.

v0.77 adds no third-party package, feature activation, build script, native
component, network stack, runtime, filesystem, clock, or unsafe code.

The Robot decoder reuses the existing opt-in `serde` graph, strict JSON parser,
`ResponseDecodeWorkspace`, provider-neutral quota types, and protected
`SensitiveText` storage. The new fuzz target uses only dependencies already in
the isolated fuzz workspace. Default provider features remain empty and the
default graph remains allocation-free and transport-free.

## Root Lockfile Changes

| Package | Previous | v0.77 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.76.0` | `0.77.0` | Advance the provider-neutral facade source identity without API changes. |
| `ovhcloud-v2-probe` | `0.76.0` | `0.77.0` | Advance the unpublished workspace probe with the shared workspace version. |

Fuzz and reqwest-feature-unification lockfiles advance only their exact local
`cloud-sdk` path identity from 0.76.0 to 0.77.0. Their external package sets
and checksums do not change.

## Independent Versions

| Package | Published | v0.77 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.77.0` | metadata | no |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | accumulated code | no |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged | no |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged | no |

The release plan selects no package. Cumulative publication is deferred to
v0.80.0, where changed package trees receive independent versions.
