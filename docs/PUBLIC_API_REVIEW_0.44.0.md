# v0.44.0 Public API Review

Date: 2026-07-31

Scope: provider-neutral pagination strategy separation.

## Added API

`PaginationLimits`, `PaginationBudget`, `PaginationProgress`, `SnapshotId`,
and `SnapshotPolicy` provide shared request, item, opaque-state, and snapshot
policy. Budget admission is transactional and public so opaque strategies can
use the same limits as structured strategies. `SnapshotId` accepts exact
nonempty provider bytes up to `MAX_SNAPSHOT_ID_BYTES`; `PaginationBudget`
retains and compares those bytes without truncation or hashing.

`NumberedPagination` and `OffsetPagination` own strategy-specific progression.
Their metadata and accepted-boundary types expose only validated navigation,
counts, rate-limit metadata, and progress.

`PaginationCursor`, `PaginationMarker`, `CursorHistory`, and `CursorDigest`
add bounded opaque state and fail-closed cycle/collision handling.
`ValidatedProviderLink` and `ProviderLinkBinding` bind raw continuation links
to one endpoint, method, operation, and exact path. Link use requires the
actual `BoundTransport`; its endpoint is compared with the retained identity
before a closure-scoped request is reconstructed.

`ProviderLinkQuery` is observable through `RequestQuery::ProviderLink` but has
no public constructor. `RequestTarget::assemble` rejects it, preserving the
inseparable validated target.

## Removed And Changed API

The former numbered `PaginationCursor`, `PageMetadata`, and `PageLimit` are
removed. `PaginationCursor` now deliberately names opaque continuation state.
Hetzner `PaginationMetadata::as_core` returns `NumberedPageMetadata`.

These are accepted pre-1.0 source breaks. Retaining aliases would conflate
opaque and numbered state and make strategy selection unclear.

## Security Review

Opaque cursor, marker, history, and link owners are non-`Copy`, redact debug
output, and clear complete caller storage on drop. Atomic transfer clears both
buffers on failure and the source on success. Provider links preserve raw
query bytes without decoding or re-encoding and reject authority, scheme,
userinfo, fragment, path, method, and operation changes. Reuse through an
unbound or different transport endpoint also fails closed. Snapshot state is
committed only after all admission checks succeed and is cleared on drop.
