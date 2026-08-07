# v0.62.0 Dependency Review

Status: implementation stop pending pentest.

No third-party dependency, feature, or version is added. The source-complete
models reuse `alloc`, the existing protected `SecretString` boundary, and the
existing optional Serde feature. Incremental Storage Box admission reuses the
provider's bounded parser.

The source versions of `cloud-sdk-hetzner` and `cloud-sdk-testkit` retain their
published numbers until v0.65.0. Neither crate is selected for publication in
`release-crates.toml`; the nonpublishable OVHcloud harness remains excluded.

Cargo lockfile changes are limited to advancing workspace-owned `cloud-sdk`
and packages inheriting the workspace milestone from 0.61.0 to 0.62.0.

## Root Lockfile Changes

| Package | Previous | v0.62 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.61.0` | `0.62.0` | Workspace milestone and neutral-freeze documentation; no third-party edge changed. |
| `ovhcloud-v2-probe` | `0.61.0` | `0.62.0` | Inherits the workspace milestone; source and dependency graph are unchanged. |
