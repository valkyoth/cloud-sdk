# cloud-sdk 0.49.0 Release Notes

Status: implementation stop; pentest required before release.

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
- Rejected duplicate decoded keys across chunks and escaped-key spellings.
- Validated partial UTF-8, escapes, and surrogate pairs across every chunk boundary.
- Kept temporary keys and numbers in growth-aware protected storage and cleared fixed scratch.
- Redacted payload-bearing event, decoder, and visitor-error diagnostics.
- Kept status, content-type, operation binding, retry, transport, and input-buffer cleanup explicit.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.49.0` | facade metadata |
| `cloud-sdk-hetzner` | `0.37.0` | incremental decoder code |
| `cloud-sdk-reqwest` | `0.32.2` | dependency-only patch |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.28.1` | dependency-only patch |

## Documentation

- [`docs/INCREMENTAL_DECODING.md`](../docs/INCREMENTAL_DECODING.md)
- [`docs/MIGRATION_0.49.0.md`](../docs/MIGRATION_0.49.0.md)
- [`docs/PUBLIC_API_REVIEW_0.49.0.md`](../docs/PUBLIC_API_REVIEW_0.49.0.md)
- [`docs/DEPENDENCY_REVIEW_0.49.0.md`](../docs/DEPENDENCY_REVIEW_0.49.0.md)

## Pentest

No release tag may be created until the exact implementation commit passes
pentest and a permanent report is committed at
`security/pentest/v0.49.0.md`.

## Release Gate

```text
v0.49.0 implementation stop reached. Run pentest for this exact commit.
```
