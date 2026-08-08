# v0.65.0 Dependency Review

Status: implementation complete; incremental pentest required.

The v0.65 DNS implementation adds no dependency or feature. It reuses the
existing optional Serde parser, `alloc`, and first-party protected string
boundary. TSIG response validation is implemented without a new Base64 edge.

Across the cumulative v0.61-v0.65 publication window, `base64-ng` changed from
the exact admitted `1.3.9` release to exact `2.0.1` in the optional reqwest
Basic-auth feature graph. That first-party dependency remains disabled by
default and is covered by the reqwest boundary, feature-unification, audit,
deny, fuzz, and package gates. Other third-party versions and default feature
graphs are unchanged from v0.60.0.

## Independent Versions

| Package | Previous published | v0.65 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.65.0` | cumulative code | yes |
| `cloud-sdk-hetzner` | `0.39.1` | `0.40.0` | cumulative code | yes |
| `cloud-sdk-reqwest` | `0.33.0` | `0.34.0` | cumulative code | yes |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.1` | `0.30.0` | cumulative code | yes |

The unpublished OVHcloud v2 probe inherits workspace metadata but is excluded
from the publishable package set. Publication order is core, reqwest, testkit,
then the Hetzner provider after each dependency is visible on crates.io.

## Root Lockfile Changes

| Package | Previous | v0.65 | Review |
| --- | --- | --- | --- |
| `base64-ng` | `1.3.9` | `2.0.1` | Exact first-party update in optional reqwest Basic-auth graphs; separately reviewed and tested. |
| `ovhcloud-v2-probe` | `-` | `0.65.0` | Unpublished workspace conformance harness; excluded from crates.io publication. |
