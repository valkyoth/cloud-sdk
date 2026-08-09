# v0.67.0 Dependency Review

Status: implementation stop; incremental pentest required.

v0.67 adds no third-party package and changes no dependency feature. The
Console response models reuse the existing `serde_json` strict tree,
incremental JSON admission, `cloud-sdk-sanitization`, and provider model
helpers already admitted behind `cloud-sdk-hetzner/serde`.

The source version of `cloud-sdk-hetzner` remains at its published 0.40.0
package number until v0.70.0. No crate is selected for publication.

## Root Lockfile Changes

| Package | Previous | v0.67 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.66.0` | `0.67.0` | Workspace milestone metadata; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.66.0` | `0.67.0` | Inherits workspace metadata; source and dependency graph are unchanged. |

The root, fuzz, and reqwest feature-unification lockfiles retain the exact
dependency versions reviewed at v0.66. The provider default graph remains
allocation-free, transport-free, and `no_std`; the optional Serde graph does
not acquire a network client, TLS stack, runtime, filesystem, or clock.
