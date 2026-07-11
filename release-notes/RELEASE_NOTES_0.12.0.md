# cloud-sdk 0.12.0 Release Notes

Status: release candidate; pentest and retest passed.

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
  hardened TSIG keys.
- Structural primary and secondary Zone creation modes that reject a zonefile
  on secondary Zones.
- Shared conservative public-IP validation for Load Balancer targets and DNS
  primary nameservers.
- Replaced the unused Hetzner-specific sanitization placeholder with the new
  provider-neutral `cloud-sdk-sanitization` boundary.
- Replaced the unused Hetzner-specific reqwest and testkit placeholders with
  provider-neutral `cloud-sdk-reqwest` and `cloud-sdk-testkit` boundaries.
- Added release-script regression guards that reject all retired
  Hetzner-specific boundary names in publish order, workspace metadata,
  release plans, and direct publish requests.
- Added a general one-crate-per-provider guard that rejects nested package names
  such as `cloud-sdk-ovh-dns` and `cloud-sdk-scaleway-reqwest` at every release
  boundary.
- `scripts/release_0_12_gate.sh`.

## Security Notes

- The default graph remains no_std, allocation-free, and transport-free.
- Endpoint paths are assembled in caller-owned buffers and validated through
  `EndpointPath`.
- Zone names admit only lowercase ASCII or ACE labels and reject ambiguous
  leading, trailing, empty, or overlong labels.
- Zonefiles are nonempty, NUL-free, capped at 1 MiB, redacted in `Debug`, and
  exposed only through an atomic JSON-string writer.
- TSIG keys are bounded, canonical padded standard Base64 values with zero
  unused padding bits, at least 32 decoded bytes, redacted in `Debug`, and
  exposed only through an atomic JSON-string writer.
- The TSIG API exposes only HMAC-SHA256. HMAC-MD5 and HMAC-SHA1 are
  intentionally unavailable under the hardened local policy.
- TSIG secrets must be CSPRNG-generated, shared by only two entities, and
  rotated; validation can enforce representation and size but not entropy.
- Zone files, TSIG keys, credentials, nameservers carrying credentials, and
  containing requests do not implement ordinary variable-time equality.
- Callers must securely erase the complete caller-owned buffer used by TSIG or
  zonefile JSON writers after transport completes. The SDK cannot erase memory
  it does not own; use a reviewed non-elidable mechanism for the target.
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
- `scripts/test-release-readiness.sh`
- `scripts/release_0_12_gate.sh`
- `git diff --check`

## Pentest

Pentest and retest passed. Permanent evidence is published at
`security/pentest/v0.12.0.md` as the only change in the direct child of the
reviewed release commit, matching the `eth` workspace release model.

## Publishing Plan

- `cloud-sdk` publishes as `0.12.0`.
- `cloud-sdk-hetzner` publishes as `0.12.0`.
- `cloud-sdk-reqwest` publishes for the first time as `0.12.0`.
- `cloud-sdk-sanitization` publishes for the first time as `0.12.0`.
- `cloud-sdk-testkit` publishes for the first time as `0.12.0`.
