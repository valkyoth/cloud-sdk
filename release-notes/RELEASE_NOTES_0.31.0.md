# cloud-sdk 0.31.0 Release Notes

Status: implementation complete; pentest and final release checks remain
required before tagging.

Release date: 2026-07-16

## Overview

v0.31 adds checked response decoding for all 208 source-locked,
non-deprecated Hetzner Cloud, DNS, and Console Storage operations. The decoder
consumes the exact prepared request together with the transport response, so
status, content type, body shape, operation identity, API family, and maximum
size cannot be checked independently or accidentally omitted.

## Provider-Neutral Foundation

- `PreparedRequest` can carry a bounded validated static `OperationId`.
- `PreparedRequest::validate_response` applies its complete response policy
  without executing a transport.
- Existing direct constructors remain source compatible and leave the optional
  operation identifier absent.

## Hetzner Checked Decoder

- Every active operation is bound to one exact success status, response family,
  root key, and required top-level field set generated from the pinned official
  OpenAPI documents.
- `decode_response` returns typed action, resource identity, resource-list,
  composite, metrics, zonefile, pricing, folder, or empty success values.
- 4xx and 5xx JSON envelopes return a classified typed `HetznerApiError` with
  payload-free `Display` and redacted `Debug`.
- Resource identifiers, source-known statuses, action state, pagination,
  metrics pairs, text bounds, and special envelope fields are validated before
  crossing the public model boundary.
- Secret-bearing fields and zonefiles require explicit accessors and remain
  redacted from diagnostics. Parser-owned secret strings move without a
  plaintext copy into the reviewed `sanitization::SecretString`, which clears
  full allocation capacity on drop and exposes UTF-8 only through checked
  closures. Cloned response models share the protected allocation until the
  final clone drops.

## Parser And Supply Chain

- The non-default `serde` feature now admits serde_json `1.0.150` with default
  features disabled and `alloc` only; the default graph remains unchanged.
- A private bounded parser rejects duplicate keys, trailing documents,
  excessive nesting, oversized strings, and oversized arrays or objects.
- The fetched Hetzner drift gate regenerates the response-operation table in
  memory and rejects stale committed response evidence.

## Verification

- One generated minimal success fixture is decoded for each of the 208 active
  operations.
- Golden tests cover all twelve response families and typed provider errors.
- Adversarial tests cover duplicate keys, unknown statuses, service mismatch,
  wrong success status, malformed payloads, Unicode format controls, diagnostic
  redaction, oversized integers, deep nesting, and invalid UTF-8.
- A ninth isolated fuzz target drives prepared-policy, content-type, status,
  typed success/error, and malformed-payload paths through the checked decoder.
- `scripts/check_response_operation_coverage.py` proves exact equality with the
  active API matrix.
- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/release_0_31_gate.sh` after the permanent pentest report is added.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.31.0` | Operation identity and checked-policy foundation |
| `cloud-sdk-hetzner` | `0.24.0` | Operation-complete checked response decoding |
| `cloud-sdk-reqwest` | `0.20.2` | Dependency-only patch |
| `cloud-sdk-sanitization` | `0.14.0` | Owned volatile-clearing UTF-8 secret storage |
| `cloud-sdk-testkit` | `0.18.2` | Dependency-only patch |

## Security Review

Run the pentest against the exact implementation-stop commit. Add the permanent
PASS report only after findings are resolved and the retest is green.

## Release Gate

```text
v0.31.0 implementation stop reached. Run pentest for this exact commit.
```
