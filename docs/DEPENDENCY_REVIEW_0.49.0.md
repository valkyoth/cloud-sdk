# v0.49.0 Dependency Review

Date: 2026-08-03

Scope: provider-owned bounded incremental JSON decoding.

## Result

v0.49 adds no third-party dependency and changes no dependency feature. The
incremental decoder uses `alloc` collections, the already admitted first-party
`cloud-sdk-sanitization` facade, and core UTF-8 and formatting APIs under the
existing `cloud-sdk-hetzner/serde` feature. Pentest remediation adds one
provider-neutral fallible protected-string growth helper to that facade.

No network client, TLS stack, runtime, task system, clock, filesystem, random
source, operating-system abstraction, or new serializer enters the graph.
The default provider graph remains no_std and allocation-free.

Reqwest and testkit receive only manifest dependency patches for the v0.49
core facade. Sanitization is a code release and is included in publication.
`cargo outdated --workspace --root-deps-only` and
`scripts/check_latest_tools.sh --fetch` reported every direct dependency and
pinned Cargo security/fuzz tool current on crates.io on 2026-08-03.

## Local Package Changes

| Package | Previous | Current | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.48.0` | `0.49.0` | Facade documentation and provider version alignment. |
| `cloud-sdk-hetzner` | `0.36.2` | `0.37.0` | Bounded incremental JSON visitor and decoder. |
| `cloud-sdk-reqwest` | `0.32.1` | `0.32.2` | Dependency-only core update. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.17.0` | Bounded fallible protected-string growth. |
| `cloud-sdk-testkit` | `0.28.0` | `0.28.1` | Dependency-only core update. |

## Required Verification

- default and all-feature no_std and platform compilation;
- one-shot/chunked differential JSON fixtures;
- exact and exhausted input, depth, token, field, string, number, and exponent limits;
- every representative byte, UTF-8, escape, and surrogate split;
- truncation, amplification, duplicate keys, early stop, and terminal failure;
- fallible frame, key, number, and duplicate-store allocation;
- panic poisoning, protected scratch guards, immediate stop cleanup, and redacted diagnostics;
- fuzz, package, SBOM, Cargo Deny, RustSec, MSRV, and v0.49 release-gate evidence.
