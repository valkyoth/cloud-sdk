# v0.35.0 Dependency Review

Date: 2026-07-27

Scope: direct and locked dependency freshness during the v0.35 canonical
request-target release.

## Result

No external direct dependency is added or upgraded by the implementation.
Canonical parsing, validation, pair iteration, and transactional assembly use
first-party `core`-only code. `cloud-sdk` continues to have no normal
dependency.

The independent first-party package versions move as documented in
[`CRATE_VERSION_MATRIX.md`](CRATE_VERSION_MATRIX.md):

| Package | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.35.0` | canonical request-target code |
| `cloud-sdk-hetzner` | `0.28.0` | canonical provider path integration |
| `cloud-sdk-reqwest` | `0.23.0` | canonical target adapter integration |
| `cloud-sdk-sanitization` | `0.15.3` | dependency-only patch |
| `cloud-sdk-testkit` | `0.20.0` | exact query-state matching |

No URL parser, percent-encoding library, form encoder, transport, TLS,
runtime, filesystem, clock, or secret-storage dependency enters `cloud-sdk`.
The reqwest adapter retains its existing reviewed optional dependency graph.

The 2026-07-27 crates.io freshness pass confirmed these direct stable versions:

| Package | Stable version |
| --- | --- |
| `reqwest` | `0.13.4` |
| `rustls` | `0.23.42` |
| `rustls-platform-verifier` | `0.7.0` |
| `aws-lc-rs` | `1.17.3` |
| `aws-lc-sys` | `0.43.0` |
| `aws-lc-fips-sys` | `0.13.16` |
| `webpki-roots` | `1.0.9` |
| `bytes` | `1.12.1` |
| `tokio` | `1.53.1` |
| `serde` | `1.0.229` |
| `serde_json` | `1.0.151` |
| `sanitization` | `2.0.3` |
| `libfuzzer-sys` | `0.4.13` |

Crates.io also lists `rustls 0.24.0-dev.1`; it is a prerelease and does not
replace the latest stable `0.23.42` line. Pinned cargo-deny, cargo-audit,
cargo-sbom, and cargo-fuzz versions also match crates.io.

## Required Verification

- `scripts/check_request_targets.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_sanitization_boundary.sh`
- `scripts/check_testkit_boundary.sh`
- all Cargo Deny and RustSec graph checks
- all complete SPDX freshness checks
- Rust 1.90.0 through pinned stable compatibility
- `scripts/checks.sh`
