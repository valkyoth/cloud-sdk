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
`SnapshotId` retains and compares the exact nonempty provider bytes, up to
`MAX_SNAPSHOT_ID_BYTES`; callers must not truncate or hash UUIDs, ETags,
version tokens, or other provider identities into a smaller value.

## Request Sequencing

Wrap numbered or offset strategies in `PagerDriver` when the workflow must
admit exactly one request before accepting one response. `PagerControl` keeps
cancellation independent from provider continuation state. Rejected strategy
observations remain pending and transactional; they do not advance counters or
allow another request. See [`WORKFLOW_DRIVERS.md`](WORKFLOW_DRIVERS.md).

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

## Header Cursors

Use `HeaderCursorPolicy` when a provider carries page size and continuation
state in HTTP headers. The policy binds one `OperationId`, three distinct
validated names, and one nonzero page size. `with_initial_request_headers`
creates the first public size header. A decoded `HeaderCursorContinuation`
retains that operation and policy and creates the next public size plus
sensitive cursor headers only for the duration of a closure. Decimal scratch
storage is cleared on every path.

`decode_next` reads directly from bounded `ResponseHeaders`. Absence of the
configured next-cursor header is terminal. A present value must be nonempty,
within `PaginationLimits::max_state_bytes`, retained as sensitive metadata,
and canonical visible ASCII that can be sent back as one request-header
value. It is transferred into cleanup-owning caller storage without exposing a
plain string. Empty, control-bearing, non-ASCII, oversized, public, duplicate,
or undersized-storage cases fail closed and clear transfer storage.

Call `HeaderCursorContinuation::observe_history` before following it.
The provider-neutral codec deliberately does not select a digest, infer a
continuation from body length, allocate storage, or issue a request. OVHcloud
v2 conformance binds this contract to `X-Pagination-Size`,
`X-Pagination-Cursor`, and `X-Pagination-Cursor-Next`; other providers must
source-lock their own names and terminal semantics.

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

At use time, call `ValidatedProviderLink::execute_blocking` or
`ValidatedProviderLink::execute_async`. Both methods require one object that
implements `BoundTransport` and the matching authenticated transport trait.
The link retains its admitted endpoint identity, rejects an unbound or
different endpoint, constructs the authenticated request, and dispatches it
through that same object. No public callback or free-standing request separates
the destination check from execution.

Both methods return one flattened `Result<(), ProviderLinkExecutionError<E>>`.
Validation failures remain payload-free `Pagination` variants; transport
failures are `Transport` variants whose `Debug` and `Display` output never
contains the transport payload. Callers must handle the result before advancing
pagination state.

Absolute links are accepted only when their normalized authority equals the
bound endpoint. Custom endpoints must still come from trusted operator
configuration and must never be selected from tenant-controlled input.

## Cleanup And Diagnostics

Opaque state, cursor history, and provider links use caller-owned storage and
clear it on drop. Failure clears source and destination state. Debug output is
redacted. The caller remains responsible for clearing provider-decoder scratch
and for choosing a collision-resistant cursor digest implementation when
history is used.
