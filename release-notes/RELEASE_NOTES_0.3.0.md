# cloud-sdk 0.3.0 Release Notes

Status: release candidate; pentest and retest complete.

## Scope

`0.3.0` adds core request and response policy domains for
`cloud-sdk-hetzner`. It does not add HTTP transport, serde request/response
models, endpoint-specific builders, token storage, live API tests, retry policy,
or action polling.

## Added

- Endpoint base URL policy for Hetzner Cloud/DNS and Hetzner Storage Box
  surfaces, including endpoint group to base URL mapping.
- Bounded relative endpoint path validation.
- Fixed-capacity query parameter builder with deterministic key ordering.
- Query component percent-encoding policy signal and fixed-buffer
  percent-encoding writer.
- Conservative label key, value, and selector validation, including malformed
  separator, parenthesis, and unsupported-byte rejection.
- Page, per-page, sort key, and sort direction domains.
- Action status parsing, terminal-state detection, and nonzero action IDs.
- API error-code classification and borrowed error envelopes.
- Rate-limit metadata validation.
- `scripts/release_0_3_gate.sh`.

## Security Notes

- The default graph remains no_std and transport-free.
- Query builders store borrowed values and do not allocate.
- Query encoding writes to caller-owned buffers and reports undersized buffers
  without panicking.
- Endpoint paths reject absolute URLs, authority-override paths, backslashes,
  query/fragment separators, parent directory segments, and
  whitespace/control bytes.
- Endpoint group base URL selection is centralized before endpoint builders are
  introduced.
- Unknown API error codes fail into an explicit `Unknown` category instead of
  being guessed.
- Rate-limit metadata rejects impossible `remaining > limit` states.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p cloud-sdk-hetzner --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_3_gate.sh`

## Pentest

- PASS. Permanent report: `security/pentest/v0.3.0.md`.

## Crate Versions

- `cloud-sdk` publishes as `0.3.0`.
- `cloud-sdk-hetzner` publishes as `0.3.0`.
- `cloud-sdk-hetzner-reqwest` publishes as `0.3.0`.
- `cloud-sdk-hetzner-sanitization` publishes as `0.3.0`.
- `cloud-sdk-hetzner-testkit` publishes as `0.3.0`.
