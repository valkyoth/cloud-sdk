# Reqwest Blocking FIPS Dependency Admission

Status: admitted only through the non-default
`cloud-sdk-reqwest/blocking-rustls-fips` feature.

## Decision

| Crate | Version | Role |
| --- | --- | --- |
| `reqwest` | `0.13.4` | blocking HTTP transport |
| `rustls` | `0.23.42` | TLS configuration and FIPS status checks |
| `rustls-platform-verifier` | `0.7.0` | reqwest graph dependency; not the FIPS verifier |
| `aws-lc-rs` | `1.17.1` | rustls cryptographic provider |
| `aws-lc-fips-sys` | `0.13.15` | AWS-LC-FIPS 3.0.x native module |
| `aws-lc-sys` | `0.42.0` | compiled transitive dependency retained by rustls feature unification |

The feature is additive and disabled by default. It does not enter the
`cloud-sdk`, provider, reqwest-default, or reqwest-`std` graphs. The published
manifest uses exact requirements for every package in the table, including
the three AWS-LC packages that would otherwise be transitive. The repository
resolution is locked by `Cargo.lock`, checked by Cargo Deny and RustSec, and
recorded in the workspace SPDX SBOM. Applications must also retain and review
their own `Cargo.lock` with `--locked`, or vendor and verify all sources;
library lockfiles do not control unrelated downstream dependencies.

## Runtime Boundary

The adapter never relies on rustls' process-global default provider. It:

1. constructs `rustls::crypto::default_fips_provider()` explicitly;
2. rejects it unless `CryptoProvider::fips()` returns true;
3. requires caller-owned trust roots and at least one complete CRL;
4. builds a WebPKI verifier that checks the complete chain, denies unknown
   revocation status, and rejects expired CRLs;
5. builds a `ClientConfig` from that provider and verifier, with safe protocol
   versions and no client authentication;
6. rejects the complete configuration unless `ClientConfig::fips()` returns
   true; and
7. passes that exact configuration to reqwest's preconfigured TLS boundary.

This makes missing or preinstalled process-global providers irrelevant. When
`blocking-rustls` and `blocking-rustls-fips` are both enabled, the FIPS-specific
construction path wins. Client construction remains fallible and payload-free.

The application owns authenticated distribution, completeness, freshness,
issuer coverage, and emergency replacement of its roots and CRLs. An absent,
empty, malformed, unknown-status, or expired revocation policy fails closed.
The FIPS path does not use Linux platform-verifier behavior that omits
revocation checks.

Rustls `0.23.42` implements its official `fips` feature by enabling both the
`aws-lc-rs/fips` backend and its ordinary `aws_lc_rs` feature. Cargo therefore
compiles both `aws-lc-fips-sys` and `aws-lc-sys`; `aws-lc-rs` selects the FIPS
FFI under the unified `fips` feature. The boundary checks this exact graph and
does not make the inaccurate claim that the ordinary sys crate is absent from
the build supply chain.

## Validation And Compliance Scope

NIST CMVP certificate
[`#4816`](https://csrc.nist.gov/projects/cryptographic-module-validation-program/certificate/4816)
covers AWS-LC FIPS `2.0.0`, not the `3.0.x` module bound by current
`aws-lc-fips-sys 0.13.15`. The current package describes its module as having
completed accredited validation testing and been submitted to NIST. This
release therefore does not claim that the selected `3.0.x` module has an
active validation certificate.

The upstream package status, NIST module listings, and any new certificate and
security policy must be rechecked before every dependency update and before a
caller makes a compliance decision. A crate feature and successful `fips()`
checks do not establish that an application, build pipeline, operating
environment, entropy source, deployment, or organization is FIPS compliant.

## Build And Supply-Chain Boundary

The bundled FIPS build requires native C/C++ compilation, CMake, Go, Perl, a
linker, and target-appropriate bindings. Targets without pregenerated bindings
also require libclang/bindgen. These tools execute before application runtime
and are part of the trusted build boundary.

The upstream build script can discover a system AWS-LC installation through
AWS-LC, OpenSSL, or pkg-config environment settings. Repository checks set
`AWS_LC_FIPS_SYS_USE_SYSTEM=0`, forcing the Cargo-authenticated bundled source.
Applications intentionally using a system module must independently pin and
verify its library, bindings, FIPS mode, loader path, version, and approved
operating environment. The repository does not test or certify that mode.

Release builders must retain Cargo checksum verification, use reviewed pinned
build images, avoid untrusted compiler environment flags, and preserve build
logs. Reproducible byte-for-byte native outputs are not claimed in `v0.23.0`.
Cargo Deny narrowly skips duplicate detection for build-only `shlex 1.3.0`:
bindgen requires the 1.x line while cc requires 2.x. No runtime duplicate is
allowed, and `v0.24.0` must re-evaluate this exception.

Primary upstream references:

- <https://docs.rs/rustls/0.23.42/rustls/manual/_06_fips/>
- <https://docs.rs/aws-lc-fips-sys/0.13.15/aws_lc_fips_sys/>
- <https://aws.github.io/aws-lc-rs/requirements/index.html>
- <https://aws.github.io/aws-lc-rs/platform_support.html>

## Platform Scope

The repository runs the FIPS boundary and runtime status test on
`x86_64-unknown-linux-gnu`. Other targets are unsupported for the FIPS feature
until a dedicated job is added and its exact operating environment is checked
against the active security policy. This limitation does not reduce the
portable no_std or ordinary reqwest transport support claims.

## Verification

`scripts/check_reqwest_fips_boundary.sh` checks exact published constraints and
the resolved direct/native graph, rejects alternate TLS, crypto, and
decompression dependencies, forces bundled source, runs provider,
configuration, CRL-policy, and process-global independence tests, and compiles
the additive blocking feature combination. `scripts/release_0_23_gate.sh` also
runs workspace tests, platform checks, Cargo Deny, RustSec, SBOM freshness,
upstream drift checks, and pentest readiness.
