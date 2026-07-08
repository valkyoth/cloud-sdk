# Changelog

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
