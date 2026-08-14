# Dependency Review 0.91.0

Status: release candidate; pentest and final retest passed.

## Result

v0.91 adds no dependency, feature, unsafe code, native build, network client,
runtime, filesystem, clock, randomness, secret-store, or cryptographic edge.

Robot ordering preparation reuses the existing fixed-buffer path/query
writers. Exact decimals and provider text reuse the admitted first-party
sanitization boundary. Strict responses reuse the existing bounded JSON
decoder and checked-response association. The new fuzzer uses the existing
fuzz workspace dependency graph.

All publishable crates retain empty default features and their existing
`no_std` boundaries. Full freshness, deny, RustSec, SBOM, feature-unification,
package, and platform gates remain required before tagging.

## Lockfile Changes

The v0.91 internal train starts from the v0.90 public checkpoint.

| Package | Previous | v0.91 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.91.0` | Advance the internal facade for Robot ordering catalogs. |
| `ovhcloud-v2-probe` | `0.90.0` | `0.91.0` | Advance the excluded workspace probe identity only. |

The exact local core requirement advances in the fuzz and reqwest feature-
unification lockfiles. The fuzz lock also records the new ordering response
target as workspace source evidence. External package versions, checksums,
features, and sources are unchanged.

## Publication Selection

| Package | Published | v0.91 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.91.0` | code | no |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | unchanged | no |

`scripts/release_crates.py` must select no package for this internal milestone.
Fuzz, tools, isolated tests, and the OVHcloud probe remain excluded.
