# cloud-sdk 0.16.0 Release Notes

Status: implementation complete; pentest and retest required before tagging.

## Scope

`0.16.0` admits the first provider-neutral blocking production transport in
`cloud-sdk-reqwest`, behind the non-default `blocking-rustls` feature. The
default facade, provider, reqwest-boundary, testkit, and sanitization graphs
remain no_std and transport-free.

This release does not add async transport, automatic retries, redirects,
proxies, response decompression, pagination helpers, action polling, live API
tests, token generation, or secret-manager integration.

## Added

- Explicit bounded content-type metadata in the provider-neutral transport
  request contract.
- Validated HTTPS endpoints with exact authority preservation and guarded
  origin-form target composition.
- Owned redacted bearer authorization with adapter-owned volatile cleanup.
- Required user-agent, total-timeout, and connect-timeout configuration.
- Hardened reqwest client policy using rustls, TLS 1.2 minimum, no redirects,
  no retries, no proxies, no referer, no response decompression, HTTP/1 only,
  and the system resolver only.
- Sized sanitized request-body copies and caller-bounded response reads.
- Payload-free transport failures and response-buffer cleanup on every error.
- Deterministic loopback tests for exact wire output, status handling,
  redirect refusal, retry refusal, body bounds, and timeouts.
- Dependency admission evidence and `scripts/check_reqwest_boundary.sh`.
- `scripts/release_0_16_gate.sh`.

## Security Notes

- Production endpoints are HTTPS-only and cannot contain credentials, query,
  fragment, or ambiguous trailing path separators.
- Encoded structural path bytes and URL normalization changes are rejected
  before attaching authorization.
- Bearer tokens, request bodies, request targets, response bodies, endpoints,
  content types, and transport errors are redacted or payload-free as
  appropriate.
- Adapter-owned token and request-body storage uses
  `cloud-sdk-sanitization`; direct first-party `zeroize` use is forbidden.
- Rustls internally depends on `zeroize`; this transitive TLS implementation
  detail is documented and is not the SDK cleanup boundary.
- Callers remain responsible for token generation, scope, rotation,
  revocation, and cleanup of their original secret storage.
- No redirect, retry, proxy, decompression, or environment-derived routing can
  silently alter an authenticated request.
- A locked non-published fixture enables reqwest HTTP/2 and Hickory DNS to
  verify runtime policy remains frozen under downstream feature unification.
- The fixture's separate dependency graph receives independent advisory,
  license, source, audit, and SPDX SBOM verification in release CI.

## Version Plan

- `cloud-sdk` publishes its code release as `0.16.0`.
- `cloud-sdk-reqwest` publishes its code release as `0.13.0`.
- `cloud-sdk-hetzner` publishes dependency-only patch `0.15.1`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.2`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.13.1`.
- Retired and provider-scoped helper packages remain rejected and unpublished.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo test -p cloud-sdk-reqwest --all-features`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_rust_version_matrix.sh`
- `scripts/test-release-readiness.sh`
- `scripts/test-release-crates.py`
- `scripts/checks.sh`
- `scripts/release_0_16_gate.sh`
- `cargo deny check`
- `cargo deny --manifest-path tests/reqwest-feature-unification/Cargo.toml --config deny.toml --locked check advisories licenses sources`
- `cargo audit`
- `cargo audit --no-fetch --file tests/reqwest-feature-unification/Cargo.lock`
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
