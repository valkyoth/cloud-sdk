# v0.69.0 Dependency Review

Status: release candidate; pentest and final retest passed.

v0.69 adds no package, dependency, feature activation, build script, native
component, runtime, network stack, or unsafe code. The new client facade uses
existing `cloud-sdk`, `cloud-sdk-sanitization`, optional Serde, and testkit
boundaries.

## Package Changes

| Package | Previous | v0.69 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.68.0` | `0.69.0` | Adds allocation-free profiles and optional fallible owned workspace storage. |
| `ovhcloud-v2-probe` | `0.68.0` | `0.69.0` | Inherits workspace metadata; source and dependency graph are unchanged. |
| `cloud-sdk-hetzner` | `0.40.0` | `0.40.0` | Accumulates service-typed client code for the v0.70 public checkpoint. |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.0` | Unchanged. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | Unchanged. |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.0` | Unchanged; used only for deterministic client tests. |

## Feature Boundaries

- Default `cloud-sdk` and `cloud-sdk-hetzner` features remain empty.
- `OwnedClientWorkspace` is available only under existing `cloud-sdk/alloc`.
- Checked Hetzner execution is available only under existing
  `cloud-sdk-hetzner/serde`.
- No transport dependency enters the Hetzner normal dependency graph.

The release gate reruns latest-version checks, Cargo Deny, RustSec, feature-
graph checks, package checks, SBOM freshness, and all supported Rust versions.
