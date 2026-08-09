# v0.68.0 Dependency Review

Status: implementation complete; pentest required.

v0.68 adds no package, dependency feature, build script, native component, or
network source. The binding generator and verifier use only Python's standard
library plus existing repository scripts. Rust changes use only existing
workspace crates and `core`; no dependency edge or feature is added.

The source version of `cloud-sdk-hetzner` remains at published version 0.40.0
until v0.70.0. No crate is selected for publication.

## Root Lockfile Changes

| Package | Previous | v0.68 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.67.0` | `0.68.0` | Workspace milestone metadata; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.67.0` | `0.68.0` | Inherits workspace metadata; source and dependency graph are unchanged. |

The root, fuzz, prepared-coverage, and reqwest feature-unification dependency
graphs otherwise retain the exact versions reviewed at v0.67.
