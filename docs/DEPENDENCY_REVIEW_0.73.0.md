# v0.73.0 Dependency Review

Status: release candidate; pentest passed with no findings.

v0.73 adds generated Hetzner Console Storage client methods, tests, and
documentation. It adds no package, feature activation, build script, native
component, runtime, network stack, unsafe code, or dependency. Default
provider and core graphs remain `no_std`, transport-free, and allocation-free.

## Root Lockfile Changes

| Package | Previous | v0.73 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.72.0` | `0.73.0` | Advance the provider-neutral workspace milestone; no dependency boundary changes. |
| `ovhcloud-v2-probe` | `0.72.0` | `0.73.0` | Advance the unpublished workspace probe with the shared workspace version. |

## Boundary Decision

- `cloud-sdk-hetzner` retains its empty default feature and exact existing
  optional Serde dependency graph.
- Generated Storage methods use existing operation associations, preparation
  guards, permits, authenticated transports, checked incremental decoding, and
  workspaces.
- Password-bearing create and reset requests use the existing sensitive-body
  classification, `Sha256PlanHasher`, and complete-buffer sanitization.
- The testkit, reqwest adapter, sanitization crate, and their package versions
  do not change.
- All first-party fuzz and feature-unification path dependencies bind the exact
  v0.73 core source version.

Historical dependency reviews remain unchanged and do not describe the active
v0.73 source version.
