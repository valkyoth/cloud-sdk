# cloud-sdk 0.55.0 Release Notes

Status: release candidate; cumulative pentest and final retest passed.

Release date: 2026-08-04

Security-Review: PASS
Pentest: PASS
Publication: PENDING

## Overview

v0.55 is the public checkpoint for tagged development milestones v0.51 through
v0.55 after the published v0.50 baseline. It combines explicit mutation
authority, complete bounded client execution, workflow drivers, payload-free
diagnostics, and realistic allocation-free test scenarios.

## Cumulative Changes

- v0.51 added exact plan-confirm execution permits and state-changing
  enforcement.
- v0.52 added the provider-generic client kernel and caller-owned bounded
  workspace leases.
- v0.53 added transactional pager and bounded action-polling drivers with
  separate cancellation, backoff, progress, and time policy.
- v0.54 added finite opt-in lifecycle diagnostics without SDK-owned logging or
  retention.
- v0.55 adds bounded dynamic responders, payload-free request records,
  validated pagination/action scripts, and deterministic stream faults and
  non-terminating patterns.

## Dynamic Testkit

- Dynamic responses use the same sealed response staging path as fixed mocks.
- Selection and staging failures do not consume successful scenario steps.
- Caller-owned atomic recording has a hard 1,024-step cap and stores no request
  or response values.
- Pagination and action scripts validate complete finite lifecycles before use.
- Exact source/sink failures, short writes, endless empty input, alternating
  empty/data input, and unpolled cancellation are covered by tests.
- Security fixture failures now fail tests explicitly across all repository
  test-source roots, and CI scans those complete roots for prohibited bypasses.

## Versions

| Crate | Version | Publication |
| --- | --- | --- |
| `cloud-sdk` | `0.55.0` | planned |
| `cloud-sdk-hetzner` | `0.39.0` | planned code release |
| `cloud-sdk-reqwest` | `0.32.4` | planned dependency-only patch |
| `cloud-sdk-sanitization` | `0.18.0` | planned test-assurance code release |
| `cloud-sdk-testkit` | `0.29.0` | planned code release |

## Documentation

- [`docs/DYNAMIC_TESTKIT.md`](../docs/DYNAMIC_TESTKIT.md)
- [`docs/MIGRATION.md#v0550`](../docs/MIGRATION.md#v0550)
- [`docs/PUBLIC_API_REVIEW.md#v0550`](../docs/PUBLIC_API_REVIEW.md#v0550)
- [`docs/DEPENDENCY_REVIEW.md#v0550`](../docs/DEPENDENCY_REVIEW.md#v0550)

## Pentest

Cumulative pentest from signed `v0.50.0` through the final v0.55
implementation commit and the final retest passed. The permanent report is at
[`security/pentest/v0.55.0.md`](../security/pentest/v0.55.0.md).

## Release Gate

Run `scripts/release_0_55_gate.sh`. Tag and publish only after that clean local
gate plus GitHub CI and CodeQL are green on the final release-evidence commit.
