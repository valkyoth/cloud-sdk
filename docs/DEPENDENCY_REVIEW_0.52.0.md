# v0.52.0 Dependency Review

Date: 2026-08-04

Scope: provider-generic client execution and bounded workspace admission.

## Result

v0.52 adds no dependency and changes no dependency feature. The client kernel
uses `core` futures and atomics plus the already admitted
`cloud-sdk-sanitization::SecretBuffer` cleanup boundary. No network client, TLS
stack, runtime, task system, clock, filesystem, random source, serializer, or
operating-system abstraction enters the default graph.

The `ResponseStorageSanitizer: Sync` change is a trait-bound correction, not a
dependency change. Supporting crates retain their latest published versions
during this internal tag. Their cumulative package-tree changes remain queued
for independent versioning at v0.55.

## Local Package Changes

| Package | Published | Source milestone | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.52.0` | Provider-neutral client kernel, checked response facade, and workspace admission. |
| `cloud-sdk-hetzner` | `0.38.0` | `0.38.0` | Retained cumulative v0.51 typed permit integration. |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.3` | Unpublished source core dependency accumulation. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.17.0` | Unchanged. |
| `cloud-sdk-testkit` | `0.28.2` | `0.28.2` | Retained cumulative v0.51 fixture changes. |

## Root Lockfile Changes

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.51.0` | `0.52.0` | Workspace package version. |

## Required Verification

- default and all-feature `no_std` compilation;
- blocking, Send-async, and local-async fake-provider conformance;
- endpoint and authentication mismatch denial;
- exact success and provider-error policy paths;
- lease exhaustion, bounded concurrency, reuse, and alias compile failures;
- cancellation and cross-mode complete-storage cleanup;
- package, SBOM, Cargo Deny, RustSec, MSRV, platform, and v0.52 release gates.
