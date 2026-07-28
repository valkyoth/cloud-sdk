# v0.37.0 Dependency Review

Date: 2026-07-28

Scope: direct and locked dependency impact of response-buffer provenance.

## Result

No external dependency is added or upgraded by this implementation. The sealed
writer, cleanup-owning guard, commitment state, checked lifetimes, and
adversarial tests use first-party `core`-only code. `cloud-sdk` continues to
have no normal dependency.

| Package | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.37.0` | sealed response writer and checked cleanup guard |
| `cloud-sdk-hetzner` | `0.30.0` | cleanup-owning checked decoder migration |
| `cloud-sdk-reqwest` | `0.25.0` | blocking and async writer commitment |
| `cloud-sdk-sanitization` | `0.15.5` | dependency-only facade update |
| `cloud-sdk-testkit` | `0.22.0` | deterministic sealed-writer fixtures |

The default core and provider graphs remain `no_std` and transport-free.
Reqwest and its TLS/runtime dependencies remain behind explicit adapter
features. The production reqwest response sanitizer continues to delegate to
the admitted `cloud-sdk-sanitization` boundary.

## Required Verification

- `scripts/check_response_provenance.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_testkit_boundary.sh`
- `scripts/check_platform_matrix.sh --all`
- Rust `1.90.0` through pinned stable compatibility
- Cargo Deny, RustSec, package, documentation, fuzz, and SPDX checks
- `scripts/checks.sh`

The final release gate reruns current crate and tool freshness checks. Any
resulting dependency change requires a new dependency review and pentest commit.
