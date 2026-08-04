# v0.51.0 Dependency Review

Date: 2026-08-04

Scope: plan-confirm execution permits and typed state-changing execution
gating.

## Result

v0.51 adds no dependency and changes no dependency feature. Permit state uses
`core` atomics and the already admitted `subtle` constant-time comparison and
`cloud-sdk-sanitization` cleanup boundaries. No network client, TLS stack,
runtime, task system, clock, filesystem, random source, serializer, or
operating-system abstraction enters the default graph.

`cargo outdated --workspace --root-deps-only` reported every direct workspace
dependency current, and `scripts/check_latest_tools.sh --fetch` confirmed all
pinned Cargo security, SBOM, and fuzz tools current on crates.io on 2026-08-04.

Supporting crates retain their latest published versions during this internal
tag. Their accumulated code and dependency changes will be independently
versioned and published only at the cumulative v0.55 checkpoint.

## Local Package Changes

| Package | Published | Source milestone | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.51.0` | Provider-neutral plan-confirm and execution-permit code. |
| `cloud-sdk-hetzner` | `0.38.0` | `0.38.0` | Unpublished typed state-changing execution gate. |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.3` | Unpublished dependency accumulation only. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.17.0` | Unchanged. |
| `cloud-sdk-testkit` | `0.28.2` | `0.28.2` | Unpublished read-only direct-execution fixture alignment. |

## Root Lockfile Changes

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.51.0` | Workspace package version. |

## Required Verification

- default and all-feature no_std compilation;
- exact plan domain and field binding plus cleanup on error, unwind, and drop;
- direct and shared state, concurrency, generation, budget, expiry, and replay;
- blocking, Send-async, and local-async delivery-classified execution;
- typed and erased state-changing bypass rejection;
- package, SBOM, Cargo Deny, RustSec, MSRV, platform, and v0.51 release gates.
