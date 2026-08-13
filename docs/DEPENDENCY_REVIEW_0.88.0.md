# Dependency Review 0.88.0

Status: implementation stop; pentest required.

## Result

v0.88 adds no dependency, feature, unsafe code, native build, network client,
runtime, filesystem, clock, randomness, or secret-store edge.

The Robot decoder reuses the already admitted exact `base64-ng`, `md-5`, and
`sha2` dependencies behind the existing `serde` feature. MD5 is used only to
verify Hetzner Robot's source-documented legacy fingerprint against decoded
public-key wire bytes. It is not used as an SDK security identity; the same
wire bytes produce a SHA-256 fingerprint for caller comparisons.

`cloud-sdk`, `cloud-sdk-hetzner`, `cloud-sdk-reqwest`,
`cloud-sdk-sanitization`, and `cloud-sdk-testkit` retain their existing default
feature and no-std boundaries. Full freshness, deny, RustSec, SBOM, feature
unification, package, and platform gates remain required before tagging.

## Lockfile Changes

The cumulative internal train is compared with the latest public v0.85
checkpoint.

| Package | Previous | v0.88 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.88.0` | Advance the internal facade through reverse DNS, traffic, and SSH keys. |
| `ovhcloud-v2-probe` | `0.85.0` | `0.88.0` | Advance the excluded workspace probe identity only. |

The exact local core requirement advances in the fuzz and reqwest
feature-unification lockfiles. Published provider, reqwest, sanitization, and
testkit identities remain unchanged. External package versions, checksums,
features, and sources are unchanged.

## Publication Selection

| Package | Published | v0.88 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.88.0` | code | no |
| `cloud-sdk-hetzner` | `0.44.0` | `0.44.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.2` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.4` | unchanged | no |

`scripts/release_crates.py` must select no package for this internal milestone.
Fuzz, tools, isolated tests, and the OVHcloud probe remain excluded.
