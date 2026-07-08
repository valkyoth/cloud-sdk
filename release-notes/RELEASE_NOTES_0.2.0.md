# cloud-sdk 0.2.0 Release Notes

Status: implementation and retest complete; permanent pentest report pending.

## Scope

`0.2.0` is the official Hetzner API source-lock release. It does not add HTTP
transport, serde models, endpoint builders, token storage, or live API tests.

## Added

- Pinned the official Hetzner Cloud/DNS OpenAPI spec:
  <https://docs.hetzner.cloud/cloud.spec.json>
- Pinned the official Hetzner Storage Box OpenAPI spec:
  <https://docs.hetzner.cloud/hetzner.spec.json>
- Added a complete 221-operation API matrix with owner modules, pagination,
  sorting, action behavior, deprecation state, and implementation status.
- Added local upstream lock validation through
  `scripts/check_hetzner_upstream.sh --local-only`.
- Added API drift detection through `scripts/check_hetzner_api_drift.py`.
- Added `scripts/release_0_2_gate.sh`.
- Expanded `docs/RELEASE_PLAN.md` with concrete per-version deliverables,
  verification commands, and pentest stop gates.

## Security Notes

- Deprecated operations are retained in the matrix for drift tracking and marked
  `deferred-deprecated`.
- Resource-local action lookup deprecations are documented before endpoint model
  work begins.
- Current Hetzner DNS pointer and RRSet TTL omission deprecations are recorded
  so future request models can avoid accepting unsafe implicit omission.
- Upstream API drift checks compare operation and schema fingerprints while
  ignoring prose-only OpenAPI fields.
- The lock refresh path requires explicit `--accept-lock-refresh` acknowledgement
  and verifies fetched spec bytes against pinned SHA-256 values before writing
  fingerprint files.

## Verification

- `scripts/check_hetzner_api_drift.py --local-only`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/check_hetzner_api_drift.py --fetch --write-lock`
  fails without `--accept-lock-refresh`.
- `scripts/check_hetzner_api_drift.py --fetch --write-lock --accept-lock-refresh`
- `git diff --check`
- `scripts/checks.sh`
- `scripts/release_0_2_gate.sh`

## Pentest

- Retest passed. Permanent report will be added as
  `security/pentest/v0.2.0.md` before tagging.

## Versioning

- `cloud-sdk` publishes as `0.2.0`.
- `cloud-sdk-hetzner` publishes as `0.2.0`.
- `cloud-sdk-hetzner-reqwest` publishes as `0.2.0`.
- `cloud-sdk-hetzner-sanitization` publishes as `0.2.0`.
- `cloud-sdk-hetzner-testkit` publishes as `0.2.0`.
