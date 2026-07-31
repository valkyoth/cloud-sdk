# Migrating To v0.44

v0.44 replaces the single numbered-page cursor with distinct numbered,
offset, cursor, marker, and provider-link strategies.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.44.0"
cloud-sdk-hetzner = "0.34.0"
cloud-sdk-reqwest = { version = "0.30.1", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.25.1"
```

`cloud-sdk-sanitization` is unchanged and is not published for this release.
The reqwest and testkit releases contain dependency-only patch changes.

## Renamed Numbered Types

Replace `PageMetadata`, `PageLimit`, and the old numbered
`PaginationCursor` with:

- `NumberedPageMetadata`;
- `PaginationLimits` and `PaginationBudget`;
- `NumberedPagination`.

`NumberedPagination::observe` now accepts an optional `SnapshotId` in addition
to response metadata, decoded entry count, and optional rate-limit metadata.
Hetzner `PaginationMetadata::as_core` returns `NumberedPageMetadata`.

## Opaque Cursor And Marker State

`PaginationCursor` now means an opaque cleanup-owning cursor. It has no
constructor from a borrowed string and is not `Copy` or `Clone`. Decode into a
mutable source buffer and call `transfer_from` with separate caller-owned
destination storage. Use `with_cursor` or `with_marker` for closure-scoped
access.

Call `PaginationBudget::admit` exactly once for each decoded cursor, marker,
or provider-link response. Use `CursorHistory` when a provider can repeat or
cycle opaque cursor values.

## Provider Links

Do not parse a provider next link into `CanonicalQuery` or `FormQuery`.
Construct a `ProviderLinkBinding` from the original endpoint, method,
operation ID, and exact path, then use `ValidatedProviderLink::transfer_from`.
The validated link creates a closure-scoped transport request only when the
method and operation still match.

Provider-link queries are represented by `RequestQuery::ProviderLink`. The
variant can be observed but not constructed directly, and
`RequestTarget::assemble` returns
`RequestTargetError::ProviderLinkQueryCannotAssemble` for it.

See [`PAGINATION_STRATEGIES.md`](PAGINATION_STRATEGIES.md) for complete policy
and cleanup guidance.
