# cloud-sdk 0.21.0 Release Notes

Status: implementation stop reached; pentest and retest required before tagging.

## Overview

`0.21.0` turns the existing request, transport, pagination, action, DNS, and
Storage Box APIs into workflow-oriented, compile-checked documentation. It also
makes docs.rs feature coverage and local documentation validation explicit for
every published crate.

No provider API behavior, transport behavior, default feature, or third-party
dependency is added by this release.

## Executable Examples

- Provider-neutral transport request construction.
- Hetzner read-only public-image catalog listing.
- Validated server mutation request construction without execution.
- Strict pagination-envelope decoding and cursor observation.
- Strict action-envelope decoding with caller-owned polling policy.
- DNS Zone create request construction without execution.
- Read-only Storage Box listing.

The examples perform no network I/O and read no credentials. Mutation examples
stop before transport execution.

## Documentation And Security

- Added a [provider-neutral quickstart](../docs/QUICKSTART.md).
- Added a [Hetzner workflow index](../docs/HETZNER_EXAMPLES.md).
- Added [security recipes](../docs/SECURITY_RECIPES.md) for token handling,
  logging, timeouts, retries, action polling, live smoke tests, and incident
  response.
- Added a concise [release runbook](../docs/RELEASE_RUNBOOK.md) aligned with the
  existing pentest parent/report evidence model.
- Added complete feature tables and docs.rs all-feature metadata to every
  crate.

The local link checker uses only Python's standard library, ignores fenced
code examples and external URLs, validates every local Markdown and HTML link
on a line, rejects missing targets, and rejects paths escaping the repository.
Regression tests cover each security-relevant branch.

The final lockfiles refresh existing transitive dependencies to
`simd_cesu8 1.2.0` and `socket2 0.6.5`. The all-target transport boundary
continues to keep broad Windows dependency ranges on `windows-sys 0.61`; only
the target-specific `ring` dependency retains its required `0.52` line.

## Version Plan

- `cloud-sdk` publishes code/documentation release `0.21.0`.
- `cloud-sdk-hetzner` publishes code/example release `0.18.0`.
- `cloud-sdk-reqwest` publishes dependency-only patch `0.15.3`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.7`.
- `cloud-sdk-testkit` publishes dependency-only patch `0.15.3`.
- Retired and provider-scoped helper packages remain rejected and unpublished.

## Verification

- `scripts/checks.sh`
- `scripts/check_doc_links.sh`
- `scripts/test-doc-links.py`
- `cargo test --workspace --doc --all-features`
- `cargo test --workspace --all-features`
- `scripts/check_platform_matrix.sh --all`
- `scripts/check_rust_version_matrix.sh`
- `scripts/check_hetzner_upstream.sh --local-only`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_sbom_freshness.sh`
- `scripts/release_crates.py --dry-run --yes --version 0.21.0`
- `cargo deny check`
- fixture-scoped advisory, license, and source checks
- workspace and fixture RustSec audits
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
