# v0.60.0 Dependency Review

Status: implementation review complete; pentest required before release.

Scope: cumulative public checkpoint from published v0.55.0 through v0.60.0.

## Result

No new third-party crate enters the workspace. The default `cloud-sdk` graph
remains allocation-free and `no_std`. The excluded OVHcloud probe consists of
source-lock data, Python validation, shell gates, and Rust conformance tests;
it is not a Cargo package and cannot enter the publish sequence.

| Package | Published | v0.60 | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.60.0` | cumulative code | Yes |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.1` | core dependency | Yes |
| `cloud-sdk-reqwest` | `0.32.4` | `0.33.0` | cumulative code | Yes |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged | No |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.1` | core dependency | Yes |

## Root Lockfile Changes

| Package | Previous | v0.60 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.60.0` | Cumulative provider-neutral code release. |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.1` | Core dependency-only patch. |
| `cloud-sdk-reqwest` | `0.32.4` | `0.33.0` | Cumulative credential-lifetime code release. |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.1` | Core dependency-only patch. |
| `regex-automata` | `0.4.16` | `0.4.18` | Compatible transitive patch selected by the regenerated lockfile; no manifest or feature change. |

## Cumulative Review

- v0.56 adds provider-generic drift tooling without runtime dependencies.
- v0.57 adds the excluded source-locked OVHcloud probe.
- v0.58 uses existing core and adapter dependencies for authority and bearer
  lifetime policy.
- v0.59 adds allocation-free cursor and schema contracts.
- v0.60 adds allocation-free borrowed async-resource models and source-bound
  conformance fixtures.

The public gate must compare package trees with v0.55.0, verify the complete
v0.56-v0.60 pentest chain, regenerate all committed SBOMs, and prove that only
the four selected packages enter the publisher.
