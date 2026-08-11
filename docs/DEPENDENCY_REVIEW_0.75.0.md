# v0.75.0 Dependency Review

Status: implementation stop reached; pentest required.

v0.75 adds no third-party package, feature activation, build script, native
component, runtime, network stack, allocator requirement, unsafe code, or
normal provider dependency. The form codec reuses the existing
`cloud-sdk-sanitization` volatile-clear boundary and transactional core
snapshot encoder.

## Independent Versions

| Package | Previous published | v0.75 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.75.0` | cumulative code | yes |
| `cloud-sdk-hetzner` | `0.41.0` | `0.42.0` | cumulative code | yes |
| `cloud-sdk-reqwest` | `0.34.1` | `0.35.0` | cumulative code | yes |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.2` | dependency-only | yes |

The unpublished OVHcloud v2 probe inherits workspace version `0.75.0` but is
excluded from the publisher. Publication order is core, reqwest, testkit, then
Hetzner. Sanitization is not republished.

## Root Lockfile Changes

| Package | Previous | v0.75 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.74.0` | `0.75.0` | Advance the provider-neutral public checkpoint. |
| `ovhcloud-v2-probe` | `0.74.0` | `0.75.0` | Advance the unpublished workspace probe with the shared workspace version. |

Independent provider, adapter, and testkit package versions also advance as
listed above. Their path dependency identities are represented by those
publication rows rather than a new external source or checksum.

## Cumulative Transport Decision

`cloud-sdk-reqwest 0.35.0` publishes the v0.71 removal of the experimental
AWS-LC FIPS feature and implementation. Ordinary rustls adapters remain
optional and unchanged in behavior. Future FIPS integration is deferred until
Brynja satisfies the separately documented module and environment
qualification requirements.

## Fuzz Boundary

The excluded fuzz package adds one target and one synthetic seed. Nightly Rust,
libFuzzer, generated corpora, and fuzz artifacts remain outside every
publishable package and supported stable dependency graph.
