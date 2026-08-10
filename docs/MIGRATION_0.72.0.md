# Migrating Source Users To v0.72.0

v0.72.0 is an internal source milestone. The latest crates.io checkpoint
remains v0.70.0, and cumulative publication is deferred to v0.75.0.

## Adopting Named Security Reads

Construct an official `HetznerClient::security` from an endpoint-bound
authenticated transport. Build the same `AssociatedOperation<O, ...>` used by
existing generic code, acquire one `ClientWorkspaceLease`, and call the named
blocking, `Send` async, or local-async method.

The compile-checked
[`security_client` example](../crates/cloud-sdk-hetzner/examples/security_client.rs)
shows a paginated certificate list against the deterministic testkit transport.

## Adopting Named Security State Changes

For certificate and SSH-key mutation or deletion:

1. Call the named `prepare_<operation>` method with a
   `PreparationStorageGuard`.
2. Review the complete method, target, query, and body.
3. Build and fingerprint an `AssociatedPlanConfirmation`.
4. Create the matching mutation or destructive permit.
5. Begin one attempt and pass it to the named executor method.

There is no direct state-changing method and no implicit retry. Uploaded
private-key source buffers remain caller-owned and require caller cleanup; the
SDK clears its complete guarded request and response buffers.

## SSH-Key Rotation

Create and verify the replacement key first. Only after deployment succeeds,
prepare deletion of the old key and authorize that separate destructive plan.
v0.72 deliberately provides no automatic create-delete sequence or rollback.
