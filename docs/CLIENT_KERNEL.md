# Provider-Generic Client Kernel

`cloud_sdk::client` provides one transport-neutral execution path for typed
provider operations. It does not add a network stack, executor, allocator,
clock, retry loop, or hidden request queue.

## Execution Contract

A provider implements `ClientOperation` on a request type that already
implements `PrepareOperation`. One kernel call then performs these steps:

1. clear all four caller-owned workspace regions;
2. prepare the complete target, body, endpoint, authentication, operation, raw
   response, and success-response policies;
3. verify the bound transport endpoint;
4. execute exactly one authenticated transport attempt;
5. classify the bounded response as success, provider error, or other;
6. enter either the checked success decoder or checked provider-error decoder;
7. clear target, request body, response body, headers, and decoder scratch
   before returning an owned result.

The decoder receives `ClientResponse`, not a reusable prepared request. Success
decoding applies the complete `ResponsePolicy`. Error decoding requires a
`4xx` or `5xx` status and applies request-ID policy before provider code sees a
borrowed response.

## Caller-Owned Workspaces

```rust
use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};

let pool = ClientWorkspacePool::<4>::new()?;
let mut target = [0_u8; 1024];
let mut request_body = [0_u8; 4096];
let mut response_body = [0_u8; 8192];
let mut response_headers = [0_u8; 8192];

let workspace = ClientWorkspace::new(
    &mut target,
    &mut request_body,
    &mut response_body,
    &mut response_headers,
);
let lease = pool.try_acquire(workspace)?;
assert_eq!(lease.capacities(), (1024, 4096, 8192, 8192));
drop(lease);

assert!(target.iter().all(|byte| *byte == 0));
# Ok::<(), Box<dyn core::error::Error>>(())
```

`ClientWorkspacePool<N>` stores only an atomic admission bitmap. Every lease
contains a workspace supplied by the caller. Exhaustion returns immediately;
there is no wait list, allocation, or implicit backpressure policy. `N` must be
between one and the target's `usize::BITS` value.

Moving a lease into an async call keeps all four mutable borrows inside that
future. Safe caller code cannot access those regions until completion or
cancellation. Dropping the future clears the buffers before releasing its pool
slot.

## Execution Modes

- `execute_blocking` accepts `BlockingAuthenticatedTransport`.
- `execute_async` accepts a `Sync` `AsyncAuthenticatedTransport` and returns a
  `Send` future. The operation and owned output/error types must satisfy the
  corresponding `Send`/`Sync` bounds.
- `execute_local_async` accepts `LocalAsyncAuthenticatedTransport` and permits
  a `!Send` future on one executor thread.

The same preparation, endpoint, authentication, response, decoding, and
cleanup policy applies in every mode.

## Mutation Boundary

The direct kernel path executes read-only, no-known-cost operations. Mutation,
destructive, and cost-bearing prepared requests fail with
`AuthorizationRequired` before transport access. They must first pass the
plan-confirm permit lifecycle documented in
[`EXECUTION_PERMITS.md`](EXECUTION_PERMITS.md). Provider client integration for
permit-authorized typed decoding remains roadmap work; the kernel does not
weaken or bypass the v0.51 authority boundary.

## Endpoint And Authentication Ownership

The operation owns the admitted endpoint and authentication-scope policy. The
transport owns credential validation and wire I/O. A kernel cannot replace an
operation's official or explicitly acknowledged endpoint with a looser one.
Custom endpoint values must remain trusted configuration and must never come
from tenant-controlled input.

The kernel never retries. Callers must use the explicit retry and idempotency
contracts and may retry only when operation metadata, body replayability,
delivery classification, and caller policy all permit it.
