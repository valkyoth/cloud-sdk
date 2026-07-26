# v0.34.0 Dependency Review

Date: 2026-07-26

Scope: direct and locked dependency freshness during the v0.34 endpoint-policy
release.

## Result

No external direct dependency is added or upgraded by the endpoint-policy
implementation. Endpoint identity, policy algebra, provider migration, and
authority pre-validation are implemented in first-party code. Core continues
to have no normal dependency.

The independent first-party package versions move as documented in
[`CRATE_VERSION_MATRIX.md`](CRATE_VERSION_MATRIX.md). The release freshness
pass also updated these transitive lock entries to their latest
Rust-1.90-compatible versions:

| Package | Previous | Locked |
| --- | --- | --- |
| `cc` | `1.3.0` | `1.4.0` |
| `either` | `1.16.0` | `1.17.0` |
| `glob` | `0.3.3` | `0.3.4` |
| `hyper` | `1.10.1` | `1.11.0` |
| `libc` | `0.2.186` | `0.2.189` |
| `rustls-pki-types` | `1.15.0` | `1.15.1` |
| `syn` | `3.0.2` | `3.0.3` |

No direct feature or dependency boundary changed as a result.

The current direct external versions remain:

| Package | Version | Boundary |
| --- | --- | --- |
| `reqwest` | `0.13.4` | optional transport |
| `rustls` | `0.23.42` | optional transport |
| `rustls-platform-verifier` | `0.7.0` | optional transport |
| `aws-lc-rs` | `1.17.3` | optional native transport |
| `aws-lc-sys` | `0.43.0` | optional native transport |
| `aws-lc-fips-sys` | `0.13.16` | optional FIPS transport |
| `webpki-roots` | `1.0.9` | optional deterministic roots |
| `bytes` | `1.12.1` | optional async body ownership |
| `tokio` | `1.53.1` | optional async transport and tests |
| `serde` | `1.0.229` | optional provider serialization |
| `serde_json` | `1.0.151` | optional/dev provider parsing |
| `sanitization` | `2.0.3` | optional first-party cleanup boundary |

No resolver, socket, URL, IDNA, transport, TLS, runtime, filesystem, clock, or
secret-storage dependency enters `cloud-sdk`. The existing optional reqwest
boundary continues using its reviewed `url` graph only after first-party raw
authority validation.

## Required Verification

- `scripts/check_endpoint_policy.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_sanitization_boundary.sh`
- `scripts/check_testkit_boundary.sh`
- all Cargo Deny and RustSec graph checks
- all complete SPDX freshness checks
- Rust 1.90.0 through pinned stable compatibility
- `scripts/checks.sh`
