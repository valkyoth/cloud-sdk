# cloud-sdk 0.15.0 Release Notes

Status: implementation candidate; pentest pending.

## Scope

`0.15.0` adds provider-neutral blocking transport contracts to `cloud-sdk` and
the first usable `cloud-sdk-testkit`. It provides deterministic no_std mock
transport, bounded response fixtures, compact oversized bodies, and a shared
adversarial response corpus before real HTTP adapters are admitted.

It does not add network access, TLS, DNS resolution, authentication headers,
token storage, timeouts, retry policy, async runtime integration, live API
tests, or a production transport implementation.

## Added

- `RequestTarget` validation for bounded origin-form `/path?query` targets.
- `TransportRequest`, bounded `StatusCode`, `TransportResponse`, and
  `BlockingTransport` contracts over caller-owned response buffers.
- Ordered `MockTransport` exchanges with distinct payload-free method, target,
  body, exhaustion, capacity, and cursor failures.
- Atomic `FixtureBody` output from borrowed bytes or compact repeated-byte
  bodies up to 8 MiB plus one byte.
- Success, pagination, action, rate-limit, and error response fixture builders.
- Coherent pagination, action-progress, and rate-limit metadata validation.
- Six-case adversarial corpus for malformed JSON, unknown fields, missing
  fields, oversized responses, invalid pagination, and invalid action state.
- Hetzner Serde tests consuming applicable shared adversarial fixtures.
- `scripts/check_testkit_boundary.sh` and `scripts/release_0_15_gate.sh`.

## Security Notes

- Request targets and request/response bodies are redacted from debug output.
- Fixture writes preflight destination capacity and leave undersized buffers
  unchanged. Failed and mismatched exchanges are not consumed.
- Mock errors carry no compared request or response payloads.
- The mock uses ordinary exact-byte matching and is test infrastructure, not a
  production secret-comparison boundary.
- The testkit default graph contains only `cloud-sdk`; it has no provider,
  network, TLS, filesystem, clock, process, or runtime dependency.
- Authentication, trusted base URLs, headers, TLS, timeout, retry, and secret
  ownership remain explicit future adapter responsibilities.

## Version Plan

- `cloud-sdk` publishes its code release as `0.15.0`.
- `cloud-sdk-testkit` publishes its code release as `0.13.0`.
- `cloud-sdk-hetzner` publishes its test-integration code release as `0.15.0`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.1`.
- `cloud-sdk-reqwest` publishes dependency-only patch `0.12.3`.
- Retired provider-specific helper packages remain rejected and unpublished.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo test -p cloud-sdk-testkit --all-features`
- `cargo tree -p cloud-sdk-testkit --no-default-features`
- `scripts/check_testkit_boundary.sh`
- `scripts/check_rust_version_matrix.sh`
- `scripts/test-release-readiness.sh`
- `scripts/test-release-crates.py`
- `scripts/checks.sh`
- `scripts/release_0_15_gate.sh`
- `cargo deny check`
- `cargo audit`
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
