# v0.38.0 Dependency Review

Date: 2026-07-28

Scope: dependency inversion required for mandatory core response cleanup.

## Result

No external package is added or upgraded. The already admitted
`sanitization 2.0.3` remains default-feature-disabled, `no_std`,
allocation-free, and without runtime dependencies in the default graph.

The first-party direction changes:

```text
cloud-sdk -> cloud-sdk-sanitization -> sanitization
```

`cloud-sdk-sanitization` no longer depends on `cloud-sdk`. This removes the
former facade-version patch cycle and lets the neutral core use one audited
cleanup primitive directly. Release automation and regression fixtures now
publish the sanitization boundary before the core crate.

| Package | Version | Change |
| --- | --- | --- |
| `cloud-sdk-sanitization` | `0.16.0` | dependency inversion and scalar cleanup wrapper |
| `cloud-sdk` | `0.38.0` | mandatory response and workspace cleanup |
| `cloud-sdk-hetzner` | `0.31.0` | guard-owned checked decoder scratch |
| `cloud-sdk-reqwest` | `0.26.0` | non-Copy metadata transfer and additive hook integration |
| `cloud-sdk-testkit` | `0.23.0` | non-Copy fixtures and mandatory cleanup conformance |

The default workspace graph contains only the five first-party crates and
`sanitization`. Reqwest, TLS, Tokio, Serde, allocation, filesystem, and clock
capabilities remain absent unless explicitly enabled.

## Required Verification

- `scripts/check_sanitization_boundary.sh`
- `scripts/check_response_cleanup.sh`
- `scripts/check_response_provenance.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_testkit_boundary.sh`
- `scripts/check_platform_matrix.sh --all`
- `scripts/release_crates.py --check`
- Cargo Deny, RustSec, package, documentation, fuzz, MSRV, and SPDX checks
- `scripts/release_0_38_gate.sh`

The release gate reruns crate and tooling freshness checks. Any dependency
change after this review requires updated evidence and another pentest commit.
