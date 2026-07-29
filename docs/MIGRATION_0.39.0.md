# Migrating To v0.39

v0.39 makes fixed-buffer request encoding transactional and adds a
cleanup-owning request preparation path.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.39.0"
cloud-sdk-hetzner = "0.32.0"
cloud-sdk-reqwest = { version = "0.26.1", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.23.1"
```

## Transactional Encoding

`cloud_sdk::buffer::encode_snapshot_bounded` receives a `Copy` snapshot and a
noncapturing encoder callback. It measures with checked arithmetic, rejects an
aggregate-cap or output-capacity failure before writing, emits into the exact
admitted prefix, then performs a compare-only replay. Any write or replay
mismatch clears that exact prefix before returning the caller-selected error.

Provider request builders now use this path for complete endpoint paths,
queries, and JSON bodies. Existing high-level Hetzner request methods retain
their signatures and domain-specific errors. Low-level code that calls legacy
single-value buffer writers remains source compatible; those writers now
preflight each value atomically.

No ordinary `Hash` or non-cryptographic digest is used. Exact bounded bytes are
compared directly.

## Preparation Cleanup

Prefer one cleanup owner for target and body storage:

```rust
use cloud_sdk::operation::{PreparationStorageGuard, PrepareOperation};

# fn prepare<O: PrepareOperation>(
#     operation: &O,
# ) -> Result<(), O::Error> {
let mut target = [0_u8; 1024];
let mut body = [0_u8; 16 * 1024];
{
    let mut storage = PreparationStorageGuard::new(&mut target, &mut body);
    let prepared = storage.prepare(operation)?;
    let _request = prepared.transport_request();
    // Execute while `prepared` borrows `storage`.
}
assert!(target.iter().all(|byte| *byte == 0));
assert!(body.iter().all(|byte| *byte == 0));
# Ok(())
# }
```

`PreparationStorage::new` remains available for callers that own an equivalent
cleanup lifecycle. Plain slices are still not cleared automatically after a
successful preparation.

Each `PreparationStorageGuard::prepare` call volatile-clears both complete
buffers before lending them to the operation. Reusing a guard therefore does
not leave a longer earlier request in the unused tail of a shorter later
request. The prepared request must still finish transport use before another
call can borrow the guard.

As part of the transactional migration, internal static-path writers now
validate their complete output with `EndpointPath`. Current callers use static
literals, but retaining this validation prevents a future dynamic call site
from turning the helper into a path-injection boundary.

## Capacity Profiles

`PreparationCapacityProfile` provides:

| Profile | Target | Body |
| --- | ---: | ---: |
| `EMBEDDED` | 1 KiB | 16 KiB |
| `DEFAULT` | 8 KiB | 1 MiB |
| `LARGE` | 8 KiB | 8 MiB |

Profiles validate fixed caller storage without allocation. With
`cloud-sdk/alloc`, `OwnedPreparationStorage::try_for_profile` allocates exact
boxed regions through a fallible API and clears both complete allocations on
drop.

These are storage profiles, not permission or response-size policies. Select
the smallest profile that admits the intended provider operation.

## Guarantee Limits

Request cleanup has the same limits as response cleanup: it does not cover
process abort, `mem::forget`, deliberately leaked guards, immutable or external
copies, TLS/allocator/kernel/device buffers, swap, crash dumps, or remote
systems.
