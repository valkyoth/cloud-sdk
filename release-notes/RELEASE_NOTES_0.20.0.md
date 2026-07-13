# cloud-sdk 0.20.0 Release Notes

Status: implementation stop reached; pentest and retest required before tagging.

## Overview

`0.20.0` makes platform support explicit and machine-checked. Portable crates
are compiled in no_std and alloc/Serde configurations for representative
Linux, Windows, FreeBSD, macOS, Android, iOS, WebAssembly, and bare-metal Rust
targets. Optional reqwest/rustls transports are checked only on native Linux,
Windows, macOS ARM64, and macOS x86-64 runners.

No provider API, transport behavior, default feature, or third-party dependency
is added by this release.

## Portable Matrix

The allowlisted target checks cover:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `x86_64-unknown-freebsd`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`
- `aarch64-linux-android`
- `aarch64-apple-ios`
- `wasm32-unknown-unknown`
- `thumbv7em-none-eabihf`

Each target checks `cloud-sdk`, `cloud-sdk-hetzner`,
`cloud-sdk-sanitization`, and `cloud-sdk-testkit` with no default features. A
second pass checks the allocation-bearing core/testkit features and Hetzner's
optional no_std Serde boundary.

## Native Transport Matrix

The full workspace, all targets, and all features compile natively on:

- `ubuntu-latest`
- `windows-latest`
- `macos-15` for ARM64
- `macos-15-intel` for x86-64

This includes both `cloud-sdk-reqwest/blocking-rustls` and `async-rustls`.
FreeBSD transport remains best effort. Android, iOS, WASM, and bare-metal
targets require a platform-native implementation of the provider-neutral
transport traits.

## Security Boundaries

- The target argument is accepted only from a fixed allowlist.
- Missing Rust target libraries and unavailable or failing rustup commands fail
  with distinct diagnostics before Cargo execution.
- Cross-target checks never infer native runtime or network support.
- The all-target default-feature workspace graph permits only the five
  first-party workspace crates and the admitted `sanitization` package. Every
  unlisted package fails closed regardless of its name or purpose.
- Regression tests cover unknown targets, missing targets, extra arguments,
  exact command construction, native mode, and forbidden dependencies.
- Publishable READMEs use immutable release wording; validation rejects
  development-only status that would become false in crates.io snapshots.
- Existing response bounds, credential redaction, timeout, redirect, retry,
  and cleanup requirements remain mandatory for every future transport.

## Version Plan

- `cloud-sdk` publishes metadata release `0.20.0`.
- `cloud-sdk-hetzner` publishes dependency-only patch `0.17.1`.
- `cloud-sdk-reqwest` publishes dependency-only patch `0.15.2`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.6`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.15.2`.
- Retired and provider-scoped helper packages remain rejected and unpublished.

## Verification

- `scripts/checks.sh`
- `scripts/test-platform-matrix.py`
- `scripts/check_platform_matrix.sh --default-boundary`
- `scripts/check_platform_matrix.sh --all`
- target-specific `scripts/check_platform_matrix.sh --portable TARGET`
- native Linux, Windows, macOS ARM64, and macOS x86-64 CI jobs
- `scripts/check_rust_version_matrix.sh`
- `scripts/check_hetzner_upstream.sh --local-only`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_sbom_freshness.sh`
- `scripts/release_crates.py --dry-run --yes --version 0.20.0`
- `cargo deny check`
- fixture-scoped advisory, license, and source checks
- workspace and fixture RustSec audits
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
