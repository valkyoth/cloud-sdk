# Migrating Source Users To v0.87

v0.87 is an internal cumulative milestone. No crate is published until the
v0.90 checkpoint.

Source users should update the core dependency to `0.87.0`; the Hetzner package
version remains `0.44.0` during the cumulative train.

```toml
[dependencies]
cloud-sdk = "0.87.0"
cloud-sdk-hetzner = { version = "0.44.0", features = ["serde"] }
```

Traffic queries use protected interval and target values:

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{
    RobotIpAddress, RobotTrafficGranularity, RobotTrafficInterval,
    RobotTrafficRequest, RobotTrafficTarget,
};

let interval = RobotTrafficInterval::new(
    RobotTrafficGranularity::Month,
    "2026-07-01",
    "2026-07-31",
)?;
let target = RobotTrafficTarget::ip(RobotIpAddress::new("192.0.2.10")?);
let request = RobotTrafficRequest::new(interval, alloc::vec![target], false)?;
let mut path = [0_u8; 32];
let mut body = [0_u8; 256];
let prepared = request.prepare_bound(PreparationStorage::new(&mut path, &mut body))?;
# let _ = prepared;
# Ok::<(), Box<dyn core::error::Error>>(())
```

`RobotTrafficRequest::new` canonicalizes target order. Callers must treat the
input as a set rather than rely on insertion order. A closed core approval owns
the Robot traffic operation ID; subsequent builder calls cannot replace that
provider-approved identity.

Execute through the existing authenticated transport boundary, validate the
response with `PreparedRobotTraffic::validate_response`, and call
`CheckedRobotTraffic::decode_response`. Returned targets can be fewer than the
request because Robot omits targets with no data.
