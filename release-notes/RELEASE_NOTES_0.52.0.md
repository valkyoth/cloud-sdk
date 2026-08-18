# cloud-sdk 0.52.0 Milestone Notes

Status: internal tagged development milestone.

Release date: pending

Security-Review: PASS
Pentest: DEFERRED TO v0.55.0
Publication: DEFERRED TO v0.55.0

## Overview

v0.52 adds an allocation-free provider-generic client kernel that joins typed
preparation, endpoint and authentication policy, one transport attempt,
checked success or provider-error decoding, and complete cleanup. It also adds
bounded caller-owned workspace admission for concurrent requests.

This milestone receives the complete local and GitHub release gates and a
normal signed tag. Its changes remain inside the cumulative v0.55 pentest and
crates.io publication range.

## Client Kernel

- Added one `ClientOperation` and `ClientKernel` path for blocking, Send-async,
  and local-async execution.
- Kept raw send-once helpers crate-private and denied decoder access to a
  reusable prepared request.
- Added checked success and provider-error response facades with operation
  status, media, body, request-ID, and raw-response limits.
- Preserved direct mutation, destructive, and cost-bearing denial until an
  explicit v0.51 plan-confirm permit authorizes execution.
- Added payload-free preparation, execution, decoding, pool, and lease errors.

## Workspace And Async Safety

- Added fixed-capacity atomic admission with immediate exhaustion and no hidden
  queue or allocation.
- Required one caller-owned four-buffer workspace per in-flight request.
- Kept all mutable storage uniquely borrowed across async suspension and
  cleared every complete region before releasing a cancelled lease.
- Made the Send-async kernel return an explicitly `Send` future.
- Required additive response sanitizers to be `Sync`, correcting the previous
  mismatch between the documented Send path and a retained trait object.
- Raised the workspace MSRV from Rust 1.90 to 1.92. Rust 1.90 and 1.91 reject
  the generic `Send` future because of compiler issue rust-lang/rust#100013;
  Rust 1.92 through the pinned 1.97.1 toolchain compile the same contract.

## Versions

| Crate | Source version | Publication |
| --- | --- | --- |
| `cloud-sdk` | `0.52.0` | deferred to v0.55.0 |
| `cloud-sdk-hetzner` | `0.38.0` | retained; cumulative changes deferred |
| `cloud-sdk-reqwest` | `0.32.3` | retained; dependency accumulation deferred |
| `cloud-sdk-sanitization` | `0.17.0` | unchanged |
| `cloud-sdk-testkit` | `0.28.2` | retained; cumulative changes deferred |

## Documentation

- [`docs/CLIENT_KERNEL.md`](../docs/CLIENT_KERNEL.md)
- [`docs/MIGRATION.md#v0520`](../docs/MIGRATION.md#v0520)
- [`docs/PUBLIC_API_REVIEW.md#v0520`](../docs/PUBLIC_API_REVIEW.md#v0520)
- [`docs/DEPENDENCY_REVIEW.md#v0520`](../docs/DEPENDENCY_REVIEW.md#v0520)

## Release Gate

Tag only after the clean local v0.52 gate plus GitHub CI and CodeQL pass on the
final milestone commit. No v0.52 pentest report or crates.io publication is
created under the ordinary cumulative cadence.
