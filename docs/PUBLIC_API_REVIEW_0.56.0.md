# v0.56.0 Public API Review

Date: 2026-08-05

Scope: provider-generic API drift evidence and release-process contracts.

## Rust API

v0.56.0 adds no Rust public API and changes no provider, transport, client,
sanitization, or testkit runtime behavior. The `cloud-sdk` source package moves
to 0.56.0 for the repository tag while crates.io publication remains at 0.55.0
until the v0.60.0 checkpoint.

## Tooling Contract

The new stable tooling inputs are strict JSON plugin, provider-lock, and
observation documents. Plugins are declarative data and cannot select or load
code. Provider adapters remain explicit repository-reviewed scripts.

Canonical reports identify category, row, change kind, changed field paths,
old and new canonical SHA-256 values, owner, and compatibility severity. They
never include source text or normalized field values.

Every release after v0.55.0 now carries an incremental pentest report bound to
the immediately preceding tag. Intermediate tags continue to select no crates
for publication; v0.60.0 is the next scheduled crates.io checkpoint.

## Security Review

All local documents are bounded, strict, no-follow regular files. Remote
sources require exact credential-free HTTPS URLs, default certificate and
hostname validation, redirect denial, size/time bounds, and full digest
authentication before adapter use. Drift output is deterministic and
payload-free.
