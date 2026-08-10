# v0.72.0 Dependency Review

Status: release candidate; pentest and final retest passed.

v0.72 adds generated Hetzner Security client methods, tests, and documentation.
It adds no package, feature activation, build script, native component,
runtime, network stack, unsafe code, or dependency. Default provider and core
graphs remain `no_std`, transport-free, and allocation-free.

## Root Lockfile Changes

| Package | Previous | v0.72 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.71.0` | `0.72.0` | Advance the provider-neutral workspace milestone; no dependency boundary changes. |
| `ovhcloud-v2-probe` | `0.71.0` | `0.72.0` | Advance the unpublished workspace probe with the shared workspace version. |

## Boundary Decision

- `cloud-sdk-hetzner` retains its empty default feature and exact existing
  optional Serde dependency graph.
- Generated Security methods use existing operation associations, preparation
  guards, permits, authenticated transports, checked decoding, and workspaces.
- Uploaded certificate private keys create no new owned source allocation. The
  guarded prepared body retains escaped wire material; digest construction
  uses caller-owned canonical scratch that is cleared immediately after the
  existing `sha2` implementation computes the retained 32-byte identity.
- The testkit, reqwest adapter, sanitization crate, and their package versions
  do not change.
- All first-party fuzz and feature-unification path dependencies bind the exact
  v0.72 core source version.

Historical dependency reviews remain unchanged and do not describe the active
v0.72 source version.
