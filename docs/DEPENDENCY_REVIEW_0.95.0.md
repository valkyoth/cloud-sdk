# Dependency Review 0.95.0

Status: implementation stop; pentest required.

## Result

v0.95 adds no third-party dependency, feature, unsafe code, native build,
network client, runtime, filesystem, clock, randomness, or secret-store edge.
The live harness is test-only and reuses the already admitted optional
reqwest/rustls Basic transport, first-party sanitization boundary, client
workspace, and strict Robot decoder.

The optional non-FIPS graph remains exactly `aws-lc-rs 1.18.0`,
`aws-lc-sys 0.44.0`, and `http-body-util 0.1.5`. FIPS packages and features
remain absent and deferred to Brynja. All ordinary first-party crate defaults
remain empty, and `cloud-sdk-hetzner` has no transport dependency.

## Lockfile Changes

| Package | Previous | v0.95 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.94.0` | `0.95.0` | Advance the internal facade for the public Robot checkpoint. |
| `ovhcloud-v2-probe` | `0.94.0` | `0.95.0` | Advance the excluded workspace probe identity only. |

## Workspace Version Changes

| Package | Published | v0.95 | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.95.0` | cumulative core code | yes |
| `cloud-sdk-hetzner` | `0.45.0` | `0.46.0` | cumulative Robot code and live evidence | yes |
| `cloud-sdk-reqwest` | `0.35.3` | `0.36.0` | accumulated transport code and dependency updates | yes |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.5` | `0.31.0` | accumulated regression code | yes |

The release tool must publish exactly the four selected crates in dependency
order and must exclude sanitization, fuzzing, internal tools, isolated tests,
the OVHcloud probe, and retired provider-specific helper crates. Cargo Deny,
RustSec, package, feature-unification, platform, freshness, and complete SPDX
SBOM gates remain mandatory before publication.
