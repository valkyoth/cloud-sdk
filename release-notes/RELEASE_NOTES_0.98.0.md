# cloud-sdk 0.98.0 Release Notes

Status: implementation stop; incremental pentest required.

Release date: 2026-08-19

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.100.0

## Overview

v0.98 qualifies the complete pre-1.0 source across every documented compiler,
target, public feature, package, documentation, and native-build boundary. It
is an internal cumulative tag and publishes no crate.

## Platform And Feature Evidence

- Portable CI checks default, alloc, and Serde graphs independently on Linux,
  Windows, FreeBSD, macOS, Android, iOS, WASM, and bare metal.
- Native CI checks and executes each blocking, deterministic-root, and async
  reqwest feature separately and then tests the combined graph on Linux,
  Windows, macOS ARM64, and macOS x86-64.
- Reqwest dependencies are target-qualified for Linux, Windows, macOS, and
  FreeBSD. Unsupported targets require one crate-owned diagnostic instead of
  leaking failures from networking dependencies.
- `cloud-sdk-reqwest/std` no longer propagates `cloud-sdk/std`; callers select
  core std integrations explicitly, and bare-metal diagnostics are no longer
  preempted by a missing core standard library.
- Every publishable crate is Cargo-packaged with its docs.rs feature graph,
  and workspace documentation builds with all features.

## Compiler And Supply Chain

- Locked all-target checks cover Rust 1.92.0, 1.93.0, 1.93.1, 1.94.0, 1.94.1,
  1.95.0, 1.96.0, 1.96.1, 1.97.0, and pinned stable 1.97.1.
- A structural contract binds empty default features, docs.rs all-feature
  metadata, target-qualified transport dependencies, MSRV, and toolchain pin.
- The complete all-target build-script inventory is exact. Bundled
  `aws-lc-sys 0.44.0` remains the active native crypto build, while
  target-specific `ring 0.17.14` remains reviewed.
- FIPS features, dependencies, package content, and compliance claims remain
  absent and deferred until Brynja is ready.

## Versions

| Crate | Published | v0.98 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.95.0` | `0.98.0` | deferred |
| `cloud-sdk-hetzner` | `0.46.0` | `0.46.0` | code; deferred |
| `cloud-sdk-reqwest` | `0.36.0` | `0.36.0` | code; deferred |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.31.0` | `0.31.0` | unchanged |

## Stop Gate

Run the incremental pentest for the exact implementation commit. After a green
retest, add permanent v0.98 evidence and run `scripts/release_0_98_gate.sh`.
Do not publish crates; the cumulative public checkpoint is v0.100.0.
