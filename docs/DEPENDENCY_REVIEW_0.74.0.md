# v0.74.0 Dependency Review

Status: release candidate; pentest and final retest passed.

v0.74 adds a repository-only JSON source lock, Python standard-library
validation, tests, and documentation. It adds no package, crate dependency,
feature activation, build script, native component, runtime, network stack,
unsafe code, or published file.

## Root Lockfile Changes

| Package | Previous | v0.74 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.73.0` | `0.74.0` | Advance the provider-neutral workspace milestone; no dependency boundary changes. |
| `ovhcloud-v2-probe` | `0.73.0` | `0.74.0` | Advance the unpublished workspace probe with the shared workspace version. |

## Boundary Decision

- The Robot lock remains under `tests/fixtures` and cannot enter a package.
- Fetch validation uses only Python's standard library, rejects redirects,
  enforces HTTPS identity and byte bounds, and never executes fetched bytes.
- `cloud-sdk-hetzner`, reqwest, sanitization, and testkit manifests do not
  change.
- First-party fuzz and feature-unification path dependencies bind the exact
  v0.74 core source version without changing their external dependency graphs.

Historical dependency reviews remain unchanged and do not describe the active
v0.74 source version.
