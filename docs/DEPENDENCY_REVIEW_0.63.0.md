# v0.63.0 Dependency Review

Status: release candidate; pentest and final retest passed.

No dependency or feature is added. Source-derived Cloud models reuse the
existing optional Serde boundary, `alloc`, and protected JSON parser. The
generator and drift integration use only Python's standard library.

The optional `base64-ng` dependency used by `cloud-sdk-reqwest` Basic
authentication is updated from exact version 1.3.9 to 2.0.1 with default
features disabled. The lockfile changes only its version and checksum: no
transitive package enters any graph. The existing bounded caller-buffer API is
source-compatible, and transport feature-boundary checks continue to require
the exact reviewed version.

The source version of `cloud-sdk-hetzner` remains at its published 0.39.1
package number until v0.65.0. No crate is selected for publication.

## Root Lockfile Changes

| Package | Previous | v0.63 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.62.0` | `0.63.0` | Workspace milestone metadata; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.62.0` | `0.63.0` | Inherits the workspace milestone; source and dependency graph are unchanged. |
| `base64-ng` | `1.3.9` | `2.0.1` | Exact optional Basic-auth encoder update; default features disabled and no transitive dependency change. |
