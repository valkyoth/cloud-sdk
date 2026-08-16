# Dependency Review 0.92.0

Status: implementation stop; incremental pentest required.

## Result

v0.92 adds no dependency, feature, unsafe code, native build, network client,
runtime, filesystem, clock, randomness, secret-store, or cryptographic edge.

Transaction preparation reuses existing fixed-buffer path encoding, endpoint
identity, authentication scope, and response policies. Protected response
values reuse `cloud-sdk-sanitization`; strict models reuse the bounded JSON
decoder. The new fuzz target uses the existing isolated fuzz graph.

All publishable crates retain empty default features and existing `no_std`
boundaries. Full freshness, deny, RustSec, SBOM, feature-unification, package,
and platform gates remain required before tagging.

## Lockfile Changes

| Package | Previous | v0.92 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.91.0` | `0.92.0` | Advance the internal facade for Robot transactions. |
| `ovhcloud-v2-probe` | `0.91.0` | `0.92.0` | Advance the excluded workspace probe identity only. |

The exact local core requirement advances in the fuzz and reqwest feature-
unification lockfiles. External package versions, checksums, features, and
sources are unchanged.

## Publication Selection

| Package | Published | v0.92 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.92.0` | code | no |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | unchanged | no |

`scripts/release_crates.py` must select no package. Fuzz, tools, isolated
tests, and the OVHcloud probe remain excluded.
