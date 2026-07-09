# cloud-sdk 0.8.0 Release Notes

Status: implementation stop; awaiting pentest.

## Scope

`0.8.0` adds no_std Hetzner storage and IP request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, serde request/response
models, body serialization, token storage, live API tests, retry policy,
pagination iterators, or action polling.

## Added

- `cloud_sdk_hetzner::cloud::volumes` request domains for volume list/create,
  get, update, delete, action list/get, per-volume action list, attach, detach,
  resize, and protection action paths.
- `cloud_sdk_hetzner::cloud::networks::floating_ips` request domains for
  floating IP list/create/get/update/delete and floating IP actions.
- Bounded `VolumeSizeGb` validation for the source-locked `10..=10240` GB
  range.
- Explicit volume create placement through server or location markers.
- Explicit floating IP create placement through server or home-location
  markers.
- Explicit floating IP DNS pointer set/reset intent.
- `scripts/release_0_8_gate.sh`.

## Security Notes

- The default graph remains no_std and transport-free.
- Volume and floating IP resource IDs are nonzero.
- Endpoint paths validate the fully assembled path through the shared
  `EndpointPath` boundary.
- List queries use caller-owned fixed buffers and percent encoding.
- Volume create and floating IP create use explicit placement enums so future
  body serializers do not accidentally combine server and location fields.
- Floating IP address and DNS pointer values are bounded text markers in this
  transport-free release; semantic IP and hostname validation must be added
  before these markers are serialized into request bodies.
- Deprecated resource-local action lookup endpoints remain deferred.
- The SDK still does not serialize request bodies or execute API requests.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p cloud-sdk-hetzner --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features storage_ip`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_8_gate.sh`

## Pentest

- Pending. Stop at the implementation commit and run pentest before release
  metadata is finalized.
