# Migrating To v0.85.0

v0.85 is additive and publishes the accumulated v0.81-v0.85 Robot APIs.
Existing v0.80 published APIs and v0.81-v0.84 source APIs are not removed.

## Published Versions

```toml
[dependencies]
cloud-sdk = "0.85.0"
cloud-sdk-hetzner = { version = "0.44.0", features = ["serde"] }
```

The independently versioned reqwest and testkit packages receive
dependency-only patches `0.35.2` and `0.30.4`. Sanitization remains unchanged
at `0.19.0` and is not published.

## Preparing A Boot Read

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{RobotBootGetRequest, RobotServerNumber};

let request = RobotBootGetRequest::new(RobotServerNumber::new(321)?);
let mut target = [0_u8; 64];
let mut body = [0_u8; 1];
let prepared = request.prepare_bound(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(
    prepared.as_untyped().transport_request().target().as_str(),
    "/boot/321"
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

Use the family-specific get requests to discover provider-advertised options.
Activation constructors require explicit bounded values; Rescue and Linux
also accept a bounded unique key-fingerprint slice. Validate admitted
responses through the typed prepared wrapper and call its operation-specific
decoder.

## Mutation Requirements

All activation and deactivation operations are non-idempotent and never
automatically retryable. Linux, VNC, and Windows activation is destructive.
Review exact selections, use authenticated endpoint-bound transport, and read
current state after uncertain delivery before another mutation. Access
generated passwords or keys only through `RobotBootSecret::try_with_secret`
and avoid creating additional owned copies.
