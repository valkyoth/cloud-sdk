# Migrating To v0.90

v0.90 is the cumulative v0.86-v0.90 public checkpoint. Update the facade and
Hetzner provider together. The neutral reqwest adapter and testkit receive
dependency-only patches; sanitization does not change.

```toml
[dependencies]
cloud-sdk = "0.90.0"
cloud-sdk-hetzner = { version = "0.45.0", features = ["serde"] }
```

## Robot vSwitch Preparation

The new vSwitch API keeps the exact request associated with the checked
response. This creation example prepares bytes but performs no network call:

```rust
use cloud_sdk::operation::PreparationStorageGuard;
use cloud_sdk_hetzner::robot::{
    RobotVSwitchCreateRequest, RobotVSwitchName, RobotVlanId,
};

let name = RobotVSwitchName::new("private fabric")?;
let vlan = RobotVlanId::new(4000)?;
let request = RobotVSwitchCreateRequest::new(name, vlan);
let mut target = [0_u8; 32];
let mut body = [0_u8; 128];
let mut storage = PreparationStorageGuard::new(&mut target, &mut body);
let prepared = storage.prepare_with(|buffers| request.prepare_bound(buffers))?;
assert_eq!(
    prepared.as_untyped().transport_request().target().as_str(),
    "/vswitch",
);
# drop(prepared);
# Ok::<(), Box<dyn core::error::Error>>(())
```

`RobotVSwitchUpdateIntent` requires at least one replacement field. Membership
changes require a non-empty duplicate-free `RobotVSwitchServers` slice whose
entries are canonical positive server numbers or canonical IP addresses.
VLAN IDs are restricted to `1..=4094`. vSwitch names use the conservative
ASCII profile `[A-Za-z0-9 ._-]` and cannot begin or end with a space.

Creation decodes and verifies the requested name and VLAN. Update,
cancellation, attachment, and detachment accept only the documented empty
success body and return `()`. Treat that acknowledgement as request acceptance,
not reconciled state. Issue a later `RobotVSwitchGetRequest` when current state
matters, and account for concurrent account changes because Robot provides no
revision or ETag binding the two observations.

Every state-changing operation requires its exact request-bound mutation or
destructive permit. None is automatically retryable. After uncertain delivery,
reconcile provider state before constructing another mutation.
