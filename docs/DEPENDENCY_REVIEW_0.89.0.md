# Dependency Review 0.89.0

Status: release candidate; pentest and final retest passed.

## Result

v0.89 adds no dependency, feature, unsafe code, native build, network client,
runtime, filesystem, clock, randomness, secret-store, or cryptographic edge.

Robot firewall preparation reuses the existing allocation-backed form codec,
fixed-buffer path writers, sanitization boundary, strict JSON decoder, and
request-bound execution permits. The new decoder fuzzer uses the existing fuzz
workspace dependency graph.

`cloud-sdk`, `cloud-sdk-hetzner`, `cloud-sdk-reqwest`,
`cloud-sdk-sanitization`, and `cloud-sdk-testkit` retain their existing default
feature and no-std boundaries. Full freshness, deny, RustSec, SBOM, feature
unification, package, and platform gates remain required before tagging.

## Lockfile Changes

The cumulative internal train is compared with the latest public v0.85
checkpoint.

| Package | Previous | v0.89 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.89.0` | Advance the internal facade through reverse DNS, traffic, SSH keys, and firewalls. |
| `ovhcloud-v2-probe` | `0.85.0` | `0.89.0` | Advance the excluded workspace probe identity only. |

The exact local core requirement advances in the fuzz and reqwest
feature-unification lockfiles. The fuzz lock also records the new firewall
target as workspace source evidence. Published provider, reqwest,
sanitization, and testkit identities remain unchanged. External package
versions, checksums, features, and sources are unchanged.

## Publication Selection

| Package | Published | v0.89 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.89.0` | code | no |
| `cloud-sdk-hetzner` | `0.44.0` | `0.44.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.2` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.4` | unchanged | no |

`scripts/release_crates.py` must select no package for this internal milestone.
Fuzz, tools, isolated tests, and the OVHcloud probe remain excluded.
