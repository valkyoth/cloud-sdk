# Migrating Source Users To v0.52.0

v0.52 adds a provider-generic typed client kernel and tightens one async cleanup
hook bound. It is an internal tagged milestone; crates.io installation remains
on the v0.50 public checkpoint until the cumulative v0.55 release.

## Typed Execution

Provider operations that want the common client path implement
`cloud_sdk::client::ClientOperation` in addition to `PrepareOperation`.
`decode_response` now receives only `ClientResponse`; it cannot obtain a second
prepared request after the one transport attempt.

Callers construct one `ClientWorkspace` per in-flight request, acquire it from
a fixed `ClientWorkspacePool<N>`, and move that lease into `ClientKernel`.
There is no default workspace size or hidden allocation.

The direct kernel path preserves the v0.51 permit boundary. State-changing or
cost-bearing operations return `AuthorizationRequired` before transport.

## Send Async Bounds

`ClientKernel::execute_async` returns a `Send` future. Its transport must be
`Sync`; its operation must be `Sync`; and its owned output, preparation error,
transport error, and decode error must be `Send`.

`ResponseStorageSanitizer` now requires `Sync`. Implementations that previously
used `Cell` or another thread-local interior-mutation primitive must use a
thread-safe primitive such as an atomic or mutex, or keep that sanitizer out of
the Send async path. This closes a mismatch where the documented Send path
could retain a non-Sync trait object across `.await`.

## Cancellation And Cleanup

No migration is needed for caller buffers. Dropping a blocking result, checked
decoder, local future, or Send future continues to clear complete admitted
storage. A cancelled kernel future additionally releases its bounded workspace
slot only after its four buffer guards are dropped.

See [`CLIENT_KERNEL.md`](CLIENT_KERNEL.md) for the complete contract.
