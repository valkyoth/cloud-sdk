# v0.80.0 Dependency Review

Status: release candidate; pentest and final retest passed.

v0.80 adds no third-party package, feature activation, build script, native
component, network stack, runtime, filesystem, clock, cryptography, or unsafe
code. Robot IP implementation reuses `core::net`, the admitted protected
storage and form codec, strict JSON parser, operation metadata, response guard,
and provider-neutral permit boundary.

The source checker uses only Python's standard library. The fuzz target reuses
the existing fuzz graph and does not add a package or feature edge. External
package versions, checksums, features, and sources are unchanged.

## Root Lockfile Changes

| Package | Previous | v0.80 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.79.0` | `0.80.0` | Publish the cumulative neutral checkpoint. |
| `cloud-sdk-hetzner` | `0.42.0` | `0.43.0` | Publish accumulated Robot implementation, including IP management. |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.1` | Internal dependency-only patch. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.19.0` | Publish accumulated protected fixed-byte ownership. |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.3` | Internal dependency-only patch. |
| `ovhcloud-v2-probe` | `0.79.0` | `0.80.0` | Advance the excluded workspace probe identity only. |

The fuzz lock advances the exact local core, Hetzner, reqwest, and sanitization
versions. The reqwest feature-unification lock advances its exact local core,
reqwest, and sanitization versions. No external lock entry changes.

## Publication Selection

| Package | Published | v0.80 | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.80.0` | code | yes |
| `cloud-sdk-hetzner` | `0.42.0` | `0.43.0` | code | yes |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.1` | dependency | yes |
| `cloud-sdk-sanitization` | `0.18.0` | `0.19.0` | code | yes |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.3` | dependency | yes |

`scripts/release_crates.py` computes dependency order and excludes retired
provider-specific boundary crates, non-published fuzz/tools packages, and the
OVHcloud probe.
