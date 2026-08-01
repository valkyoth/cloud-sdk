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

The optional Hetzner Serde decoder already uses `alloc`; it boxes retained
quota beside owned checked success and provider-error models to keep public
error sizes bounded. The provider-neutral quota types themselves do not
allocate.

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
- workspace tests, Clippy, docs, package, platform, MSRV, SBOM, Cargo Deny,
  RustSec, and `scripts/release_0_45_gate.sh` after pentest evidence is committed.
