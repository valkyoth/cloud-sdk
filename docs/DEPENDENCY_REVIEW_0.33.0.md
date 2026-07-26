# v0.33.0 Dependency Review

Date: 2026-07-26

Scope: direct and locked dependency freshness during the v0.33 HTTP method
release.

## Result

No external dependency is added or upgraded by v0.33. The complete method
domain is implemented in first-party `no_std` code and the existing reqwest
adapter uses reqwest's already admitted `Method::from_bytes` conversion.

The independent first-party package versions move as documented in
[`CRATE_VERSION_MATRIX.md`](CRATE_VERSION_MATRIX.md). Lockfile changes are
limited to those local package versions.

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

No package enters the default `cloud-sdk` or provider dependency graph. No new
feature, proc macro, native code, runtime, filesystem, clock, network, or
secret-storage capability is admitted.

## Required Verification

- `scripts/check_http_method_domain.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_sanitization_boundary.sh`
- `scripts/check_testkit_boundary.sh`
- all Cargo Deny and RustSec graph checks
- all complete SPDX freshness checks
- Rust 1.90.0 through pinned stable compatibility
- `scripts/checks.sh`
