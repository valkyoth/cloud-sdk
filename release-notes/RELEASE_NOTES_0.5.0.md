# cloud-sdk 0.5.0 Release Notes

Status: implementation stop; awaiting pentest.

## Scope

`0.5.0` adds no_std Hetzner security-resource request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, serde request/response
models, token storage, live API tests, retry policy, pagination iterators, or
action polling.

## Added

- `cloud_sdk_hetzner::security::ssh_keys` request domains for list, create,
  get, update, and delete.
- `cloud_sdk_hetzner::security::certificates` request domains for list,
  create, get, update, delete, and certificate retry action.
- Fixed-buffer path writers for SSH key and certificate `{id}` paths.
- Fixed-buffer list-query writers for source-locked pagination, filtering, and
  sorting parameters.
- Conservative validation for security resource names, SSH public keys,
  certificate PEM values, private key PEM values, managed certificate domain
  names, and label entry order.
- Redacted `Debug` output for SSH public keys, PEM values, and request structs
  containing secret-adjacent values.
- Tests for source-locked paths, query construction, missing required fields,
  invalid labels and inputs, redaction, managed certificate modes, and
  certificate retry action paths.
- `scripts/release_0_5_gate.sh`.

## Security Notes

- The default graph remains no_std and transport-free.
- Security resource IDs are nonzero.
- Request builders validate path output through the shared `EndpointPath`
  boundary after assembling ID paths.
- SSH public keys and PEM values never expose raw contents through `Debug`.
- Uploaded certificate requests require both certificate and private-key PEM
  values; managed certificate requests require at least one validated domain.
- The SDK still does not serialize request bodies or execute API requests.

## Verification

- `cargo fmt --all --check`
- `cargo clippy -p cloud-sdk-hetzner --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features security`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_5_gate.sh`

## Pentest

- Pending. Stop at the implementation commit and run pentest before release
  metadata is finalized.
