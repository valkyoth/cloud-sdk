# cloud-sdk 0.28.0 Release Notes

Release date: 2026-07-15

## Overview

`0.28.0` makes the provider-neutral blocking and async transport contracts
shareable without requiring callers to hold a mutex across I/O or `.await`.
The optional reqwest clients become cloneable shared handles with immutable
endpoint identity, source-clearing token ingestion, and explicit atomic token
rotation.

The default workspace remains `no_std`, transport-free, runtime-free, and free
of new third-party dependencies.

## Transport Contract

- `BlockingTransport::send` and `AsyncTransport::send` now use `&self`.
- `BoundTransport` reports a validated credential-free `EndpointIdentity` with
  scheme, normalized host, effective port, and normalized base path.
- Concurrency remains entirely caller-bounded. The SDK creates no task set,
  queue, semaphore, retry fan-out, sleep, clock, or executor.
- The no-std ordered mock preserves lazy cancellation behavior and uses an
  atomic cursor with a distinct concurrent-race error.

## Credential Lifecycle

- Blocking and async reqwest clients implement `Clone + Send + Sync` and share
  one immutable endpoint plus credential state.
- `BearerToken::from_mut_bytes` and guarded-buffer ingestion clear complete
  source storage on success or rejection.
- Client rotation validates a complete replacement before changing state;
  rejected input leaves the current token active.
- Requests take a short-lived token snapshot and release the state lock before
  network I/O or `.await`.
- In-flight requests retain their prior snapshot. Retired adapter-owned storage
  is sanitized after the final snapshot drops.
- Poisoned credential locks recover while guarded complete-token state remains
  structurally valid, preventing permanent failure across all client clones.
- Immutable strings and copies owned by reqwest, TLS, the operating system, or
  remote services remain documented caller and deployment boundaries.

## Security Evidence

- Endpoint identity tests cover host, subdomain, scheme, port, base-path, and
  normalization differences.
- `cloud-sdk-hetzner::verify_official_endpoint` checks both official v1 API
  authorities without coupling the provider crate to a transport adapter. It
  derives its expected identity from the canonical provider base-URL constants,
  and regression tests reject malformed or divergent canonical forms.
- Blocking and async tests issue overlapping requests through one shared client
  and keep response buffers isolated.
- Blocking and async in-flight tests prove rotation changes only newly started
  requests.
- Mutable and guarded source tests cover successful and rejected cleanup,
  rejected-rotation rollback, clone visibility, and retired-token drop timing.
- Existing redirect, proxy, retry, timeout, response-bound, cancellation, TLS,
  FIPS, feature-unification, no-std, API drift, SBOM, and supply-chain gates
  remain required.

## Independent Crate Versions

- `cloud-sdk` publishes code release `0.28.0`.
- `cloud-sdk-hetzner` publishes code release `0.22.0`.
- `cloud-sdk-reqwest` publishes code release `0.19.0`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.14`.
- `cloud-sdk-testkit` publishes code release `0.17.0`.

No retired provider-specific helper crate is present in the publish plan.

## Migration

See [`docs/MIGRATION_0.28.0.md`](../docs/MIGRATION_0.28.0.md) for receiver,
endpoint-identity, token-rotation, and testkit changes.

## Verification

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-targets --all-features`
- `scripts/checks.sh`
- `scripts/release_crates.py --check`
- `scripts/test-release-crates.py`
- `scripts/validate-file-lengths.sh`
- `scripts/check_sbom_freshness.sh`

- `scripts/release_0_28_gate.sh`
- `scripts/release_crates.py --dry-run --yes --version 0.28.0`

## Security Review

Iterative pentest and retest reviewed commit
`fc3765fdc3b2b1aef5efc950f643622317476e64`. Review identified two Low
findings and one informational defense-in-depth improvement. Credential lock
poisoning now recovers safely, the Hetzner provider supplies exact official
endpoint verification, and that verifier derives from canonical base-URL
constants instead of duplicated destination literals. The final retest is
green and the permanent report records `PASS`; tagging remains blocked until
the clean release gate, GitHub CI, and CodeQL default setup are green.
