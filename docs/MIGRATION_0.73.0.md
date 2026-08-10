# Migrating Source Users To v0.73.0

v0.73.0 is an internal source milestone. The latest crates.io checkpoint
remains v0.70.0, and cumulative publication is deferred to v0.75.0.

## Adopting Named Storage Reads

Construct an official `HetznerClient::storage` from an endpoint-bound
authenticated transport. Build the same `AssociatedOperation<O, ...>` used by
existing generic code, acquire one `ClientWorkspaceLease`, and call the named
blocking, `Send` async, or local-async method.

The compile-checked
[`storage_client` example](../crates/cloud-sdk-hetzner/examples/storage_client.rs)
shows a paginated Storage Box list against the deterministic testkit
transport. Existing generic associated-operation execution remains available.

## Adopting Named Storage State Changes

For Storage mutation, deletion, rollback, password reset, or cost-bearing
operations:

1. Call the named `prepare_<operation>` method with a
   `PreparationStorageGuard`.
2. Review the complete method, target, query, body, and cost implications.
3. Build and fingerprint an `AssociatedPlanConfirmation`.
4. Create the exact matching mutation, destructive, or cost permit.
5. Begin one attempt and pass it to the named executor method.

Password-bearing create and reset requests require
`build_associated_plan_digest` with `Sha256PlanHasher`, caller-owned canonical
scratch, and a 32-byte digest buffer. Exact fingerprint construction fails
closed with `SensitiveBodyRequiresDigest`.

There is no direct state-changing method, automatic product selection,
password generation, snapshot rollback, or implicit retry. Sensitive source
buffers remain caller-owned and require caller cleanup; the SDK clears its
complete guarded request and response buffers.

## Large Response Storage

Select response capacity for the largest admitted page and use the existing
incremental checked decoder. A response exceeding caller capacity fails before
partial models are returned; the SDK does not silently truncate or switch to
unbounded allocation.
