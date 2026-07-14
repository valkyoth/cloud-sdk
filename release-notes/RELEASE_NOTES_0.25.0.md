# cloud-sdk 0.25.0 Release Notes

Release date: 2026-07-14

## Summary

`0.25.0` turns the existing Hetzner source lock into a recurring, actionable
maintenance process. It does not change published Rust APIs or the default
dependency graph. The facade and provider remain transport-free and no_std by
default.

## Grouped Drift Reports

The detector now reports added, removed, newly deprecated, and changed
operations separately from schema-only changes. Raw source-digest changes are
also explicit. Output is deterministic, control-safe, and bounded to 50 entries
per category, with omitted counts. Duplicate identities, malformed lock rows,
and malformed indexed OpenAPI structures fail closed with bounded diagnostics.

Prose-only OpenAPI fields remain excluded from semantic fingerprints. The
separately indexed deprecation flag is now excluded from that fingerprint. A
deprecation-only transition is not duplicated as a generic operation change;
if the operation contract also changes, it appears in both relevant groups so
reviewers do not lose the additional signal. Refreshed TSV locks now use
portable LF endings instead of Python's platform-oriented CSV default.

## Acquisition And Trust Boundary

Live acquisition still accepts only the two exact official Hetzner HTTPS URLs,
rejects redirects, uses certificate and hostname validation, bounds connection
and total time, and caps each document at 32 MiB. Documents must be valid UTF-8
JSON objects.

Previously, any new raw digest stopped execution before the detector could
explain the semantic change. Live fetched documents may now be parsed only to
produce the drift report. A changed digest still fails the command and cannot
update pins, fingerprints, source, or packages automatically. Fingerprint
refresh additionally requires the fetched digests to match the reviewed pins.
Caller-supplied local documents continue to require the reviewed SHA-256 before
JSON parsing.

## Maintenance Automation

A read-only GitHub workflow runs the live comparison weekly and supports manual
dispatch. It has only `contents: read`, never invokes lock-refresh flags, and
cannot create commits or pull requests. Release gates retain the same live
comparison.

The new maintenance runbook defines required review for every category and an
explicit accept, reject, or defer decision. An accepted change requires a
complete old/new source diff, coordinated source pins and fingerprints, API
matrix and SDK work, release notes, tests, and security evidence. A separate
template records upstream digests, disposition, changelog evidence, security
and cost impact, and regression coverage.

## Fixture Evidence

Four checked-in synthetic OpenAPI fixture specifications model stable,
added, removed, deprecated, changed, and schema-only behavior across both
official API documents. Twenty deterministic tests cover report grouping,
prose filtering, source-digest handling, duplicate identities, secure local
reads, bounded network reads, redirects, timeouts, and the read-only workflow.

## Independent Crate Versions

- `cloud-sdk` publishes metadata release `0.25.0`.
- `cloud-sdk-hetzner` publishes dependency-only patch `0.19.3`.
- `cloud-sdk-reqwest` publishes dependency-only patch `0.17.1`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.11`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.15.7`.

No retired provider-specific helper crate is published.

## Verification

- `scripts/checks.sh`
- `scripts/test-hetzner-api-drift.py`
- `scripts/check_hetzner_api_drift.py --local-only`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/check_latest_tools.sh --fetch`
- `scripts/check_platform_matrix.sh --all`
- `scripts/check_rust_version_matrix.sh`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_fuzz_harness.sh --build`
- `scripts/check_fuzz_harness.sh --smoke`
- `scripts/check_sbom_freshness.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/release_crates.py --dry-run --yes --version 0.25.0`
- `scripts/release_0_25_gate.sh` after pentest evidence is committed.

## Security Review

The v0.25 pentest and retest passed with no security finding requiring
remediation. Two non-vulnerability observations confirmed the bounded Python
socket timeout behavior and the read-only workflow's fail-closed upstream trust
boundary. The permanent report is stored at
`security/pentest/v0.25.0.md`. Tagging remains blocked until the full release
checks, GitHub CI, CodeQL default setup, and clean versioned release gate pass.
