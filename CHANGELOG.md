# Changelog

## Unreleased

- Added crate-local README and rustdoc entry-point documentation for every
  workspace crate.
- Clarified that `cloud-sdk-hetzner` is the main Hetzner-specific documentation
  surface while `cloud-sdk` remains the provider-neutral foundation.
- Added the first `v0.3.0` core request/response policy domains in
  `cloud-sdk-hetzner`: endpoint paths, base URL selection, bounded query
  parameters, labels, pagination, sorting, action status, API errors, and
  rate-limit metadata.
- Added fixed-buffer query percent encoding, endpoint group base URL mapping,
  and stricter label selector structure checks for `v0.3.0`.
- Added the `v0.3.0` release gate script.

## 0.2.0 - 2026-07-08

- Source-locked the official Hetzner Cloud/DNS and Storage Box OpenAPI specs.
- Replaced the initial group-level API plan with a complete 221-operation
  endpoint matrix, including pagination, sorting, action, deprecation, and owner
  module tracking.
- Added local Hetzner upstream lock validation and a `v0.2.0` release gate.
- Added Hetzner API drift fingerprints and a drift detector for upstream
  operation and schema changes.
- Expanded the release plan so every milestone has concrete deliverables,
  verification commands, and an explicit pentest stop gate.
- Hardened the API drift lock refresh path to require explicit acknowledgement
  and pinned SHA-256 verification before overwriting fingerprint locks.
- Documented current Hetzner changelog items that affect future SDK models,
  including deprecated omitted DNS pointer and RRSet TTL fields, deprecated
  datacenter endpoints, and deprecated resource-local action lookups.

## 0.1.0 - 2026-07-08

- Initialized the `cloud-sdk` Rust workspace.
- Added `cloud-sdk` as the provider-neutral crate.
- Added `cloud-sdk-hetzner` as the first provider crate.
- Added one no_std SDK crate with internal Cloud, DNS, security, and Storage Box
  modules.
- Added placeholder crates for future reqwest transport, testkit, and
  sanitization boundaries.
- Added MIT OR Apache-2.0 licensing, security policy, dependency policy, CI
  metadata, and release planning.
- Added local checks for formatting, linting, tests, no_std policy, modularity,
  shell syntax, security policy, and file length.
- Hardened release gates for pentest evidence, no_std policy validation, and
  required dependency security tools.
- Configured CI checkout with full history so pentest reviewed-commit ancestry
  checks work on GitHub Actions.
