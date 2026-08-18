# Dependency Review 0.95.0

Status: release candidate; pentest and final retest passed.

## Result

v0.95 adds one exact Unix-only development dependency: `rustix 1.1.4` with
only `fs`, `process`, and `std`. The ignored live integration harness uses its
safe descriptor API for `NOFOLLOW`, `CLOEXEC`, and effective-user ownership
checks. This edge is absent from published normal dependencies, every default
graph, non-Unix targets, and the provider's `no_std` library surface. It adds
no first-party unsafe code, native build, network client, runtime, clock,
randomness, or secret-store edge.

The harness otherwise reuses the already admitted optional reqwest/rustls
Basic transport, first-party sanitization boundary, client workspace, and
strict Robot decoder. The admission rationale and alternatives are recorded in
[`dependency-admission-rustix.md`](dependency-admission-rustix.md).

The optional non-FIPS graph remains exactly `aws-lc-rs 1.18.0`,
`aws-lc-sys 0.44.0`, and `http-body-util 0.1.5`. FIPS packages and features
remain absent and deferred to Brynja. All ordinary first-party crate defaults
remain empty, and `cloud-sdk-hetzner` has no transport dependency.

## Lockfile Changes

| Package | Previous | v0.95 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.94.0` | `0.95.0` | Advance the internal facade for the public Robot checkpoint. |
| `errno` | `-` | `0.3.14` | Target-specific transitive OS-error support for the Unix-only `rustix` development edge. |
| `linux-raw-sys` | `-` | `0.12.1` | Generated Linux syscall constants used by `rustix`; no first-party direct use. |
| `ovhcloud-v2-probe` | `0.94.0` | `0.95.0` | Advance the excluded workspace probe identity only. |
| `rustix` | `-` | `1.1.4` | Exact safe descriptor and effective-user API for the ignored Unix live harness only. |

## Workspace Version Changes

| Package | Published | v0.95 | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.95.0` | cumulative core code | yes |
| `cloud-sdk-hetzner` | `0.45.0` | `0.46.0` | cumulative Robot code and live evidence | yes |
| `cloud-sdk-reqwest` | `0.35.3` | `0.36.0` | accumulated transport code and dependency updates | yes |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.5` | `0.31.0` | accumulated regression code | yes |

The release tool must publish exactly the four selected crates in dependency
order and must exclude sanitization, fuzzing, internal tools, isolated tests,
the OVHcloud probe, and retired provider-specific helper crates. Cargo Deny,
RustSec, package, feature-unification, platform, freshness, and complete SPDX
SBOM gates remain mandatory before publication.
