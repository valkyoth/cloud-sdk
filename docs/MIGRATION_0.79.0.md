# Migrating Source Users To v0.79.0

v0.79 is additive. Existing v0.78 Robot server code does not require changes.

## Preparing A Cancellation Read

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{RobotIpAddress, RobotIpCancellationGetRequest};

let ip = RobotIpAddress::new("192.0.2.10")?;
let request = RobotIpCancellationGetRequest::new(ip);
let mut target = [0_u8; 96];
let mut body = [0_u8; 1];
let prepared = request.prepare_bound(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(
    prepared.as_untyped().transport_request().target().as_str(),
    "/ip/192.0.2.10/cancellation"
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

Use the matching server or subnet request types for those identity domains.
POST requires `RobotCancellationSchedule`; server POST also takes an optional
validated reason and explicit `RobotLocationReservationIntent`. DELETE uses a
named revoke request. Server DELETE accepts only an empty `200 OK`; IP and
subnet DELETE require their documented JSON cancellation objects.

With `serde`, read-only code may call `prepare_bound`, validate through the
resulting `PreparedCancellation`, and call `checked.decode_response()`.
Destructive POST and DELETE execution must instead move that prepared value
into `CancellationPlanConfirmation`, create `CancellationDestructivePermit` or
`CancellationSharedDestructivePermit`, and execute its attempt. POST
cancellation form bodies are sensitive, so POST must use
`build_cancellation_plan_digest`; `build_cancellation_canonical_plan` rejects
them with `SensitiveBodyRequiresDigest`. Bodyless DELETE may use either the
exact canonical builder or the strong-digest builder. The attempt's blocking,
Send-async, and local-async methods return `CheckedCancellation` directly,
retaining the exact request instance through authorization and wire execution
without caller rebinding.

POST decoding verifies active state, requested date, reason, and complete
reservation acknowledgement. `Omit` is accepted only when reservation is
unavailable and inactive; `Reserve` requires available and active reservation;
`DoNotReserve` requires inactive reservation. IP/subnet DELETE verifies that
the returned cancellation is inactive.

POST and DELETE are destructive and never automatically retryable. Use the
request-bound cancellation permit API and caller-owned reconciliation after
uncertain delivery. v0.79 does not add the Robot high-level client.

The crate package remains 0.42.0 until cumulative publication at v0.80.0.
