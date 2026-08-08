# v0.65.0 Dependency Review

Status: implementation complete; incremental pentest required.

The v0.65 DNS implementation adds no dependency or feature. It reuses the
existing optional Serde parser, `alloc`, and first-party protected string
boundary. TSIG response validation is implemented without a new Base64 edge.

Across the cumulative v0.61-v0.65 publication window, `base64-ng` changed from
the exact admitted `1.3.9` release to exact `2.0.1` in the optional reqwest
Basic-auth feature graph. The newer AWS-LC `1.18.0/0.44.0/0.14.1` set was
reviewed but rejected because its FIPS crate cannot complete a clean build from
Cargo's read-only source tree. The exact `1.17.3/0.43.0/0.13.16` set therefore
remains pinned. Routine compatible transitive updates were resolved on
2026-08-08 without changing any default feature graph.

## Independent Versions

| Package | Previous published | v0.65 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.65.0` | cumulative code | yes |
| `cloud-sdk-hetzner` | `0.39.1` | `0.40.0` | cumulative code | yes |
| `cloud-sdk-reqwest` | `0.33.0` | `0.34.0` | cumulative code | yes |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.1` | `0.30.0` | cumulative code | yes |

The unpublished OVHcloud v2 probe inherits workspace metadata but is excluded
from the publishable package set. Publication order is core, reqwest, testkit,
then the Hetzner provider after each dependency is visible on crates.io.

## Root Lockfile Changes

| Package | Previous | v0.65 | Review |
| --- | --- | --- | --- |
| `base64-ng` | `1.3.9` | `2.0.1` | Exact first-party update in optional reqwest Basic-auth graphs; separately reviewed and tested. |
| `cc` | `1.4.0` | `1.4.2` | Build-only native compiler driver patch update. |
| `find-msvc-tools` | `0.1.9` | `0.1.10` | Build-only MSVC discovery patch update. |
| `js-sys` | `0.3.103` | `0.3.104` | Target-specific wasm binding patch update. |
| `ovhcloud-v2-probe` | `-` | `0.65.0` | Unpublished workspace conformance harness; excluded from crates.io publication. |
| `thiserror` | `2.0.19` | `2.0.20` | Compatible error-derive patch update in optional transport graphs. |
| `thiserror-impl` | `2.0.19` | `2.0.20` | Compile-time derive implementation paired with `thiserror`. |
| `wasm-bindgen` | `0.2.126` | `0.2.127` | Target-specific wasm binding patch update. |
| `wasm-bindgen-futures` | `0.4.76` | `0.4.77` | Target-specific wasm future bridge patch update. |
| `wasm-bindgen-macro` | `0.2.126` | `0.2.127` | Compile-time wasm macro patch update. |
| `wasm-bindgen-macro-support` | `0.2.126` | `0.2.127` | Compile-time wasm macro support patch update. |
| `wasm-bindgen-shared` | `0.2.126` | `0.2.127` | Shared wasm binding support patch update. |
| `web-sys` | `0.3.103` | `0.3.104` | Target-specific Web API binding patch update. |

## Rejected AWS-LC Update

The reviewed Cargo archive SHA-256 values for the rejected AWS-LC update are:

| Package | SHA-256 |
| --- | --- |
| `aws-lc-rs 1.18.0` | `ce2b2dcc879c3bae0d371e77c99f2238400ef24ec001394befa67b6e543add9e` |
| `aws-lc-sys 0.44.0` | `f09fae7be8bb3174e05c6afdb34199e6dc0c7c04ba9fa237b1967adfbde27483` |
| `aws-lc-fips-sys 0.14.1` | `118303cd75f63d1933a90c2ceb7e697281ac6acbdbcc490b46419f25a527ab90` |

The candidate FIPS source reports AWS-LC FIPS `4.1.0`. A clean MSRV matrix
build fails because its CMake configuration writes `base.h` and `opensslv.h`
into Cargo's read-only registry source directory. No environment override was
found that preserves the required bundled-source policy while redirecting
those generated files. The update is rejected until upstream supports a clean
out-of-tree build and the complete platform, FIPS, package, audit, and pentest
gates pass.

The retained `aws-lc-fips-sys 0.13.16` source reports AWS-LC FIPS `3.4.0`.
The active NIST certificate `#5314` identifies AWS-LC 3 module `3.1.0`; this
project therefore does not claim that the selected module, build environment,
or deployment is covered by that certificate or is accredited.
