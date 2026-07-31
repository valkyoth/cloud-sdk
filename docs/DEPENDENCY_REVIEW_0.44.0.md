# v0.44.0 Dependency Review

Date: 2026-07-31

Scope: provider-neutral pagination strategy separation.

## Result

No direct dependency was added, removed, or version-changed. The implementation
uses core arithmetic and the existing `cloud-sdk-sanitization` caller-buffer
owner. It adds no allocation, hash implementation, network client, TLS stack,
runtime, filesystem, clock, randomness, or OS dependency.

Cursor digests remain caller-provided. This avoids selecting cryptography or a
non-cryptographic `Hash` implementation in the default graph. Exact cursor
bytes are retained beside each digest so collision or digest inconsistency
fails closed. Snapshot identifiers likewise retain exact bounded bytes and do
not require a hashing dependency.

`cloud-sdk-reqwest 0.30.1` and `cloud-sdk-testkit 0.25.1` are dependency-only
patch releases. `cloud-sdk-sanitization 0.16.0` is unchanged and is excluded
from publication.

## Required Verification

- default and all-feature `no_std` checks;
- pagination strategy, cleanup, compile-fail, DigitalOcean fixture, and fuzz
  checks, including dedicated offset, opaque-state, cursor-history, and raw
  provider-link parser targets plus a deterministic positive-seed preflight;
- workspace tests, Clippy, docs, package, platform, MSRV, SBOM, Cargo Deny,
  and RustSec checks;
- `scripts/check_pagination_strategies.sh`;
- `scripts/release_0_44_gate.sh` after pentest evidence is committed.
