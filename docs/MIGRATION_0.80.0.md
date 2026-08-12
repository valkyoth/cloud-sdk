# Migrating To v0.80.0

v0.80 is additive. Existing v0.75 published APIs and v0.76-v0.79 source APIs
do not require changes.

## Published Versions

```toml
[dependencies]
cloud-sdk = "0.80.0"
cloud-sdk-hetzner = { version = "0.43.0", features = ["serde"] }
```

The independently versioned transport, sanitization, and testkit packages are
`0.35.1`, `0.19.0`, and `0.30.3` respectively.

## Preparing An IP Read

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{RobotIpAddress, RobotIpGetRequest};

let address = RobotIpAddress::new("192.0.2.10")?;
let request = RobotIpGetRequest::new(address);
let mut target = [0_u8; 96];
let mut body = [0_u8; 1];
let prepared = request.prepare_bound(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(
    prepared.as_untyped().transport_request().target().as_str(),
    "/ip/192.0.2.10"
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

`RobotIpListRequest::all()` lists all assigned addresses;
`RobotIpListRequest::for_server(address)` returns `Result`, accepts only an
IPv4 server main address, and binds every returned entry to that address.
IPv6 filters fail before transport. Use `RobotIpUpdateRequest` with a non-empty
`RobotIpTrafficUpdate` for explicit partial threshold changes.

Separate-MAC operations use `RobotIpMacGetRequest`, `RobotIpMacSetRequest`, and
`RobotIpMacDeleteRequest`. With `serde`, call `prepare_bound`, validate through
`PreparedRobotIp`, and decode the resulting `CheckedRobotIp`. Read decoders
bind identity/filter state; update and MAC decoders also verify the provider's
acknowledgement.

Traffic updates and MAC mutations must use the request-bound permit API for
execution. Sensitive traffic forms require `build_robot_ip_plan_digest`;
bodyless MAC operations may use `build_robot_ip_canonical_plan` or the digest
builder. MAC generation and deletion are never automatically retried. Read and
reconcile provider state after uncertain delivery.

v0.80 does not add a high-level Robot client or perform network requests by
itself.
