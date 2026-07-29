# v0.39.0 Public API Review

Date: 2026-07-29

Scope: transactional fixed-buffer encoding, request preparation cleanup, and
capacity profiles.

## Decision

Core adds `SnapshotEncoder` and measure/encode entry points over immutable
`Copy` snapshots. The callback is a function pointer, preventing captured
mutable state. Every pass receives the same by-value snapshot. Exact output is
compared during a final read-only replay, so no digest or collision assumption
is required.

Capacity and aggregate-limit rejection occurs before destination mutation.
When later pass behavior differs, only the exact measured destination is
volatile-cleared. An armed rollback owner clears that same exact prefix during
panic unwinding from the write or verification callback. Arithmetic in
variable-length measurements uses checked addition.

## Preparation Ownership

`PreparationStorageGuard` owns two `SecretBuffer` guards. Its `prepare` method
returns a `PreparedRequest` whose lifetime is tied to the mutable guard borrow.
Safe Rust therefore prevents cleanup ownership from ending while the request
is usable. Every preparation attempt clears complete target and body buffers
before reuse, and dropping the guard performs the same cleanup. This includes
unused tails and prevents a shorter later request from retaining bytes from a
longer earlier request.

Profile construction establishes both cleanup guards before validating
capacities. Target- or body-capacity rejection therefore clears both complete
caller buffers before returning.

`PreparationStorage::new` remains as the low-level contract for callers with
their own cleanup owner. This preserves provider integrations while making the
first-party secure route shorter.

## Allocation

The default graph remains allocation-free and `no_std`. Named capacities are
plain constants. The existing `alloc` feature admits
`OwnedPreparationStorage`, which uses fallible reservation and exact boxed
slices. Allocation failure is payload-free and does not panic through the
public constructor.

## Compatibility

This is additive in `cloud-sdk`. Hetzner request methods preserve their public
signatures and error enums while changing failure semantics so undersized
destinations remain unchanged. Obsolete internal mutable query cursors were
removed; they were not public crate API. Internal static-path writers now
validate their complete result through `EndpointPath`; this is intentional
defense in depth for any future non-literal call site.

See [`MIGRATION_0.39.0.md`](MIGRATION_0.39.0.md).
