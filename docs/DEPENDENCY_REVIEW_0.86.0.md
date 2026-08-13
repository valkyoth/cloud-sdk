# v0.86.0 Dependency Review

Status: implementation stop; pentest required.

v0.86 adds no third-party package, version, feature activation, build script,
native component, network stack, runtime, filesystem, clock, cryptography, or
unsafe code. Reverse-DNS implementation reuses the admitted protected address
model, Robot form codec, strict JSON parser, response guards, endpoint and
authentication policy, request-bound permits, and provider-neutral operation
metadata.

The source checker uses only Python's standard library. The new fuzz target
reuses the existing fuzz graph and introduces no package or feature edge.

## Lockfile Changes

| Package | Previous | v0.86 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.86.0` | Advance the internal facade source milestone. |
| `ovhcloud-v2-probe` | `0.85.0` | `0.86.0` | Advance the excluded workspace probe identity only. |

The exact local core requirement advances in the fuzz and reqwest
feature-unification lockfiles. Published provider, reqwest, sanitization, and
testkit identities remain unchanged. External package versions, checksums,
features, and sources are unchanged.

## Publication Selection

| Package | Published | v0.86 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.86.0` | code | no |
| `cloud-sdk-hetzner` | `0.44.0` | `0.44.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.2` | unchanged | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.4` | unchanged | no |

`scripts/release_crates.py` must reject publication for this internal
milestone. Fuzz, tools, isolated tests, and the OVHcloud probe remain excluded.
