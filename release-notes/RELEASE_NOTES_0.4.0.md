# cloud-sdk 0.4.0 Release Notes

Status: release candidate; pentest and retest complete.

## Scope

`0.4.0` adds read-only Hetzner catalog request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, serde response models,
token storage, live API tests, retry policy, pagination iterators, or action
polling.

## Added

- `cloud_sdk_hetzner::cloud::catalog` request domains.
- List and get request primitives for locations.
- Singleton request primitive for pricing.
- List and get request primitives for server types.
- List and get request primitives for load balancer types.
- List and get request primitives for ISOs.
- Public image list and get request primitives. Public image list requests are
  restricted to provider image types `system` and `app`.
- Fixed-buffer get-path writer for catalog `{id}` paths.
- Fixed-buffer list-query writer for pagination, public image type filters, and
  source-locked sort parameters.
- Tests for catalog paths, pagination support, sorting support, unsupported
  sorting, public image query restriction, and Cloud GET/base URL policy.
- `scripts/release_0_4_gate.sh`.

## Security Notes

- The default graph remains no_std and transport-free.
- Catalog IDs are nonzero.
- Catalog path and query construction writes to caller-owned buffers and reports
  undersized buffers without panicking.
- Catalog get paths validate the fully assembled path through the same
  `EndpointPath` boundary used elsewhere in the SDK.
- Public image listing is explicit about the admitted provider image kinds so
  snapshot and backup images are not accidentally included by the helper.
- Public image get requests document that ID scoping is enforced server-side by
  Hetzner, not proven by the request builder.
- Mutation endpoints for images and other catalog-adjacent resources remain out
  of scope.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p cloud-sdk-hetzner --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features catalog`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_4_gate.sh`

## Pentest

- PASS. Permanent report: `security/pentest/v0.4.0.md`.

## Crate Versions

- `cloud-sdk` publishes as `0.4.0`.
- `cloud-sdk-hetzner` publishes as `0.4.0`.
- `cloud-sdk-hetzner-reqwest` publishes as `0.4.0`.
- `cloud-sdk-hetzner-sanitization` publishes as `0.4.0`.
- `cloud-sdk-hetzner-testkit` publishes as `0.4.0`.
