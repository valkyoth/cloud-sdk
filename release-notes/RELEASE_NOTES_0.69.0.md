# cloud-sdk 0.69.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-09

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.70.0

The final CI candidate isolates RustSec advisory acquisition in a fresh
temporary database. This prevents stale deleted draft advisories in a runner
checkout from causing a false audit failure; all four lockfiles still use one
fetched database and fail closed on any audit error.

## Overview

v0.69 establishes the Hetzner client construction, endpoint-trust, complete
workspace-storage, and checked read-only execution foundation. It is an
internal milestone and publishes no crate.

## Client Foundation

- Added service-typed official Cloud, DNS, security, and Console Storage
  constructors with exact endpoint verification.
- Added conspicuous custom HTTPS constructors requiring explicit trusted-
  operator acknowledgement. Custom trust is a separate type with no execution
  methods in this milestone.
- Connected associated read-only operations to the provider-neutral client
  kernel and complete checked Hetzner success/error decoder.
- Sealed the provider-owned operation-to-service association and required
  compile-time service equality between a client and operation.
  State-changing operations remain behind plan-confirm permits.
- Preserved caller-bounded `&self` concurrency, exactly one transport attempt,
  and no client-owned executor, queue, clock, retry, or backoff policy.

## Complete Workspace Storage

- Added `EMBEDDED`, `DEFAULT`, and `LARGE` four-buffer client capacity
  profiles.
- Profile rejection clears all complete supplied regions before returning.
- Added optional fallible exact-profile owned storage under `alloc`, with full
  cleanup on drop.
- Added vertical authenticated mock evidence from typed operation through
  checked provider response plus constructor, trust, capacity, cleanup,
  default-feature, and all-feature tests.

## Security Review

- Sealed `HetznerClientOperation` so foreign operation types cannot claim a
  provider-owned service association.
- Replaced source-text client gate assertions with executable unit,
  integration, rustdoc, and feature-boundary checks.
- Constrained the custom-trust compile-fail proof fully and paired it with a
  compiling official-trust control using identical bounds.
- Kept custom endpoint execution unavailable, direct mutation behind permits,
  and endpoint-redacted diagnostics allocation-free under default features.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.69.0` | code | deferred to v0.70.0 |
| `cloud-sdk-hetzner` | `0.40.0` | code | deferred |
| `cloud-sdk-reqwest` | `0.34.0` | unchanged | no |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.0` | unchanged | no |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0690`](../docs/PUBLIC_API_REVIEW.md#v0690)
- [`docs/DEPENDENCY_REVIEW.md#v0690`](../docs/DEPENDENCY_REVIEW.md#v0690)
- [`docs/THREAT_MODEL_DELTA.md#v0690`](../docs/THREAT_MODEL_DELTA.md#v0690)
- [`docs/REJECTED_ABSTRACTIONS.md#v0690`](../docs/REJECTED_ABSTRACTIONS.md#v0690)
- [`docs/MIGRATION.md#v0690`](../docs/MIGRATION.md#v0690)
- [`security/pentest/v0.69.0.md`](../security/pentest/v0.69.0.md)

## Release Gate

Run `scripts/release_0_69_gate.sh` on the clean final evidence commit after the
incremental pentest and retest. GitHub CI and CodeQL must be green on that
unchanged commit before the signed internal tag. Do not publish crates.
