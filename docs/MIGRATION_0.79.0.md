# Migrating Source Users To v0.79.0

v0.79 is additive. Existing v0.78 Robot server code does not require changes.

## Preparing A Cancellation Read

```rust
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk_hetzner::robot::{RobotIpAddress, RobotIpCancellationGetRequest};

let ip = RobotIpAddress::new("192.0.2.10")?;
let request = RobotIpCancellationGetRequest::new(ip);
let mut target = [0_u8; 96];
let mut body = [0_u8; 1];
let prepared = request.prepare(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(
    prepared.transport_request().target().as_str(),
    "/ip/192.0.2.10/cancellation"
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

Use the matching server or subnet request types for those identity domains.
POST requires `RobotCancellationSchedule`; server POST also takes an optional
validated reason and explicit `RobotLocationReservationIntent`. DELETE uses a
named revoke request and accepts only an empty `200 OK` response.

With `serde`, validate the transport response through the prepared request,
then call `request.decode_response(checked)`. The decoder binds the returned
identity to the request and rejects contradictory state.

POST and DELETE are destructive and never automatically retryable. Execution
must use the core destructive permit boundary and caller-owned reconciliation
after uncertain delivery. v0.79 does not add the Robot high-level client.

The crate package remains 0.42.0 until cumulative publication at v0.80.0.
