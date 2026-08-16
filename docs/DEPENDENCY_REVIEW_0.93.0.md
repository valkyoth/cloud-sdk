# Dependency Review 0.93.0

Status: implementation stop; incremental pentest required.

## Result

v0.93 adds no dependency, feature, unsafe code, native build, network client,
runtime, filesystem, clock, randomness, or secret-store edge.

Billable order preparation reuses caller-owned guarded buffers, the existing
Robot form encoder, official endpoint and Basic-auth scopes, strict response
decoders, provider-neutral cost permits, delivery classification, and the
existing SHA-256 plan hasher. No production code performs a network request by
itself, and CI/live smoke contain no billable route.

All publishable crates retain empty default features and their existing
`no_std` boundaries. Full freshness, deny, RustSec, SBOM, feature-unification,
package, and platform gates remain required before tagging.

## Lockfile Changes

| Package | Previous | v0.93 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.92.0` | `0.93.0` | Advance the internal facade for guarded Robot orders. |
| `ovhcloud-v2-probe` | `0.92.0` | `0.93.0` | Advance the excluded workspace probe identity only. |

External package versions, checksums, features, and sources are unchanged.

## Publication Selection

| Package | Published | v0.93 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.93.0` | code | no |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | unchanged | no |

`scripts/release_crates.py` must select no package. Fuzz, tools, isolated
tests, and the OVHcloud probe remain excluded.
