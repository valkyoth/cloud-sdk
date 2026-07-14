# cloud-sdk 0.24.0 Release Notes

Release date: 2026-07-14

## Summary

`0.24.0` completes the dependency and tooling hardening milestone and adds an
optional deterministic public WebPKI root mode to the provider-neutral
blocking transport. Default features remain empty, and no transport, TLS,
runtime, native-code, filesystem, or allocator dependency enters the default
facade or provider graphs.

## Deterministic Root Mode

`cloud-sdk-reqwest/blocking-rustls-webpki-roots` builds a complete custom
rustls client configuration from `webpki-roots 1.0.8` and an explicit AWS-LC
provider. The client consults only the compiled Mozilla root snapshot, even
though reqwest retains a platform-verifier dependency in its compiled graph.
Host-added enterprise or private roots cannot alter this client's decisions.

The mode intentionally does not add revocation checking, certificate pinning,
private roots, or FIPS status. Root changes require a reviewed crate update.
If the FIPS feature is also enabled, its mandatory caller roots and complete
CRL policy take precedence.

## Dependency And Native Review

Every direct normal, optional, development, and fuzz dependency was checked
against crates.io and was current on 2026-07-14. The review records the three
AWS-LC Cargo archive checksums, bundled-source enforcement, transitive graph,
SBOM coverage, and the trust placed in native build scripts and C/C++/assembly
tools. The build-only `shlex 1.3.0`/`2.0.1` duplicate remains necessary for
bindgen and cc and has no runtime path.

All maintained repository checks that can compile AWS-LC force bundled
ordinary and FIPS sources. Target-qualified system-library controls are
rejected because they take precedence over generic controls; regression tests
bind this policy to standalone checks and the release gate.

The FIPS CI job remains pinned to Ubuntu 22.04, but hosted image tools are
mutable. This release does not claim byte-for-byte native reproducibility;
production and offline builders must pin an immutable toolchain image and
preserve Cargo checksum verification.

## Tooling

Development remains on stable Rust `1.97.0`, with compatibility checked from
Rust `1.90.0` through `1.97.0`. Cargo Deny `0.20.2`, cargo-audit `0.22.2`,
cargo-sbom `0.10.0`, and cargo-fuzz `0.13.2` were current. The tooling checker
now validates installed pins and compares Cargo tool releases with crates.io.
`actions/checkout v7.0.0` remains pinned to its verified tag commit.

## Independent Crate Versions

- `cloud-sdk` publishes metadata release `0.24.0`.
- `cloud-sdk-reqwest` publishes code release `0.17.0`.
- `cloud-sdk-hetzner` publishes dependency-only patch `0.19.2`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.10`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.15.6`.

No retired provider-specific helper crate is published.

## Verification

- `scripts/checks.sh`
- `scripts/check_latest_tools.sh --fetch`
- `scripts/check_reqwest_webpki_roots_boundary.sh`
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
- `scripts/release_crates.py --dry-run --yes --version 0.24.0`
- `scripts/release_0_24_gate.sh` after pentest evidence is committed.

## Security Review

The v0.24 pentest identified a target-qualified AWS-LC system-library override
that could bypass the repository's generic bundled-source control. The fix
rejects target-qualified controls, forces both ordinary and FIPS bundled
sources, and binds maintained native build entry points through regression
tests. The focused retest passed; the permanent report is stored at
`security/pentest/v0.24.0.md`. Tagging remains blocked until GitHub CI, CodeQL
default setup, and the clean versioned release gate pass.
