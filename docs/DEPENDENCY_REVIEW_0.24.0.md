# v0.24.0 Dependency And Tooling Review

Review date: 2026-07-14.

## Direct Published-Crate Dependencies

Every direct third-party normal, optional, and development dependency was
checked against crates.io. The selected releases were current on the review
date and remain compatible with Rust 1.90.

| Crate | Version | Scope | Defaults |
| --- | --- | --- | --- |
| `reqwest` | `0.13.4` | optional transport | disabled |
| `rustls` | `0.23.42` | optional explicit TLS configuration | disabled |
| `rustls-platform-verifier` | `0.7.0` | optional reqwest graph | disabled |
| `webpki-roots` | `1.0.8` | optional deterministic roots | disabled |
| `aws-lc-rs` | `1.17.1` | optional rustls provider | disabled |
| `aws-lc-sys` | `0.42.0` | optional native provider boundary | disabled |
| `aws-lc-fips-sys` | `0.13.15` | optional FIPS native boundary | disabled |
| `bytes` | `1.12.1` | optional async body ownership | disabled |
| `tokio` | `1.52.3` | reqwest tests only | disabled |
| `serde` | `1.0.228` | optional provider models | `alloc`, `derive` only |
| `serde_json` | `1.0.150` | tests only | enabled |
| `sanitization` | `1.2.4` | reviewed secret cleanup | disabled |

The excluded fuzz workspace uses current `libfuzzer-sys 0.4.13`. Transitive
packages are resolved by the three committed lockfiles, checked by Cargo Deny
and RustSec, and enumerated in the workspace, feature-unification, and fuzz
SPDX SBOMs. Published library users must review their own complete resolution.

## Tooling

Rust `1.97.0`, cargo-deny `0.20.2`, cargo-audit `0.22.2`, cargo-sbom `0.10.0`,
and cargo-fuzz `0.13.2` were current. `scripts/check_latest_tools.sh` verifies
the installed pins and compares Cargo tool releases with crates.io. The sole
GitHub action, `actions/checkout v7.0.0`, was current and its tag resolved to
the commit SHA used by every workflow job.

## Deterministic Trust Roots

`cloud-sdk-reqwest/blocking-rustls-webpki-roots` admits
`webpki-roots 1.0.8`. It constructs a complete rustls client configuration
from that compiled Mozilla snapshot and an explicit AWS-LC provider. The
client does not consult host-added roots, although reqwest still compiles its
platform-verifier dependency. Root updates are therefore reviewable and
deterministic for a fixed dependency graph.

This choice trades host enterprise/private roots and immediate operating
system root updates for a source-pinned public WebPKI set. It does not add
revocation checking or pinning. The FIPS feature's caller-managed roots and
CRLs take precedence when both features are selected.

## Native AWS-LC Boundary

The lockfile authenticates the crates.io archives with these SHA-256 checksums:

| Crate | SHA-256 |
| --- | --- |
| `aws-lc-rs 1.17.1` | `4342d8937fc7e5dd9b1c60292261c0670c882a2cd1719cfc11b1af41731e32ad` |
| `aws-lc-sys 0.42.0` | `6d9ceb1da931507a12f4fccea479dccd00da1943e1b4ae72d8e502d707361444` |
| `aws-lc-fips-sys 0.13.15` | `6c0e6249c249b8916c98ebae7bc06216c8dcab3002f32872b4abe642d17063b1` |

Every maintained native check and release entry point sources a shared policy
that forces both `AWS_LC_SYS_USE_SYSTEM=0` and
`AWS_LC_FIPS_SYS_USE_SYSTEM=0`. It rejects
target-qualified variants because those take precedence over the generic
controls. Automatic system module discovery therefore cannot replace either
authenticated bundled source in repository checks. The build still executes
upstream Rust build scripts and native C/C++/assembly tooling. FIPS
additionally requires CMake and Go; Perl and bindgen/libclang may be required
by the target and build path.

The repository pins the FIPS CI operating-system image to `ubuntu-22.04`, but
GitHub-hosted image tools are mutable and are not byte-pinned. Production and
offline builders must use an immutable reviewed image, preserve Cargo archive
checksums and logs, disable system-library discovery, and pin their compiler,
assembler, linker, CMake, Go, Perl, and libclang versions. v0.24 does not claim
byte-for-byte native reproducibility.

The build-only duplicate `shlex 1.3.0` remains necessary for bindgen while cc
uses `shlex 2.0.1`. It has no runtime path, is isolated by the existing Cargo
Deny exception, and must be reconsidered whenever either parent is updated.

## Evidence

- `scripts/checks.sh`
- `scripts/check_latest_tools.sh --fetch`
- `scripts/check_reqwest_webpki_roots_boundary.sh`
- `scripts/check_reqwest_fips_boundary.sh`
- `scripts/check_sbom_freshness.sh`
- `cargo deny check`
- `cargo audit`
