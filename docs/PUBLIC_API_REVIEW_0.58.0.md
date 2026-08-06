# v0.58.0 Public API Review

Date: 2026-08-06

Scope: provider-neutral regional endpoint pairing and expiring bearer
credential lifecycle.

## Core API

`RegionalEndpointPair` and `EndpointPairPolicy` admit only a finite, unique set
of exact HTTPS region/API/token identities. The API allocates nothing, retains
only caller borrows, and exposes no credential bytes. Its errors are finite,
payload-free, and implement `core::error::Error`.

`CredentialTimestamp`, `CredentialLifetime`, and `CredentialLifetimeState`
model explicit caller time, refresh admission, and exclusive expiry without a
clock or runtime dependency. Construction rejects zero lifetime, incoherent
refresh margins, and arithmetic overflow.

## Reqwest API

`BearerCredential::new_expiring` initializes one expiry-qualified lifecycle.
Snapshots expose only lifetime metadata and can issue a handoff through
`refresh_handoff_at(now)` only during the refresh window. Blocking and async
clients provide matching token-plus-lifetime rotation and refresh methods for
validated tokens, mutable byte sources, and `SecretBuffer` sources.

`refresh_handoff()` now returns `Result` so an expiring credential cannot
bypass explicit time qualification. Existing rotation and refresh errors add
static lifetime-policy variants. These are source-breaking changes for
exhaustive matches and direct static handoff calls, acceptable before 1.0 and
documented in the migration guide.

## Security Review

The pair policy rejects alias, region, duplicate, HTTPS, and exact-identity
confusion. Credential replacement preserves lineage-bound compare-and-swap,
clears mutable inputs, keeps in-flight snapshots stable, and changes token plus
lifetime atomically. `Debug` and `Display` expose no endpoint, token, scope, or
provider payload. The default core and adapter feature graphs remain unchanged.
