# Migrating Source Users To v0.81.0

v0.81 is additive. Existing v0.80 code does not require changes. The published
crate versions remain `cloud-sdk 0.80.0` and `cloud-sdk-hetzner 0.43.0`; v0.81
is a source milestone and crates.io publication is deferred to v0.85.0.

## Preparing A Subnet Read

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{RobotSubnetAddress, RobotSubnetGetRequest};

let subnet = RobotSubnetAddress::new("192.0.2.10")?;
let request = RobotSubnetGetRequest::new(subnet);
let mut target = [0_u8; 96];
let mut body = [0_u8; 1];
let prepared = request.prepare_bound(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(
    prepared.as_untyped().transport_request().target().as_str(),
    "/subnet/192.0.2.10"
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

`RobotSubnetListRequest::for_server` accepts only an IPv4 server main address.
`RobotSubnetUpdateRequest` requires a nonempty `RobotSubnetTrafficUpdate`.
`RobotSubnetMacSetRequest` requires the selected `RobotMacAddress`; it does not
infer one from an earlier response.

Default restoration is intentionally evidence-gated. Decode a checked subnet
detail and checked MAC snapshot, then consume both snapshots:

```rust,ignore
let delete = RobotSubnetMacDeleteRequest::from_checked(subnet, mac_state)?;
```

The snapshots must agree on route identity and prefix. The subnet must be
assigned, and the assigned server main address must occur in `possible_mac`.
There is no `RobotSubnetMacDeleteRequest::new(address)` constructor. This is a
pre-release source correction made before v0.81 is tagged.

With `serde`, use `prepare_bound`, validate the response through
`PreparedRobotSubnet`, and call the matching checked decoder. Traffic and MAC
forms are sensitive, so their authorized execution must use
`build_robot_subnet_plan_digest`. MAC PUT/DELETE never retry automatically.
For non-success responses, call `decode_failure` on the exact request. It
admits only that operation's documented status/code combinations.

Do not assume the returned route identity is the mathematical network address.
Official examples contain host bits. Use `with_network_address` and
`with_broadcast` for derived boundaries and preserve the route identity for
subsequent API paths.
