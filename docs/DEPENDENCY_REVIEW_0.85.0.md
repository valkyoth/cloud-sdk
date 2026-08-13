# v0.85.0 Dependency Review

Status: implementation stop; pentest required.

v0.85 adds no third-party package, version, feature activation, build script,
native component, network stack, runtime, filesystem, clock, cryptography, or
unsafe code. Boot implementation reuses the admitted protected storage, Robot
form codec, strict JSON parser, endpoint/authentication policy, response
guards, and provider-neutral operation metadata.

The source checker uses only Python's standard library. The new fuzz target
reuses the existing fuzz graph and introduces no package or feature edge.

## Root Lockfile Changes

| Package | Previous | v0.85 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.84.0` | `0.85.0` | Publish the cumulative facade checkpoint. |
| `cloud-sdk-hetzner` | `0.43.0` | `0.44.0` | Publish accumulated Robot provider code. |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.2` | Internal core dependency-only patch. |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.4` | Internal core dependency-only patch. |
| `ovhcloud-v2-probe` | `0.84.0` | `0.85.0` | Advance the excluded workspace probe identity only. |

The fuzz lock advances only exact local core and Hetzner identities. The
reqwest feature-unification lock advances only the exact local core identity.
External package versions, checksums, features, and sources are unchanged.

## Publication Selection

| Package | Published | v0.85 | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.85.0` | code | yes |
| `cloud-sdk-hetzner` | `0.43.0` | `0.44.0` | code | yes |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.2` | dependency | yes |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.4` | dependency | yes |

`scripts/release_crates.py` must publish the four selected crates in dependency
order while leaving sanitization unselected. Fuzz, tools, isolated tests, and
the OVHcloud probe remain excluded.
