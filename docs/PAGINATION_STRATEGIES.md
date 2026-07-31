# Pagination Strategies

`cloud-sdk` models pagination as provider-neutral, caller-driven state. It
does not fetch pages, allocate continuation storage, select a digest, sleep,
or retry. Every traversal must select hard request, item, and opaque-state
limits before its first request.

## Shared Budget

```rust
use cloud_sdk::pagination::{
    PaginationBudget, PaginationLimits, SnapshotPolicy,
};

let limits = PaginationLimits::new(20, 10_000, 512)?;
let budget = PaginationBudget::new(limits, SnapshotPolicy::Optional);
# Ok::<(), cloud_sdk::pagination::PaginationError>(())
```

`PaginationBudget::admit` is transactional and must be called once for every
decoded cursor, marker, or provider-link response. A response that advertises
a continuation is rejected when no request remains to follow it. Snapshot
presence and value cannot change after the first accepted response.

## Numbered Pages

Use `NumberedPagination` when the provider requests one-based page numbers.
It binds page size, total entries, last page, exact adjacent navigation, item
count, and the shared budget. Hetzner list metadata converts to
`NumberedPageMetadata` through `PaginationMetadata::as_core`.

## Offsets

Use `OffsetPagination` when the next request starts at an absolute item
offset. The strategy derives the next offset from accepted entries and rejects
skips, total drift, empty continuation responses, and budget exhaustion before
advancing.

## Cursors And Markers

`PaginationCursor` and `PaginationMarker` own caller-provided destination
storage through `SecretBuffer`. They are neither `Copy` nor `Clone`, expose
state only to closures, and can be created only by `transfer_from`:

```rust
use cloud_sdk::pagination::{PaginationCursor, PaginationLimits};

let limits = PaginationLimits::new(20, 10_000, 512)?;
let mut decoded = *b"opaque-next-cursor";
let mut storage = [0_u8; 64];
{
    let cursor = PaginationCursor::transfer_from(
        &mut decoded,
        &mut storage,
        limits,
    )?;
    cursor.with_cursor(|value| assert_eq!(value, b"opaque-next-cursor"));
}
assert!(decoded.iter().all(|byte| *byte == 0));
assert!(storage.iter().all(|byte| *byte == 0));
# Ok::<(), cloud_sdk::pagination::PaginationError>(())
```

Transfer clears the complete destination before validation and clears the
source on success or failure. `CursorHistory` stores the exact cursor beside a
caller-produced 32-byte digest. Exact repetition is a cycle; equal digests for
different cursors are collisions; equal cursors with different digests also
fail closed. History has independent entry and byte budgets.

## Provider Links

Some providers return an absolute or origin-form next link. Bind such links to
the original endpoint, HTTP method, operation ID, and exact operation path with
`ProviderLinkBinding`, then use `ValidatedProviderLink::transfer_from`.

The validator rejects scheme or authority changes, userinfo, fragments, path
changes, method changes, and operation changes. It preserves the exact raw
query bytes, including ordering, duplicates, `+`, multiple `=`, and valid
percent triplets. The resulting `ProviderLinkQuery` has no public constructor
and `RequestTarget::assemble` rejects it, so it cannot be mixed with another
path or a structured query builder.

Absolute links are accepted only when their normalized authority equals the
bound endpoint. Custom endpoints must still come from trusted operator
configuration and must never be selected from tenant-controlled input.

## Cleanup And Diagnostics

Opaque state, cursor history, and provider links use caller-owned storage and
clear it on drop. Failure clears source and destination state. Debug output is
redacted. The caller remains responsible for clearing provider-decoder scratch
and for choosing a collision-resistant cursor digest implementation when
history is used.
