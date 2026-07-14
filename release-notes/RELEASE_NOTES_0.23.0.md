# cloud-sdk 0.23.0 Release Notes

Release date: 2026-07-14

## Summary

`0.23.0` adds an optional fail-closed blocking rustls FIPS-mode transport to
the provider-neutral reqwest crate. Default features remain empty and no
transport, TLS, runtime, native-code, filesystem, or allocator dependency
enters the default facade or provider graphs.

## FIPS-Mode Construction

`cloud-sdk-reqwest/blocking-rustls-fips` constructs
`rustls::crypto::default_fips_provider()` directly, checks
`CryptoProvider::fips()`, builds a complete `ClientConfig` from that provider,
and checks `ClientConfig::fips()` before passing the configuration to reqwest.
It does not rely on rustls' process-global provider. If both blocking features
are selected, this explicit FIPS path wins.

The ordinary HTTPS, TLS 1.2 minimum, platform certificate verifier, HTTP/1,
system resolver, timeout, redirect, retry, proxy, referer, decompression,
authority, response-bound, redaction, and sanitization policies are unchanged.

## Validation Limit

The current graph resolves `aws-lc-rs 1.17.1` and
`aws-lc-fips-sys 0.13.15`, which binds AWS-LC-FIPS 3.0.x. NIST certificate
`#4816` covers AWS-LC FIPS 2.0.0 and is not presented as validation evidence
for this dependency line. Runtime FIPS status is not an application,
deployment, operating-environment, or organizational compliance claim.

Rustls' current feature graph also compiles ordinary `aws-lc-sys 0.42.0`
alongside `aws-lc-fips-sys`; aws-lc-rs selects the FIPS FFI under unified
features. This compiled supply-chain surface is checked and documented rather
than incorrectly claimed absent.

## Build And CI

Repository checks set `AWS_LC_FIPS_SYS_USE_SYSTEM=0`, forcing the
Cargo-authenticated bundled source. Native C/C++, CMake, Go, Perl, linker, and
bindings tooling remain trusted build inputs. A dedicated Ubuntu Linux job
runs the exact graph and runtime FIPS checks; it is not described as a
validated operating environment. Standard native Linux, Windows, and macOS
jobs exclude the separately scoped FIPS feature.

## Release Publisher Fix

The crate publisher no longer reruns the complete network-sensitive gate after
a signed tag has bound the unchanged locally and GitHub-approved commit. It
still requires a clean tree, matching verifiable signed tag, strict release
metadata, and the independent crate publish plan. `--rerun-gate` explicitly
requests a second full gate run. Subprocess failures now produce one concise
release-command diagnostic instead of a Python traceback.

## Independent Crate Versions

- `cloud-sdk` publishes metadata release `0.23.0`.
- `cloud-sdk-reqwest` publishes code release `0.16.0`.
- `cloud-sdk-hetzner` publishes dependency-only patch `0.19.1`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.9`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.15.5`.

No retired provider-specific helper crate is published.

## Verification

- `scripts/checks.sh`
- `scripts/check_reqwest_fips_boundary.sh`
- `scripts/check_platform_matrix.sh --all`
- `scripts/check_rust_version_matrix.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_fuzz_harness.sh --build`
- `scripts/check_fuzz_harness.sh --smoke`
- `scripts/check_sbom_freshness.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/release_crates.py --dry-run --yes --version 0.23.0`
- `scripts/release_0_23_gate.sh` after pentest evidence is committed.

## Security Review Stop

Pentest and any required focused retest must pass before tagging. The permanent
report will be stored at `security/pentest/v0.23.0.md` and bound to the exact
reviewed implementation commit.
