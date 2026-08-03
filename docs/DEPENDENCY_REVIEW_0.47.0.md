# v0.47.0 Dependency Review

Date: 2026-08-03

Scope: local asynchronous transport and execution contracts.

## Result

v0.47 adds no third-party dependency and changes no dependency feature. Local
future contracts use only `core::future::Future`; conformance state uses
`core::cell::Cell`. No executor, allocator, task system, network client, TLS
stack, clock, filesystem, random source, or operating-system abstraction enters
the default graph.

The cross-thread traits use the same non-committing response staging as the
local contracts and receive provider-neutral blanket implementations of those
local contracts. Reqwest migrates its adapter code without changing its
reviewed dependency graph and continues to use Tokio only when explicitly
enabled.

`scripts/check_latest_tools.sh --fetch` reports the pinned Cargo security and
fuzz tools current on crates.io. `cargo update --workspace --dry-run` reports
zero compatible package updates for Rust 1.90. Registry review on 2026-08-03
found no admitted stable direct dependency update requiring a lockfile change.

## Third-Party Version Changes

None.

## Local Package Changes

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.46.0` | `0.47.0` | Local async traits and execution paths. |
| `cloud-sdk-hetzner` | `0.36.0` | `0.36.1` | Dependency-only core update. |
| `cloud-sdk-reqwest` | `0.31.1` | `0.32.0` | Send async staging and driver migration. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | Unchanged and not published. |
| `cloud-sdk-testkit` | `0.26.0` | `0.27.0` | Local-only mock and conformance fixtures. |

## Required Verification

- genuinely `!Send` future compilation and execution;
- dropped Send/local future body/header cleanup and possibly-sent classification;
- sequential and cooperatively outstanding local futures;
- prepared request, provider-link, raw-executor, and retry-permit local paths;
- automatic cross-thread-to-local compatibility;
- default/all-feature no_std, browser-WASM, embedded, portable/native platform,
  MSRV, package, SBOM, Cargo Deny, RustSec, and v0.47 release-gate evidence.
