# Dependency Review 0.94.0

Status: implementation stop; incremental pentest required.

## Result

v0.94 adds no dependency, feature, unsafe code, native build, network client,
runtime, filesystem, clock, randomness, or secret-store edge.

Robot clients compose the existing provider-neutral client kernel, bounded
workspace leases, official endpoint and Basic-auth binding, credential-attempt
state, cleanup guards, prepared operations, permit families, and strict Robot
decoders. Provider crates remain transport-free and default features remain
empty.

Full freshness, deny, RustSec, SBOM, feature-unification, package, and platform
gates remain required before tagging.

## Lockfile Changes

| Package | Previous | v0.94 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.93.0` | `0.94.0` | Advance the internal facade for complete Robot clients. |
| `ovhcloud-v2-probe` | `0.93.0` | `0.94.0` | Advance the excluded workspace probe identity only. |

External package versions, checksums, features, and sources are unchanged.

## Publication Selection

| Package | Published | v0.94 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.94.0` | code | no |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | unchanged | no |

`scripts/release_crates.py` must select no package. Fuzz, tools, isolated
tests, and the OVHcloud probe remain excluded.
