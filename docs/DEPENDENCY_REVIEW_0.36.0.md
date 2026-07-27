# v0.36.0 Dependency Review

Date: 2026-07-27

Scope: direct and locked dependency impact of the bounded HTTP header release.

## Result

No external dependency is added or upgraded by this implementation. Header
validation, duplicate detection, fixed-capacity response storage, redaction,
and atomic encoding use first-party `core`-only code. `cloud-sdk` continues to
have no normal dependency.

| Package | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.36.0` | bounded request and response header contracts |
| `cloud-sdk-hetzner` | `0.29.0` | explicit prepared Accept and Content-Type headers |
| `cloud-sdk-reqwest` | `0.24.0` | request forwarding and bounded response capture |
| `cloud-sdk-sanitization` | `0.15.4` | dependency-only patch |
| `cloud-sdk-testkit` | `0.21.0` | exact header matching and response metadata |

The existing reqwest adapter graph supplies its own header map and HTTP
implementation. Core and provider crates do not admit `http`, reqwest, a URL
parser, allocation, TLS, runtime, filesystem, clock, or secret-storage
dependencies.

## Freshness

The implementation started from the dependency and tool versions reviewed for
v0.35 on 2026-07-27. The final release gate reruns
`scripts/check_latest_tools.sh --fetch`, Cargo update dry runs, Cargo Deny,
RustSec, complete SPDX generation, and all locked auxiliary graph checks.

## Required Verification

- `scripts/check_header_model.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_testkit_boundary.sh`
- `scripts/check_platform_matrix.sh --all`
- Rust `1.90.0` through pinned stable compatibility
- all Cargo Deny, RustSec, package, documentation, and SPDX checks
- `scripts/checks.sh`
