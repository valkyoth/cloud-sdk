# v0.39.0 Dependency Review

Date: 2026-07-29

Scope: transactional encoding and optional owned preparation storage.

## Result

No external dependency is added or upgraded. The default `cloud-sdk` graph
continues to use only `cloud-sdk-sanitization` and its admitted
`sanitization 2.0.3` primitive. `alloc` convenience uses only Rust's `alloc`
crate and remains disabled by default.

Registry checks on 2026-07-29 confirmed the pinned stable direct dependencies,
Cargo security tools, cargo-sbom, cargo-fuzz, and isolated parser/fuzzer
dependencies are current. Rustls `0.24.0-dev.1` is a prerelease; stable
`0.23.42` remains selected.

| Package | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.39.0` | transactional encoder and preparation profiles |
| `cloud-sdk-hetzner` | `0.32.0` | provider request writer migration |
| `cloud-sdk-reqwest` | `0.26.1` | dependency-only core range update |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.23.1` | dependency-only core range update |

The default graph adds no transport, TLS, runtime, filesystem, clock, Serde, or
allocation capability.

## Required Verification

- `scripts/check_atomic_encoders.sh`
- `scripts/check_sanitization_boundary.sh`
- `scripts/check_platform_matrix.sh --all`
- `scripts/release_crates.py --check`
- Cargo Deny, RustSec, package, documentation, fuzz, MSRV, and SPDX checks
- `scripts/release_0_39_gate.sh`
