# v0.48.0 Dependency Review

Date: 2026-08-03

Scope: provider-neutral streaming contracts and testkit fixtures.

## Result

v0.48 adds no third-party dependency and changes no dependency feature. Core
streaming uses only `core::future::Future`, caller-owned slices, and the
already mandatory `cloud-sdk-sanitization` byte clear. It adds no allocator,
executor, task system, network client, TLS stack, clock, filesystem, random
source, or operating-system abstraction.

Testkit uses core's re-export of the mandatory first-party sanitizer for
volatile fixture initialization and transactional rollback. Its one direct
dependency remains `cloud-sdk`, so this does not expand the resolved graph or
publication set.

Reqwest receives no streaming implementation or code change. Hetzner receives
no provider behavior change. Their package versions move only because their
manifest dependency on `cloud-sdk` changes. Sanitization remains unchanged and
is excluded from publication.

`scripts/check_latest_tools.sh --fetch` reports the pinned Cargo security and
fuzz tools current on crates.io. Complete compatible dependency updates were
resolved separately for the root, reqwest feature-unification, fuzz, and
prepared-coverage lockfiles on 2026-08-03.

## Third-Party Version Changes

| Package | Previous | Current | Graph | Review |
| --- | --- | --- | --- | --- |
| `aho-corasick` | `1.1.4` | `1.1.5` | root and reqwest feature unification | Compatible transitive patch used by the reviewed AWS-LC build graph; no feature or direct-dependency change. |
| `data-encoding` | `2.11.0` | `2.11.1` | reqwest feature unification | Compatible transitive patch in certificate verification dependencies; no feature or direct-dependency change. |

Final release verification reruns Cargo Deny, RustSec, complete SBOM, package,
MSRV, and release-gate checks after pentest evidence is committed.

## Local Package Changes

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.47.0` | `0.48.0` | Streaming policy, accounting, replay, and blocking/Send/local drivers. |
| `cloud-sdk-hetzner` | `0.36.1` | `0.36.2` | Dependency-only core update. |
| `cloud-sdk-reqwest` | `0.32.0` | `0.32.1` | Dependency-only core update. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | Unchanged and not published. |
| `cloud-sdk-testkit` | `0.27.0` | `0.28.0` | Deterministic stream fixtures and explicit audited rollback cleanup. |

## Required Verification

- default/all-feature no_std and platform compilation;
- blocking, Send-async, and genuinely local async execution;
- exact/under/over/unknown lengths and all hard boundaries;
- short writes, no read-ahead, empty/wait exhaustion, event cancellation;
- source replay identity and changed-source rejection;
- transactional rollback, dirty direct state, and commit cancellation;
- complete scratch cleanup and payload-free diagnostics;
- package, SBOM, Cargo Deny, RustSec, MSRV, and v0.48 release-gate evidence.
