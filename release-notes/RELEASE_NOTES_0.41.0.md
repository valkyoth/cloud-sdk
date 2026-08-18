# cloud-sdk 0.41.0 Release Notes

Status: release candidate; pentest and final retest passed. Local and GitHub
release checks remain required before tagging.

Release date: 2026-07-30

## Overview

v0.41 adds a provider-neutral bearer authentication policy above the
credential-free raw HTTP executor. Credentials are immutably scoped, every
authenticated send requires provider or operation policy, and token rotation
and external refresh use generation-safe state transitions.

## Authentication Policy

- Added explicit required, optional, or forbidden policy for provider,
  service, endpoint, audience, account, and tenant.
- Made provider, service, and endpoint mandatory bearer-credential bindings.
- Required exact `Required` policy rules for all three base identities before
  authorization construction.
- Rejected configured credential endpoint mismatch, downgraded base rules,
  supplied forbidden fields, and mismatches before network or header work.
- Applied complete scope validation to the test-only numeric HTTP loopback
  harness instead of bypassing audience, account, and tenant rules.
- Added mandatory authenticated blocking and executor-neutral async transport
  traits.
- Kept raw HTTP execution credential-free.

## Credential Lifecycle

- Added immutable owned credential scope.
- Added monotonic generations and lineage-bound compare-and-swap refresh
  handoffs.
- Rejected foreign-store handoffs and stale refresh completion after a newer
  rotation or refresh.
- Preserved old token snapshots only for in-flight requests.
- Recovered poisoned credential state without holding locks across I/O or
  `.await`.
- Cleared mutable and guarded token sources on success and rejection.
- Rejected padding-only and leading-padding bearer values.
- Added cleanup-owned authorization header values and retired-token cleanup.
- Kept token acquisition, clocks, expiry, tasks, queues, retries, and secret
  stores outside the SDK.
- Updated the optional `http` boundary to `1.5.0`, including upstream URI
  maximum-length enforcement, without admitting its new method automatically.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.41.0` | authentication scope, generation, and transport contracts |
| `cloud-sdk-hetzner` | `0.32.2` | dependency-only core range update |
| `cloud-sdk-reqwest` | `0.28.0` | scoped bearer lifecycle and authenticated adapters |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.24.1` | dependency-only core range update |

## Documentation

- [`docs/AUTHENTICATION_POLICY.md`](../docs/AUTHENTICATION_POLICY.md)
- [`docs/MIGRATION.md#v0410`](../docs/MIGRATION.md#v0410)
- [`docs/PUBLIC_API_REVIEW.md#v0410`](../docs/PUBLIC_API_REVIEW.md#v0410)
- [`docs/DEPENDENCY_REVIEW.md#v0410`](../docs/DEPENDENCY_REVIEW.md#v0410)

## Pentest

The v0.41 pentest findings covered the HTTP loopback test boundary,
credential-store refresh lineage, mandatory provider/service/endpoint
binding, bearer padding grammar, refresh state separation, and complete
extended-scope validation. All findings were remediated and
regression-tested.

The final retest passed commit
`302837f05282e1d5d8bf12c1a960d1620c59dfc3`. See the
[`v0.41.0` pentest report](../security/pentest/v0.41.0.md).

## Release Gate

```text
v0.41.0 pentest stop passed. Tag only after the clean local release gate and
GitHub checks pass on the final release commit.
```
