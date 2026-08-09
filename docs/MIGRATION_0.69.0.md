# Migrating To v0.69.0

v0.69.0 is an internal source milestone after signed v0.68.0. No crate is
published; applications remain on package versions from v0.65.0 until the
cumulative v0.70.0 checkpoint.

## Existing Applications

No migration is required. Existing preparation, direct read-only execution,
checked decoding, permits, and transport APIs remain available.

## Adopting Named Client Storage

Source users can validate all four workspace buffers together:

```rust
use cloud_sdk::client::{ClientCapacityProfile, ClientWorkspace};

# fn main() -> Result<(), cloud_sdk::client::ClientCapacityError> {
let mut target = [0_u8; 1024];
let mut request = [0_u8; 16 * 1024];
let mut response = [0_u8; 64 * 1024];
let mut headers = [0_u8; 8192];
let workspace = ClientWorkspace::for_profile(
    &mut target,
    &mut request,
    &mut response,
    &mut headers,
    ClientCapacityProfile::EMBEDDED,
)?;
assert_eq!(workspace.capacities(), (1024, 16 * 1024, 64 * 1024, 8192));
# Ok(())
# }
```

With `cloud-sdk/alloc`, use
`OwnedClientWorkspace::try_for_profile(profile)` when caller-owned static or
stack storage is unsuitable. Allocation is fallible and the complete owned
buffers are wiped on drop.

## Adopting The Hetzner Facade

With `cloud-sdk-hetzner/serde`, source users can pass an endpoint-bound
authenticated transport to `HetznerClient::cloud`, `dns`, `security`, or
`storage`. Construction verifies the exact official endpoint. Read-only
associated operations can then execute through the facade with one
`ClientWorkspaceLease` and return `CheckedHetznerResponse`.

Custom constructors require
`CustomEndpointAcknowledgement::trusted_operator_configuration()`. Custom
clients intentionally have no v0.69 execution methods. Keep custom endpoints
in trusted operator configuration and never derive them from tenant input.

Service-specific convenience methods arrive in v0.70-v0.73. State-changing
operations continue to require the existing plan-confirm permits.
