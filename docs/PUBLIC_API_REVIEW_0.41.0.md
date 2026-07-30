# v0.41.0 Public API Review

Date: 2026-07-30

Scope: provider-neutral bearer authentication policy and optional adapter
credential lifecycle.

## Decision

Core adds `AuthenticationScope`, `AuthenticationScopePolicy`,
`ScopeRequirement`, payload-free scope errors, bounded `ScopeValue`,
`CredentialGeneration`, `RefreshHandoff`, `AuthenticatedRequest`, and blocking
and executor-neutral async authenticated transport traits.

The contracts remain `no_std`, allocation-free, credential-free, and
provider-neutral. They own no token bytes, acquisition, clock, expiry logic,
executor, retry, filesystem, or secret store.

## Adapter API

`cloud-sdk-reqwest` adds `BearerCredential`, immutable
`BearerCredentialScope`, redacted `BearerCredentialSnapshot`, generation-safe
rotation, and lineage-bound `BearerRefreshHandoff` compare-and-swap refresh.
Blocking and async builders require a provider/service/endpoint-bound
credential and reject configured endpoint mismatch. Authenticated sends
require exact `Required` rules for those three base identities and validate
them before token snapshot or header construction.

Authenticated clients no longer implement `BlockingTransport` or
`AsyncTransport`. This breaking change closes the policy-bypass path. Raw
clients remain credential-free and unchanged.

Token and refresh errors expose only static categories. Scope-owned audience,
account, tenant, endpoint, token, snapshot, and client diagnostics remain
redacted.

## Compatibility

Callers must:

1. bind each bearer token to immutable provider/service/endpoint scope;
2. wrap each request in `AuthenticatedRequest` with provider or
   operation-owned requirements;
3. invoke the authenticated transport trait;
4. use a store-bound generation handoff for external refresh completion.

See [`MIGRATION_0.41.0.md`](MIGRATION_0.41.0.md) and
[`AUTHENTICATION_POLICY.md`](AUTHENTICATION_POLICY.md).
