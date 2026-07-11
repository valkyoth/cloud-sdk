# cloud-sdk 0.12.0 Release Notes

Status: implementation candidate; pentest pending.

## Scope

`0.12.0` adds no_std Hetzner DNS Zone request primitives in
`cloud-sdk-hetzner`. It does not add RRSet models, HTTP transport, request-body
serialization, response models, token storage, live API tests, retry policy,
pagination iterators, or action polling.

## Added

- Zone list/create/get/update/delete and zonefile export request domains.
- Global and per-Zone action lists plus global action lookup.
- Zonefile import, primary nameserver replacement, deletion protection, and
  explicit TTL-change action models.
- Deterministic Zone and action list queries with source-locked filters and
  sorting.
- Bounded lowercase Zone names, TTLs, zonefiles, primary nameserver lists, and
  strict padded Base64 TSIG keys.
- Structural primary and secondary Zone creation modes that reject a zonefile
  on secondary Zones.
- Shared conservative public-IP validation for Load Balancer targets and DNS
  primary nameservers.
- `scripts/release_0_12_gate.sh`.

## Security Notes

- The default graph remains no_std, allocation-free, and transport-free.
- Endpoint paths are assembled in caller-owned buffers and validated through
  `EndpointPath`.
- Zone names admit only lowercase ASCII or ACE labels and reject ambiguous
  leading, trailing, empty, or overlong labels.
- Zonefiles are nonempty, NUL-free, capped at 1 MiB, redacted in `Debug`, and
  exposed only through an atomic JSON-string writer.
- TSIG keys are bounded, strict padded standard Base64 values, redacted in
  `Debug`, and exposed only through an atomic JSON-string writer.
- Primary nameservers require unique public IP addresses, a nonzero port, and
  coherent optional TSIG credentials.
- Zone change-TTL requests require an explicit `60..=2147483647` value. Zone
  creation retains optional TTL intent because omission remains valid in the
  source-locked schema.
- The deprecated resource-local action lookup endpoint remains intentionally
  deferred.
- Initial RRSet creation and all RRSet operations remain deferred to v0.13.0.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features dns_zones`
- `scripts/test-hetzner-api-drift.py`
- `scripts/test-iana-ipv6-registry.py`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_12_gate.sh`
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. Permanent evidence will be
published at `security/pentest/v0.12.0.md` only after the finalized
release-sensitive commit passes.

## Publishing Plan

- `cloud-sdk` publishes as `0.12.0`.
- `cloud-sdk-hetzner` publishes as `0.12.0`.
- `cloud-sdk-hetzner-reqwest` publishes as `0.12.0`.
- `cloud-sdk-hetzner-sanitization` publishes as `0.12.0`.
- `cloud-sdk-hetzner-testkit` publishes as `0.12.0`.
