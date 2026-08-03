# v0.50.0 Dependency Review

Date: 2026-08-03

Scope: compile-time operation identifiers and exhaustive Hetzner operation
associations.

## Result

v0.50 adds no dependency and changes no dependency feature. The neutral change
uses only const core APIs. The provider association layer uses existing
`cloud-sdk` and first-party Hetzner request/preparation types. Its generator
uses the Python standard library and local rustfmt; it performs no network I/O.

No network client, TLS stack, runtime, task system, clock, filesystem access in
library code, random source, serializer, or operating-system abstraction enters
the default graph. Default provider code remains allocation-free and no_std.

Reqwest and testkit receive only manifest dependency patches for the v0.50
core. Sanitization has no code, dependency, or package metadata change and is
not published in this release.

## Local Package Changes

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.49.0` | `0.50.0` | Const operation IDs and compile-time literal macro. |
| `cloud-sdk-hetzner` | `0.37.0` | `0.38.0` | Exhaustive typed operation association layer. |
| `cloud-sdk-reqwest` | `0.32.2` | `0.32.3` | Dependency-only core update. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.17.0` | Unchanged and not published. |
| `cloud-sdk-testkit` | `0.28.1` | `0.28.2` | Dependency-only core update. |

## Root Lockfile Changes

| Package | Previous | Current | Reason |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.49.0` | `0.50.0` | Workspace package version. |
| `cloud-sdk-hetzner` | `0.37.0` | `0.38.0` | Workspace package version. |
| `cloud-sdk-reqwest` | `0.32.2` | `0.32.3` | Workspace package version. |
| `cloud-sdk-testkit` | `0.28.1` | `0.28.2` | Workspace package version. |

## Required Verification

- exact generated coverage for 208 active operations and 91 body operations;
- source-locked service, method, query, body, status, response, and pagination;
- runtime cross-check of endpoint, authentication, metadata, and response policy;
- compile-fail cross-operation component and typed response mismatches;
- default, no_std, all-feature, docs, package, clippy, and MSRV checks;
- fuzz, SBOM, Cargo Deny, RustSec, latest dependency/tooling, and release gate.
