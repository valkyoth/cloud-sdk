# Migrating To v0.50

v0.50 adds compile-time Hetzner operation associations. Existing prepared
requests, transports, buffered and incremental decoding, and feature graphs
remain compatible.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.50.0"
cloud-sdk-hetzner = "0.38.0"
cloud-sdk-reqwest = "0.32.3"
cloud-sdk-sanitization = "0.17.0"
cloud-sdk-testkit = "0.28.2"
```

`cloud-sdk` and `cloud-sdk-hetzner` are code releases. Reqwest and testkit
receive dependency-only patches. Sanitization is unchanged and is not part of
the v0.50 publish sequence.

## Additive Typed Route

Existing `PrepareOperation` implementations remain available and continue to
reject component mismatches at runtime. New code can use a marker from
`cloud_sdk_hetzner::association::operations` with `AssociatedOperation` and
receive a `Prepared<O>` that retains the operation identity.

```rust
use cloud_sdk::operation::PreparationStorageGuard;
use cloud_sdk_hetzner::actions::{ActionEndpoint, ActionId};
use cloud_sdk_hetzner::association::AssociatedOperation;
use cloud_sdk_hetzner::association::operations::GetAction;

let id = ActionId::new(1).expect("1 is a valid action ID");
let operation = AssociatedOperation::<GetAction, _>::endpoint(
    ActionEndpoint::Get(id),
)
.expect("the endpoint matches GetAction");

let mut target = [0_u8; 1024];
let mut body = [0_u8; 16 * 1024];
let mut storage = PreparationStorageGuard::new(&mut target, &mut body);
let prepared = operation
    .prepare_typed_guarded(&mut storage)
    .expect("the buffers fit the prepared request");

// Send `prepared` before the storage guard leaves scope.
drop(prepared);
```

Use `query`, `json`, or independently validated component wrappers for
operations requiring those components. Explicitly call `into_untyped` only
when operation type erasure is intended. For secret-bearing operations, prefer
the cleanup-owning typed route shown above.

The guard clears both complete buffers before association validation. The
validator snapshots endpoint policy once and returns an immutable checked token
that request construction consumes without recalculating policy from the
endpoint. The guard clears both buffers again when dropped after transport use.

## Operation Identifiers

`OperationId::new` is now `const`, and `operation_id!` validates static
operation identifier literals at compile time. Runtime behavior, grammar, and
the 128-byte bound are unchanged.

See [`OPERATION_ASSOCIATIONS.md`](OPERATION_ASSOCIATIONS.md) for policy fields,
construction patterns, source-lock generation, and security boundaries.
