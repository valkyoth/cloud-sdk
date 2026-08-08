# v0.66.0 Dependency Review

Status: implementation stop reached; pentest required.

No dependency or feature is added, removed, or updated. Security response
models reuse the existing optional Serde boundary, `alloc`,
`cloud-sdk-sanitization`, and request-side key validators. The excluded fuzz
package uses only previously admitted dependencies.

The source version of `cloud-sdk-hetzner` remains at its published 0.40.0
package number until v0.70.0. No crate is selected for publication.

## Root Lockfile Changes

| Package | Previous | v0.66 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.66.0` | Workspace milestone metadata; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.65.0` | `0.66.0` | Inherits workspace metadata; source and dependency graph are unchanged. |
