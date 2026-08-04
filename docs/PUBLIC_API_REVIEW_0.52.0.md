# v0.52.0 Public API Review

Date: 2026-08-04

Scope: provider-generic typed execution, bounded concurrent admission,
caller-owned workspace leases, checked success/error decoding, and async
cleanup bounds.

## Added API

`cloud_sdk::client` exports `ClientKernel`, `ClientOperation`, `ClientResponse`,
`ClientResponseKind`, `ClientWorkspace`, `ClientWorkspacePool`,
`ClientWorkspaceLease`, and payload-free execution, decoding, and admission
errors.

The pool owns only an atomic lease bitmap. It has no buffer ownership, queue,
allocator, executor, timer, or wakeup behavior. Each execution consumes one
lease over four independent mutable regions. The pool capacity is a compile-
time constant bounded by `usize::BITS`; zero and excessive capacities fail at
construction.

## Capability Boundaries

`ClientOperation::decode_response` receives only the response facade, not a
reusable `PreparedRequest`. Raw send-once helpers remain crate-private. Public
provider code chooses only checked success or error decoding, and both paths
return an owned value before cleanup owners are released.

Direct client execution retains the v0.51 state-change boundary. Mutation,
destructive, and cost-bearing requests require explicit permit authorization
and cannot use the read-only convenience path.

## Concurrency And Cancellation

Blocking and local-async paths preserve their existing receiver contracts.
The Send-async method explicitly requires `Sync` transport/operation values and
`Send` owned outputs/errors, and returns `impl Future + Send`.

Moving a workspace lease into a future keeps every mutable region uniquely
borrowed across suspension. Dropping a future clears all regions before the
atomic slot is released. Exhaustion is immediate and cannot allocate or queue.

## Changed API

`ResponseStorageSanitizer` now has a `Sync` supertrait. This intentional pre-1.0
tightening makes the existing Send-async response path truthful when an
additive sanitizer is retained across `.await`. Sanitizer methods remain
shared-reference, additive, and surrounded by mandatory core clears.

## Error And Portability Review

New errors have payload-free `Display`, structurally redacted `Debug`, and
`core::error::Error` implementations without generic formatting bounds. The
module remains allocation-free and `no_std`, uses portable core atomics, and
adds no dependency or feature.
