# Migrating Source Users To v0.58.0

v0.58.0 is a source-only milestone. The latest crates.io checkpoint remains
v0.55.0; package publication is deferred to v0.60.0.

## Core Additions

Provider integrations can define a finite `EndpointPairPolicy` from
`RegionalEndpointPair` values. Each pair binds one canonical region to exact
HTTPS API and token `EndpointIdentity` values. Alias, cross-region, duplicate,
unknown-region, and downgrade cases reject before credential use.

`CredentialLifetime::from_expires_in` converts provider expiry durations using
an explicit caller-owned `CredentialTimestamp` and nonzero refresh margin. A
zero margin is rejected because it would leave no interval in which a refresh
handoff can be acquired. No clock or refresh task is added to the default
`no_std` graph.

## Reqwest Changes

`BearerCredentialSnapshot::refresh_handoff()` now returns
`Result<BearerRefreshHandoff, RefreshHandoffError>`. Existing static-token code
must handle the result:

```rust,ignore
let snapshot = client.credential_snapshot()?;
let handoff = snapshot.refresh_handoff()?;
```

For expiring OAuth tokens, construct `BearerCredential::new_expiring`, inspect
the snapshot lifetime when needed, and call `refresh_handoff_at(now)`. Install
the replacement with one of the `*_with_lifetime` rotation or refresh methods.
The adapter rejects attempts to silently convert between static and expiring
credential lifecycles.

Existing error enums add payload-free lifetime-policy variants. Exhaustive
matches must include the new cases. No default feature or dependency is added.
