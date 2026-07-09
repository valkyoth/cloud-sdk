# cloud-sdk 0.9.0 Release Notes

Status: implementation stop; awaiting pentest.

## Scope

`0.9.0` adds no_std Hetzner Storage Box request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, serde request/response
models, body serialization, token storage, live API tests, retry policy,
pagination iterators, action polling, or Robot Webservice support.

## Added

- `cloud_sdk_hetzner::storage::storage_boxes` request domains for Storage Box
  list/create/get/update/delete and folder-list endpoints.
- Storage Box type list/get request domains.
- Storage Box snapshot list/create/get/update/delete request domains.
- Storage Box subaccount list/create/get/update/delete request domains.
- Storage Box action endpoint paths for list/get, protection, type change,
  snapshot plan, password reset, rollback, and access settings.
- Storage Box subaccount action endpoint paths for home-directory changes,
  password reset, and access settings.
- Fixed-buffer query builders for source-locked Storage Box list endpoints.
- Redacted `StorageBoxPassword` with JSON-string writing and no raw string
  accessor.
- Bounded snapshot-plan marker types and conservative subaccount home-directory
  validation.
- `scripts/release_0_9_gate.sh`.

## Security Notes

- The default graph remains no_std and transport-free.
- Storage Box, type, snapshot, subaccount, and action IDs are nonzero.
- Endpoint paths validate the fully assembled path through the shared
  `EndpointPath` boundary.
- List queries use caller-owned fixed buffers and percent encoding.
- Password-like request values are redacted in `Debug` and can only be written
  through JSON-string escaping in this release.
- The deprecated resource-local action lookup endpoint remains intentionally
  deferred.
- The SDK still does not serialize request bodies or execute API requests.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p cloud-sdk-hetzner --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features storage_box`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_9_gate.sh`

## Pentest

- Pending. Stop at the implementation commit and run pentest before release
  metadata is finalized.
