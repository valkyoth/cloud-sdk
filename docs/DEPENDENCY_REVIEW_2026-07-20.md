# Dependency And Tooling Review - 2026-07-20

Scope: maintenance-only refresh of the workspace, excluded fuzz package,
feature-unification fixture, and prepared-operation checker. No published crate
version, default feature, or provider API changes in this review.

## Compiler And Tools

| Component | Reviewed version | Decision |
| --- | --- | --- |
| Rust stable | `1.97.1` | development and full-gate compiler |
| Rust compatibility | `1.90.0` through `1.97.1` | retained |
| Rust nightly | `nightly-2026-07-20` | excluded fuzz package only |
| `cargo-deny` | `0.20.2` | current; unchanged |
| `cargo-audit` | `0.22.2` | current; unchanged |
| `cargo-sbom` | `0.10.0` | current; unchanged |
| `cargo-fuzz` | `0.13.2` | current; unchanged |
| `actions/checkout` | `v7.0.0` | current tag and immutable commit unchanged |

Rust `1.97.1` fixes the stable `1.97.0` miscompilation documented by the Rust
project. The MSRV remains `1.90.0`; every updated published dependency declares
a compiler floor at or below it. Nightly remains isolated from published-crate
compilation and release compatibility claims.

## Direct Dependency Updates

| Package | Previous | Reviewed | Rust |
| --- | --- | --- | --- |
| `aws-lc-fips-sys` | `0.13.15` | `0.13.16` | 1.71 |
| `aws-lc-rs` | `1.17.1` | `1.17.3` | 1.71 |
| `aws-lc-sys` | `0.42.0` | `0.43.0` | 1.71 |
| `sanitization` | `1.2.4` | `1.2.5` | 1.90 |
| `serde` | `1.0.228` | `1.0.229` | 1.56 |
| `serde_json` | `1.0.150` | `1.0.151` | 1.71 |
| `tokio` | `1.52.3` | `1.53.0` | 1.71 |
| `webpki-roots` | `1.0.8` | `1.0.9` | 1.70 |
| `syn` | `2.0.119` | `3.0.2` | 1.71 |

`bytes 1.12.1`, `reqwest 0.13.4`, `rustls 0.23.42`,
`rustls-platform-verifier 0.7.0`, and `libfuzzer-sys 0.4.13` remain current.
Cargo refreshed compatible transitive packages in every independent lockfile.

The direct `syn` major update is confined to the unpublished checker. Its AST
changes were migrated explicitly: safety qualifiers, receiver kinds, impl
modifiers, trait paths, and guarded patterns remain fail-closed. No checker
parser dependency enters a published crate or its build script.

The all-feature workspace also resolves `syn 3.0.2` through current Serde while
AWS-LC bindgen and platform procedural macros still require `syn 2.0.119`.
Cargo Deny permits only the older line as a duplicate. Both are compile-time
dependencies; neither enters a runtime path. The exception must be removed when
the remaining parents converge.

## Native Archive Checksums

| Cargo archive | SHA-256 from `Cargo.lock` |
| --- | --- |
| `aws-lc-rs 1.17.3` | `00bdb5da18dac48ca2cc7cd4a98e533e8635a58e2361d13a1a4ee3888e0d72f1` |
| `aws-lc-sys 0.43.0` | `43103168cc76fe62678a375e722fc9cb3a0146159ac5828bc4f0dfd755c2224c` |
| `aws-lc-fips-sys 0.13.16` | `37b00953a69b2cfb471d13d72538d2e66930832340b0f31deadd404b48c573c5` |

Cargo authenticates these archives, but the native C/C++ build chain and host
remain trusted inputs. Repository entry points continue to force bundled
sources and reject system-library discovery variables.

## FIPS Status

NIST certificate `#5314` is active for the static AWS-LC 3 module and identifies
module version `3.1.0`. The bundled `aws-lc-fips-sys 0.13.16` source reports
AWS-LC FIPS `3.4.0`. The repository therefore makes no claim that its exact
module is covered by `#5314` or another active certificate. Runtime provider
and configuration FIPS checks remain necessary but do not establish deployment
or organizational compliance.

## Verification

- Exact manifests and all four lockfiles are checked into the repository.
- Feature and native dependency boundaries assert the reviewed versions.
- Cargo Deny, RustSec, SPDX freshness, no_std, platform, package, parser, and
  full workspace gates must pass before this maintenance commit is accepted.
- Historical release notes, pentest reports, and the v0.24 dependency review
  remain unchanged.
