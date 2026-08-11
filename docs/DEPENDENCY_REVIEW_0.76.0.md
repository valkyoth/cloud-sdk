# v0.76.0 Dependency Review

Status: release candidate; pentest and final retest passed.

v0.76 adds no third-party package, build script, native component, network
stack, runtime, filesystem, clock, or unsafe code.

The provider-neutral attempt lifecycle uses `core::sync::atomic::AtomicU32`.
Its optional owned lineage uses `alloc::sync::Arc`; state construction creates
one allocation and beginning an attempt only increments the existing strong
count. No owner address is exposed through diagnostics or `Hash`.
`OwnedCredentialAttemptState::new` uses infallible `Arc::new`, so allocator
exhaustion may abort rather than return an SDK error. The threat model excludes
process-abort cleanup guarantees; high-availability and regulated deployments
must supply external memory limits and process supervision.
Protected Robot strings reuse the admitted `SecretString`, `SecretBuffer`, and
volatile-clear functions already supplied by `cloud-sdk-sanitization` under
the existing `alloc` feature. The provider's opt-in `alloc` feature now
explicitly activates that first-party dependency feature instead of relying on
dev-dependency feature unification. The default provider graph remains
allocation-free and transport-free.

Standalone production-mode checks cover `cloud-sdk-hetzner` with no default
features, `alloc`, and `std` before its test suite runs.

## Root Lockfile Changes

| Package | Previous | v0.76 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.76.0` | Add provider-neutral lockout-aware credential-attempt state. |
| `ovhcloud-v2-probe` | `0.75.0` | `0.76.0` | Advance the unpublished workspace probe with the shared workspace version. |

Fuzz and reqwest-feature-unification lockfiles advance only their exact local
`cloud-sdk` path identity from 0.75.0 to 0.76.0. Their external package sets
and checksums do not change.

## Independent Versions

| Package | Published | v0.76 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.76.0` | code | no |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | accumulated code | no |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged | no |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged | no |

The release plan selects no package. Cumulative publication is deferred to
v0.80.0, where changed package trees will receive independent versions.
