# cloud-sdk 0.49.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: pending

## Overview

v0.49 adds bounded incremental JSON decoding to the Hetzner provider so large
responses can be processed across arbitrary chunks without constructing one
complete JSON tree. Existing buffered checked decoding remains unchanged.

## Incremental Decoding

- Added borrowed object, array, key, string-fragment, number, Boolean, and null events.
- Added explicit pending, stopped, complete, and terminal-failure lifecycle states.
- Added hard and caller-lowerable limits for input, nesting, tokens, aggregate fields,
  per-object fields, strings, number tokens, and exponent digits.
- Preserved finite-number admission from the buffered checked decoder.
- Rejected duplicate decoded keys across chunks and escaped-key spellings.
- Validated partial UTF-8, escapes, and surrogate pairs across every chunk boundary.
- Kept temporary keys and numbers in growth-aware protected storage and cleared fixed scratch.
- Made parser-owned frame, key, number, and duplicate-key allocation fallible.
- Poisoned the decoder across visitor panic and immediately cleared staging on visitor stop.
- Guarded UTF-8 and character scratch across normal return and unwind.
- Added an independent `serde_json` validity oracle to the incremental fuzz target.
- Migrated incremental fuzz seeds to the two-control-byte wire format and added deterministic valid/duplicate preflight tests.
- Redacted payload-bearing event, decoder, and visitor-error diagnostics.
- Kept status, content-type, operation binding, retry, transport, and input-buffer cleanup explicit.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.49.0` | facade metadata |
| `cloud-sdk-hetzner` | `0.37.0` | incremental decoder code |
| `cloud-sdk-reqwest` | `0.32.2` | dependency-only patch |
| `cloud-sdk-sanitization` | `0.17.0` | fallible protected-string growth code |
| `cloud-sdk-testkit` | `0.28.1` | dependency-only patch |

## Documentation

- [`docs/INCREMENTAL_DECODING.md`](../docs/INCREMENTAL_DECODING.md)
- [`docs/MIGRATION.md#v0490`](../docs/MIGRATION.md#v0490)
- [`docs/PUBLIC_API_REVIEW.md#v0490`](../docs/PUBLIC_API_REVIEW.md#v0490)
- [`docs/DEPENDENCY_REVIEW.md#v0490`](../docs/DEPENDENCY_REVIEW.md#v0490)

## Pentest

Pentest and final retest passed. The permanent report is committed at
[`security/pentest/v0.49.0.md`](../security/pentest/v0.49.0.md).

## Release Gate

v0.49.0 release candidate. Tag only after the clean local release gate and
GitHub CI and CodeQL default setup pass on the final release commit.
