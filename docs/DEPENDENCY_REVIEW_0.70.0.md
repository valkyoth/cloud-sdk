# v0.70.0 Dependency Review

Status: implementation complete; pentest required.

v0.70 adds no third-party package, feature activation, build script, native
component, runtime, network stack, unsafe code, or normal provider dependency.
The existing optional SHA-256 boundary updates from `sha2 0.10.9` to current
`0.11.0`; Cloud methods otherwise reuse the existing core client, operation,
permit, checked decoder, optional Serde, reqwest development adapter, and
testkit boundaries.

## Independent Versions

| Package | Previous published | v0.70 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.70.0` | cumulative code | yes |
| `cloud-sdk-hetzner` | `0.40.0` | `0.41.0` | cumulative code | yes |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.1` | dependency-only | yes |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.1` | dependency-only | yes |

The unpublished OVHcloud v2 probe inherits workspace version `0.70.0` but is
excluded from the publishable package set. Publication order is core, reqwest,
testkit, then the Hetzner provider after each dependency is visible on
crates.io. Sanitization is not republished.

## Root Lockfile Changes

| Package | Previous | v0.70 | Review |
| --- | --- | --- | --- |
| `block-buffer` | `0.10.4` | `-` | Removed with the old digest stack. |
| `cloud-sdk` | `0.69.0` | `0.70.0` | Public facade checkpoint. |
| `cloud-sdk-hetzner` | `0.40.0` | `0.41.0` | Cumulative provider code release. |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.1` | Internal core requirement only; transport code unchanged. |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.1` | Internal core requirement only; testkit code unchanged. |
| `cpufeatures` | `0.2.17` | `-` | Removed older duplicate; the graph retains current `0.3.0`. |
| `crypto-common` | `0.1.7` | `-` | Removed with the old digest stack. |
| `digest` | `0.10.7` | `-` | Removed older duplicate; `md-5` and `sha2` use the current digest API. |
| `generic-array` | `0.14.7` | `-` | Removed with the old digest stack. |
| `ovhcloud-v2-probe` | `0.69.0` | `0.70.0` | Unpublished workspace metadata inheritance. |
| `sha2` | `0.10.9` | `0.11.0` | Current compatible release of the existing optional SHA-256 implementation. |
| `version_check` | `0.9.5` | `-` | Removed with `generic-array 0.14`. |

The newer AWS-LC `1.18.0/0.44.0/0.14.1` set remains deliberately rejected by
the documented v0.65 clean-source and FIPS qualification decision. Its
read-only Cargo source build still requires a fresh full admission review
before those exact pins can move.

## Feature Boundaries

- Default `cloud-sdk` and `cloud-sdk-hetzner` features remain empty.
- Named Cloud execution remains behind existing `cloud-sdk-hetzner/serde`.
- No transport dependency enters the provider's normal graph.
- The neutral reqwest and testkit patch releases change only their required
  first-party core version.
