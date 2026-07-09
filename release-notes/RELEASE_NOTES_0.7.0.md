# cloud-sdk 0.7.0 Release Notes

Status: implementation stop; awaiting pentest.

## Scope

`0.7.0` adds no_std Hetzner server-adjacent request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, serde request/response
models, body serialization, token storage, live API tests, retry policy,
pagination iterators, or action polling.

## Added

- `cloud_sdk_hetzner::cloud::images` request domains for image list/get,
  update, delete, action list/get, per-image action list, and image protection
  action paths.
- `cloud_sdk_hetzner::cloud::servers::placement_groups` request domains for
  placement group list/create/get/update/delete.
- `cloud_sdk_hetzner::cloud::networks::primary_ips` request domains for primary
  IP list/create/get/update/delete and primary IP actions.
- Shared `cloud_sdk_hetzner::cloud::shared` request helpers for Cloud resource
  IDs, JSON-safe bounded text/name values, ordered label slices, fixed-buffer
  paths, and fixed-buffer query construction.
- Primary IP action request markers for assign, unassign, DNS pointer, and
  protection operations.
- Explicit DNS pointer set/reset intent for primary IPs, so omitted `dns_ptr`
  behavior is not modeled as an accidental default.
- Primary IP create/update request shapes intentionally do not expose removed
  datacenter request fields.
- `scripts/release_0_7_gate.sh`.

## Security Notes

- The default graph remains no_std and transport-free.
- Server-adjacent resource IDs are nonzero.
- Server-adjacent endpoint paths validate the fully assembled path through the
  shared `EndpointPath` boundary.
- List queries use caller-owned fixed buffers and percent encoding.
- Bounded Cloud text/name values reject control bytes, JSON-significant bytes,
  and bidi-control scalars before future body serialization exists.
- Primary IP DNS pointer changes require explicit set or reset intent.
- Deprecated resource-local action lookup endpoints remain deferred.
- The SDK still does not serialize request bodies or execute API requests.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p cloud-sdk-hetzner --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features server_adjacent`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_7_gate.sh`

## Pentest

- Pending. Stop at the implementation commit and run pentest before release
  metadata is finalized.
