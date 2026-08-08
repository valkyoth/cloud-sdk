# v0.64.0 Dependency Review

Status: release candidate; pentest and final retest passed.

No dependency or feature is added, removed, or updated. Cloud special models
reuse the existing optional Serde boundary, `alloc`, and
`cloud-sdk-sanitization`. The dedicated fuzz target uses only dependencies
already admitted to the excluded non-published fuzz package.

The source version of `cloud-sdk-hetzner` remains at its published 0.39.1
package number until v0.65.0. No crate is selected for publication.

## Root Lockfile Changes

| Package | Previous | v0.64 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.63.0` | `0.64.0` | Workspace milestone metadata; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.63.0` | `0.64.0` | Inherits workspace metadata; source and dependency graph are unchanged. |
