# v0.63.0 Dependency Review

Status: implementation review complete; pentest required.

No third-party dependency, feature, or version is added. Source-derived Cloud
models reuse the existing optional Serde boundary, `alloc`, and protected JSON
parser. The generator and drift integration use only Python's standard library.

The source version of `cloud-sdk-hetzner` remains at its published 0.39.1
package number until v0.65.0. No crate is selected for publication.

## Root Lockfile Changes

| Package | Previous | v0.63 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.62.0` | `0.63.0` | Workspace milestone metadata; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.62.0` | `0.63.0` | Inherits the workspace milestone; source and dependency graph are unchanged. |
