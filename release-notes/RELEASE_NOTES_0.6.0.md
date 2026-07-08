# cloud-sdk 0.6.0 Release Notes

Status: implementation stop; awaiting pentest.

## Scope

`0.6.0` adds no_std Hetzner server request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, serde request/response
models, body serialization, token storage, live API tests, retry policy,
pagination iterators, or action polling.

## Added

- `cloud_sdk_hetzner::cloud::servers` request domains for server list, create,
  get, update, delete, and metrics.
- `cloud_sdk_hetzner::cloud::servers::actions` request domains for global
  server action list/get, per-server action list, and all source-locked server
  start-action paths.
- Server create request validation for required `name`, `server_type`, and
  `image` fields.
- Server public network option validation for disabled IP family versus
  explicit primary IP IDs.
- Server metrics request validation for RFC3339-like timestamps and start/end
  ordering.
- Explicit DNS pointer action intent via set or reset, so omitted `dns_ptr`
  behavior is not modeled as an accidental default.
- Pentest remediation for cloud-init user data redaction, zero numeric query
  serialization, JSON-significant byte rejection, bidi-control rejection, and
  fixed-width metrics timestamp validation.
- `cloud_sdk::buffer`, a shared no_std fixed-buffer writer for provider crates.
  The Hetzner server and security request domains now use it for string,
  decimal, query, and percent-encoded output while preserving domain-specific
  error enums.
- Shared JSON-string escaping in `cloud_sdk::buffer` plus a `UserData`
  body-writing path for future server create body serialization.
- Tests for source-locked server paths, list query construction, required
  create fields, mutual exclusions, metrics time ranges, DNS pointer intent,
  body-required action guards, shared buffer writer behavior, user-data JSON
  escaping, pentest remediation paths, and every server action path.
- `scripts/release_0_6_gate.sh`.

## Security Notes

- The default graph remains no_std and transport-free.
- Server and action IDs are nonzero.
- Server ID paths validate the fully assembled path through the shared
  `EndpointPath` boundary.
- Metrics and list queries use caller-owned fixed buffers and percent encoding.
- Cloud-init user data redacts through `Debug`.
- Cloud-init user data is not JSON-safe by validation; future body serializers
  must use its escaped JSON string writer.
- Server references and text action fields reject JSON-significant bytes before
  future body serialization support exists.
- Metrics timestamps use fixed-width UTC `YYYY-MM-DDTHH:MM:SSZ` validation so
  lexicographic start/end checks remain meaningful.
- Numeric and percent-encoded query output uses one shared tested writer across
  security and server request domains.
- The SDK still does not serialize request bodies or execute API requests.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p cloud-sdk-hetzner --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features servers`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_6_gate.sh`

## Pentest

- Pending. Stop at the implementation commit and run pentest before release
  metadata is finalized.
