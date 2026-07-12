# cloud-sdk 0.13.0 Release Notes

Status: implementation candidate; pentest pending.

## Scope

`0.13.0` adds no_std Hetzner DNS RRSet request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, complete request-body
serialization, response models, token storage, live API tests, retry policy,
pagination iterators, action polling, or type-specific RDATA normalization.

## Added

- RRSet list/create/get/update/delete request domains.
- Protection, TTL, set-records, add-records, remove-records, and
  update-record-comments action request domains.
- All 16 source-locked RR types: A, AAAA, CAA, CNAME, DS, HINFO, HTTPS, MX, NS,
  PTR, RP, SOA, SRV, SVCB, TLSA, and TXT.
- Relative lowercase, ACE, underscore, apex, and leading-wildcard RRSet names
  with percent-encoded path components.
- Deterministic list queries with exact name, repeated unique RR types, label
  selectors, pagination up to 100, and source-locked sorting.
- Bounded, debug-redacted record values and comments plus `1..=50`
  unique-value record and comment-update lists.
- Atomic JSON-string writers for record values and comments.
- `scripts/release_0_13_gate.sh`.

## Security Notes

- The default graph remains no_std, allocation-free, and transport-free.
- Every RRSet endpoint path is assembled in caller-owned buffers, percent
  encodes dynamic RR names, and validates the complete `EndpointPath`.
- The shared 1024-byte endpoint policy covers maximum structurally valid Zone
  and RRSet names plus the longest action suffix.
- Change-TTL requires explicit `RrsetTtl::Explicit` or
  `RrsetTtl::InheritZoneDefault`; deprecated omission is not representable.
- Create and add-records retain an outer optional TTL because their distinct
  source-locked schemas still permit omission.
- Record mutations reject empty lists, more than 50 entries, duplicate values,
  controls, and Unicode bidi controls.
- Create requests use the same conservative 50-record ceiling even though the
  source schema requires nonempty distinct values without publishing a number.
- Update-records structurally requires a comment for every identified value.
- Record values/comments do not expose raw string accessors; future body
  serializers must use the atomic JSON writers.
- Record values/comments are redacted from `Debug` output.
- The SDK validates structure but does not normalize every RR type's RDATA.
  Callers remain responsible for Hetzner-compatible record semantics.
- Duplicate detection compares exact record-value bytes. Callers requiring
  case-insensitive semantic uniqueness for domain-name RDATA must canonicalize
  those values before constructing records.
- The per-value and record-count bounds are not an aggregate transport-size
  guarantee. Future serializers and transports must enforce a separate current
  provider request-body limit before allocation or transmission.

## Version Plan

- `cloud-sdk` publishes as `0.13.0`.
- `cloud-sdk-hetzner` publishes its code release as `0.13.0`.
- `cloud-sdk-reqwest`, `cloud-sdk-sanitization`, and `cloud-sdk-testkit`
  publish dependency-only patch releases as `0.12.1`.
- Retired Hetzner-specific boundary packages remain rejected and unpublished.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features dns_rrsets`
- `scripts/test-release-readiness.sh`
- `scripts/test-hetzner-api-drift.py`
- `scripts/test-iana-ipv6-registry.py`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_13_gate.sh`
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
