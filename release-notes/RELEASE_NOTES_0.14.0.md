# cloud-sdk 0.14.0 Release Notes

Status: implementation candidate; pentest pending.

## Scope

`0.14.0` adds an optional no_std Serde boundary to `cloud-sdk-hetzner`. It
serializes the complete v0.13 RRSet request-body surface and deserializes shared
action and API error response envelopes. It also admits provider-neutral
volatile cleanup for caller-owned secret buffers. It does not add HTTP
transport, token storage, broad serialization for older resource domains,
resource response models, retry policy, pagination iterators, action polling,
or live API tests.

## Added

- Non-default `serde` feature with allocation, Serde defaults disabled, and no
  Serde `std` feature.
- Checked `RrsetRequestBody` constructors for create, labels update,
  protection, TTL, set, add, remove, and comment-update operations.
- Conservative 1 MiB RRSet JSON upper-bound enforcement before complete bodies
  become serializable, including worst-case control, non-ASCII, and surrogate
  escaping even where current constructors make a case unreachable.
- Validated `ActionEnvelope` and `ApiErrorEnvelope` response models using
  borrowed-or-owned text.
- `ResponseBytes` with an 8 MiB pre-parser input cap, plus 256-resource and
  interpreted response-text bounds.
- JSON fixtures for success, error, explicit null, escaped strings, duplicate
  fields, missing fields, unknown fields, invalid IDs, status, and progress.
- Serde dependency admission evidence and an automated default-graph gate.
- `cloud-sdk-sanitization::SecretBuffer` and explicit `sanitize_bytes` cleanup
  through `sanitization` 1.2.4 with default features disabled.
- Atomic escaped private-key JSON output without raw private-key access or
  ordinary equality, plus provider integration tests using `SecretBuffer`.
- `scripts/release_0_14_gate.sh`.

## Security Notes

- Complete request structs do not implement `Serialize`; checked wrappers omit
  endpoint selectors and enforce aggregate size policy.
- Record values and comments serialize through validated types without raw
  string accessors or interpolation.
- Explicit inherited TTL emits JSON `null`; optional omission remains possible
  only for source schemas that permit it.
- Response wire models are private. Nonzero IDs, known action status, progress
  at most 100, and interpreted text controls are validated after parsing.
- Raw response bytes and API error messages are redacted from `Debug` output.
- Storage Box passwords and certificate private keys, along with request types
  containing them, do not implement ordinary variable-time equality.
- Guarded caller-owned buffers are volatile-cleared on success, error, early
  return, and unwind where unwind exists. Source strings and downstream copies
  remain caller and transport responsibilities.
- Private-key JSON writes preflight capacity and leave undersized destination
  buffers unchanged.
- Known duplicate and missing response fields fail. Unknown response fields are
  ignored to tolerate additive provider changes.
- Escaped response strings use `Cow`, borrowing when possible and allocating
  only when unescaping requires ownership.
- Callers must use `ResponseBytes` before their selected format parser; direct
  parser calls bypass the raw response-size policy.
- Serde and its proc-macro graph are absent from default builds. serde_json is
  test-only and is not approved as a production transport parser by this
  release.

## Version Plan

- `cloud-sdk` publishes as `0.14.0`.
- `cloud-sdk-hetzner` publishes its code release as `0.14.0`.
- `cloud-sdk-sanitization` publishes its code release as `0.13.0`.
- `cloud-sdk-reqwest` and `cloud-sdk-testkit` publish dependency-only patch
  releases as `0.12.2`.
- Retired Hetzner-specific boundary packages remain rejected and unpublished.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `scripts/check_serde_boundary.sh`
- `scripts/check_sanitization_boundary.sh`
- `cargo test -p cloud-sdk-sanitization --all-features`
- `cargo package -p cloud-sdk-hetzner --features serde`
- `scripts/test-release-readiness.sh`
- `scripts/test-release-crates.py`
- `scripts/test-hetzner-api-drift.py`
- `scripts/test-iana-ipv6-registry.py`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_14_gate.sh`
- `cargo deny check`
- `cargo audit`
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
