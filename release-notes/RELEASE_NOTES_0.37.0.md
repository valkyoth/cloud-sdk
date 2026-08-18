# cloud-sdk 0.37.0 Release Notes

Status: release candidate; pentest and final retest passed. Local and GitHub
release checks remain required before tagging.

Release date: 2026-07-28

## Overview

v0.37 makes response-body provenance structural. A transport receives only a
sealed writer tied to the admitted caller-buffer prefix; core alone constructs
response views and keeps the cleanup owner alive through policy checking and
decoding.

## Response Provenance

- Added cleanup-owning `ResponseBuffer`.
- Added sealed `ResponseWriter` with exclusive admitted-prefix access.
- Added one explicit commit for status, initialized length, and bounded
  metadata.
- Rejected oversized lengths, duplicate commits, post-commit writes, and
  successful transport returns without commitment.
- Rejected precommitted writers before adapter I/O or testkit cursor movement.
- Removed public response construction from unrelated or static slices.

## Checked Lifetimes And Cleanup

- Added `CheckedResponseGuard` as the prepared execution result.
- Added higher-ranked closure-scoped borrowed inspection.
- Added owned decoding that drops and sanitizes response storage before return.
- Sanitized complete caller storage before transport admission and on ordinary
  success, rejection, error, early return, and cancellation drop paths.
- Kept the sanitizer outside suspended async writer state for sequential
  non-`Sync` transport compatibility.

## Workspace Migration

- Migrated blocking and async transport traits.
- Migrated reqwest adapters and added explicit commit failure diagnostics.
- Migrated deterministic testkit fixtures and records.
- Migrated all 208 Hetzner checked-operation response bindings.
- Migrated live smoke, public examples, doctests, and checked-response fuzzing.
- Added adversarial provenance, cleanup, parity, and compile-fail tests.
- Bound the external-construction fixture to the compiler's private-field
  diagnostic and rejected unrelated missing-field failures.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.37.0` | response provenance and checked guard |
| `cloud-sdk-hetzner` | `0.30.0` | checked decoder migration |
| `cloud-sdk-reqwest` | `0.25.0` | sealed writer integration |
| `cloud-sdk-sanitization` | `0.15.5` | dependency-only patch |
| `cloud-sdk-testkit` | `0.22.0` | sealed writer fixtures |

## Security Boundary

v0.37 proves response bytes come from the admitted caller storage and gives
ordinary-path cleanup one structural owner. v0.38 remains responsible for the
single audited non-elidable core primitive, retained-sensitive-metadata
transfer, and stronger platform lifecycle evidence.

## Verification

- `scripts/check_response_provenance.sh`
- `scripts/checks.sh`
- `scripts/release_0_37_gate.sh` after pentest evidence is committed
- default, no_std, all-feature, clippy, doctest, package, deny, audit, platform,
  MSRV, fuzz, and SBOM gates

## Pentest

The v0.37 pentest found one Low verification gap: the response-construction
compile-fail example could fail because fields were missing before field
privacy was tested. The fixture now supplies every field, and the provenance
gate requires the private-field diagnostic while rejecting missing-field
failures.

The final retest passed commit
`07a78481d78279bb95f0ca9d02ed3d6da170cea9`. See the
[`v0.37.0` pentest report](../security/pentest/v0.37.0.md).

## Migration

See [`docs/MIGRATION.md#v0370`](../docs/MIGRATION.md#v0370),
[`docs/PUBLIC_API_REVIEW.md#v0370`](../docs/PUBLIC_API_REVIEW.md#v0370), and
[`docs/DEPENDENCY_REVIEW.md#v0370`](../docs/DEPENDENCY_REVIEW.md#v0370).

## Release Gate

```text
v0.37.0 pentest stop passed. Tag only after the clean local release gate and
GitHub checks pass on the final release commit.
```
