# cloud-sdk 0.27.0 Release Notes

Release date: 2026-07-15

## Summary

`0.27.0` stabilizes the existing Hetzner Cloud, DNS, and Console Storage Box
surface before Robot implementation. Required request fields are represented by
types instead of runtime `Option` checks, public errors integrate with Rust's
standard error traits without exposing payloads, and custom credential
destinations are explicit.

The release does not add an end-to-end Hetzner client or broaden the current
Serde response surface. Request-operation coverage remains 208/208 active
operations with 13 deprecated operations deliberately excluded.

## Required Constructor Inputs

Provider request constructors now take source-required validated values
directly. This removes invalid intermediate states and the broad
`MissingRequiredField` error from affected Cloud, DNS, security, and Storage
Box requests. Optional, nullable, and explicit set/reset semantics continue to
use `Option` or intent enums where they represent real API states.

Compile-fail documentation proves representative server and RRSet create
requests cannot omit required inputs. Cross-field conflicts and validation of
raw names, addresses, domains, timestamps, ranges, and bodies remain fallible.

## Error Contract

Every public first-party error family now implements payload-free `Display` and
`core::error::Error` under the MSRV. Messages are fixed literals. Nested errors
and provider error envelopes do not format request targets, endpoints, bodies,
credentials, provider messages, or customer-controlled values.

Server action validation now reports `EmptyAliasIps` and
`ActionBodyRequired` where those precise states can occur.

## Credential Destination Safety

`HttpsEndpoint::new` is replaced by `HttpsEndpoint::new_custom`. The name and
documentation make clear that the configured origin receives the supplied
bearer token. Blocking and async examples require trusted operator
configuration and warn against tenant-controlled endpoint input.

Official Hetzner endpoint constructors remain planned for the high-level client
milestone. The current adapter remains provider-neutral.

## Public API And Documentation

The v0.27 review records feature boundaries, crate ownership, deferred client
work, versioning rules, error behavior, and deprecated-upstream handling. A new
regression checker rejects an ambiguous `Supported` capability table or claims
that body serialization, typed responses, or the end-to-end client are already
complete.

See `docs/MIGRATION_0.27.0.md` for changed signatures and endpoint setup.

## Independent Crate Versions

- `cloud-sdk` publishes code release `0.27.0`.
- `cloud-sdk-hetzner` publishes code release `0.21.0`.
- `cloud-sdk-reqwest` publishes code release `0.18.0`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.13`.
- `cloud-sdk-testkit` publishes code release `0.16.0`.

No retired provider-specific helper crate is published.

## Verification

- `cargo test --workspace`
- `cargo test --workspace --all-features`
- `cargo test --workspace --doc --all-features`
- `scripts/check-provider-capabilities.py`
- `scripts/test-provider-capabilities.py`
- `scripts/check_api_matrix_coverage.py`
- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/check_platform_matrix.sh --all`
- `scripts/check_rust_version_matrix.sh`
- `scripts/check_fuzz_harness.sh --build`
- `scripts/check_fuzz_harness.sh --smoke`
- `scripts/check_sbom_freshness.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/release_crates.py --dry-run --yes --version 0.27.0`
- `scripts/release_0_27_gate.sh`

## Security Review

Two separate pentests reviewed commit
`f115f9016a113fb00d2db87e6b4fd22f8000152f`. Neither assessment identified a
security finding requiring remediation, so no redundant retest was needed.
The permanent report records `PASS`; tagging remains blocked until the clean
release gate, GitHub CI, and CodeQL default setup are green.
