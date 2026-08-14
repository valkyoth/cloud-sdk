# Dependency Review 0.90.0

Status: release candidate; pentest and final retest passed.

## Result

v0.90 adds no dependency, feature, unsafe code, native build, network client,
runtime, filesystem, clock, randomness, secret-store, or cryptographic edge.

Robot vSwitch preparation reuses the existing bounded form codec, fixed-buffer
path writer, protected sanitization storage, strict JSON decoder, and
request-bound permit framework. The dedicated response fuzzer uses only the
existing fuzz workspace graph. Default features remain empty and the provider
crate remains transport-free and `no_std`.

## Lockfile Changes

The cumulative public checkpoint is compared with v0.85. External package
versions, checksums, features, and sources remain unchanged.

| Package | Previous | v0.90 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.90.0` | Publish cumulative provider-neutral source identity. |
| `cloud-sdk-hetzner` | `0.44.0` | `0.45.0` | Publish accumulated Robot reverse-DNS, traffic, SSH-key, firewall, and vSwitch code. |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.3` | Dependency-only patch requiring the v0.90 core. |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.5` | Dependency-only patch requiring the v0.90 core. |
| `ovhcloud-v2-probe` | `0.85.0` | `0.90.0` | Advance the excluded workspace probe identity only. |

Exact local core/provider requirements advance in fuzz and isolated
feature-unification lockfiles. Fuzz, tools, isolated tests, and the OVHcloud
probe remain excluded from publication.

## Publication Selection

| Package | Published | v0.90 | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.90.0` | code | yes |
| `cloud-sdk-hetzner` | `0.44.0` | `0.45.0` | code | yes |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.3` | dependency | yes |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.5` | dependency | yes |

The release tool must select exactly the two code-changing crates and their two
direct dependency-only neutral releases. Sanitization remains unselected.
