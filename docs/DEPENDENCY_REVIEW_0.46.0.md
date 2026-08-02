# v0.46.0 Dependency Review

Date: 2026-08-02

Scope: provider-neutral retry, fingerprint, and idempotency contracts.

## Result

No third-party dependency was added or removed. The transitive `ipnet` patch
was updated after the final registry freshness review. Canonical encoding,
constant-time exact comparison, monotonic budgets, and retry state use `core`.
Existing `cloud-sdk-sanitization` primitives clear canonical, digest, and
intent storage. The test-only SHA-256 known vector is an internal checked
reference implementation and does not enter any dependency graph.

The default graph remains allocation-free, `no_std`, transport-free,
runtime-free, clock-free, filesystem-free, randomness-free, and cryptographic-
implementation-free.

## Third-Party Version Changes

| Package | Previous | Current | Review |
| --- | --- | --- | --- |
| `ipnet` | `2.12.0` | `2.12.1` | Transitive patch used by optional transport graphs; no default-core dependency. MIT OR Apache-2.0; no features or policy boundaries changed. |

Registry checks on 2026-08-02 confirmed every direct workspace dependency is
at its latest stable release. The registry advertises `rustls 0.24.0-dev.1`,
which is a prerelease and is intentionally excluded. Pinned Cargo security and
fuzz tools are current. `cargo update --workspace --dry-run --verbose` reports
no remaining compatible update after explicitly advancing `ipnet`.

## Local Package Changes

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.45.0` | `0.46.0` | Retry, fingerprint, intent, and replayability code. |
| `cloud-sdk-hetzner` | `0.35.0` | `0.36.0` | Immutable prepared bodies marked replayable. |
| `cloud-sdk-reqwest` | `0.31.0` | `0.31.1` | Dependency-only; no retry owner added. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | Unchanged and not published. |
| `cloud-sdk-testkit` | `0.25.2` | `0.26.0` | Replayability policy recording. |

## Required Verification

- canonical domain, version, tags, lengths, exact fields, SHA-256 vector, and
  digest-length rejection;
- complete scratch and digest cleanup plus redacted diagnostics;
- fresh-intent bounds, non-cloneability, and fingerprint mismatch rejection;
- every delivery phase, transient status, body replayability, mutation rule,
  attempt bound, cumulative-delay bound, overflow, and monotonic rollback;
- complete Hetzner operation metadata and replayability coverage;
- default/all-feature no_std, workspace, doctest, Clippy, platform, MSRV,
  package, SBOM, Cargo Deny, RustSec, fuzz, and v0.46 release-gate evidence.
