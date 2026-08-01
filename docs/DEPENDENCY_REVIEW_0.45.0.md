# v0.45.0 Dependency Review

Date: 2026-08-01

Scope: provider-owned quota decoding and provider-neutral delay policy.

## Result

No dependency was added or removed. The exact rustls patch pin advances from
`0.23.42` to current stable `0.23.43`, whose declared Rust 1.71 requirement
remains below the workspace MSRV. HTTP-date parsing, calendar conversion,
quota storage, and delay decisions use only `core`.
The default graph remains allocation-free, `no_std`, transport-free,
runtime-free, clock-free, filesystem-free, and randomness-free.

## Root Lockfile Change Inventory

The release gate compares the root lockfile with `v0.44.0` and requires every
package-version change below to remain present in this review.

| Package | Previous | Current | Execution surface | Review |
| --- | --- | --- | --- | --- |
| `clang-sys` | `1.8.1` | `1.9.1` | native FIPS build dependency | Locked crates.io source; bundled AWS-LC build-policy tests, FIPS compile checks, Cargo Deny, and RustSec pass. |
| `cloud-sdk` | `0.44.0` | `0.45.0` | local runtime library | Current release source; workspace, no_std, MSRV, and public API review pass. |
| `cloud-sdk-hetzner` | `0.34.0` | `0.35.0` | local provider library | Current release source; provider, wire, checked-decoder, and API coverage tests pass. |
| `cloud-sdk-reqwest` | `0.30.1` | `0.31.0` | local transport library | Current release source; ordinary, deterministic-root, FIPS, and feature-unification checks pass. |
| `cloud-sdk-testkit` | `0.25.1` | `0.25.2` | local development library | Dependency-only update; package and complete testkit checks pass. |
| `displaydoc` | `0.2.6` | `0.2.7` | procedural macro | Locked crates.io source; Cargo Deny, RustSec, package builds, and complete workspace compilation pass. |
| `rustls` | `0.23.42` | `0.23.43` | optional TLS runtime | Exact patch pin; ordinary, deterministic-root, FIPS, feature-unification, Cargo Deny, and RustSec checks pass. |

`displaydoc 0.2.7` selects the already locked `syn 3.0.3` parser instead of
`syn 2.0.119` for that procedural-macro edge. Both versions were already in
the `v0.44.0` root lockfile, so this is an execution-edge change rather than
an additional root package-version change. It is included here because
procedural macros execute during compilation.

The optional Hetzner Serde decoder already uses `alloc`; it boxes retained
quota beside owned checked success and provider-error models to keep public
error sizes bounded. Boxing occurs immediately after provider decoding, and
the box moves directly through success and error decoding. The several-
kilobyte fixed-capacity aggregates are deliberately not `Copy`; read-only
accessors borrow them, and the quota gate denies Clippy's
`large_types_passed_by_value` lint. The provider-neutral quota types
themselves do not allocate.

`cloud-sdk-hetzner 0.35.0` and `cloud-sdk-reqwest 0.31.0` are code releases.
`cloud-sdk-testkit 0.25.2` is dependency-only. `cloud-sdk-sanitization 0.16.0`
is unchanged and excluded from publication.

## Required Verification

- default and all-feature no_std checks;
- HTTP-date, duration, timestamp, rollback, conflict, maximum, duplicate,
  partial, overflow, extension, and multi-bucket tests;
- checked Hetzner success/error quota retention;
- dedicated quota/Retry-After fuzz target with deterministic seed coverage;
- transport boundary checks proving reqwest does not decode provider quota;
- exact root lockfile change inventory against `v0.44.0`;
- workspace tests, Clippy, docs, package, platform, MSRV, SBOM, Cargo Deny,
  RustSec, and `scripts/release_0_45_gate.sh` after pentest evidence is committed.
