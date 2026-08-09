# Migrating To v0.70.0

v0.70.0 is the public checkpoint after v0.65.0. Update independently versioned
dependencies that your application uses:

```toml
[dependencies]
cloud-sdk = "0.70.0"
cloud-sdk-hetzner = { version = "0.41.0", features = ["serde"] }
cloud-sdk-reqwest = { version = "0.34.1", features = ["blocking-rustls"] }

[dev-dependencies]
cloud-sdk-testkit = "0.30.1"
```

`cloud-sdk-sanitization` remains at `0.18.0` and is not republished.

## Existing Applications

Existing generic associated preparation, direct read-only execution, checked
decoding, and plan-confirm permit code remains valid. No default feature or
third-party dependency changed.

## Adopting Named Cloud Reads

Construct an official `HetznerClient::cloud` from a bound authenticated
transport. Build the same `AssociatedOperation<O, ...>` used by generic code,
acquire one `ClientWorkspaceLease`, then call the operation's named blocking,
`Send` async, or local-async method. The result is an owned
`CheckedHetznerResponse` and the complete workspace is cleared when the lease
returns.

See the compile-checked
[`cloud_client` example](../crates/cloud-sdk-hetzner/examples/cloud_client.rs)
and [`HETZNER_CLIENT.md`](HETZNER_CLIENT.md).

## Adopting Named State Changes

For mutation, destructive, and cost-bearing operations:

1. Call the named `prepare_<operation>` method with a
   `PreparationStorageGuard`.
2. Build and inspect `AssociatedPlanConfirmation`.
3. Create an exact or strong-digest fingerprint.
4. Create the operation's matching `AssociatedMutationPermit`,
   `AssociatedDestructivePermit`, or `AssociatedCostPermit`.
5. Begin one attempt and pass it to the named executor method.

There is no direct state-changing method and no implicit retry. Existing permit
recovery and reconciliation rules remain unchanged.

## Remaining Service Methods

Named DNS, security, and Console Storage Box methods arrive in v0.71-v0.73.
Their existing generic associated-operation workflows remain available.
