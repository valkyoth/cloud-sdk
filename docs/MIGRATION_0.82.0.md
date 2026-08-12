# Migrating Source Users To v0.82.0

v0.82 is additive. Existing v0.81 code does not require changes. The published
crate versions remain `cloud-sdk 0.80.0` and `cloud-sdk-hetzner 0.43.0`; v0.82
is a source milestone and crates.io publication is deferred to v0.85.0.

## Preparing A Reset Read

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::robot::{RobotResetGetRequest, RobotServerNumber};

let number = RobotServerNumber::new(321)?;
let request = RobotResetGetRequest::new(number);
let mut target = [0_u8; 64];
let mut body = [0_u8; 1];
let prepared = request.prepare_bound(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(
    prepared.as_untyped().transport_request().target().as_str(),
    "/reset/321"
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

With `serde`, execute the exact prepared detail through a transport that
implements `BoundCredentialTransport`. The authorizing helper validates and
decodes the response, samples the caller clock, and returns a 30-second
`AuthorizedRobotReset` tied to that credential lineage:

```rust,ignore
let authorized = prepared.execute_authorizing_blocking(
    &clock,
    &transport,
    &mut response_body,
    &mut response_headers,
)?;
let execute = RobotResetExecuteRequest::from_checked(
    &authorized,
    RobotResetIntent::Execute(RobotResetType::Hardware),
)?;
```

An unadvertised type returns `UnsupportedCapability`. Prepare the execute
request with `prepare_bound`, build a strong-digest plan with
`build_robot_reset_plan_digest`, and execute it through
`RobotResetDestructivePermit` or `RobotResetSharedDestructivePermit`. Exact
fingerprints are rejected because the body is sensitive. Automatic retry is
never permitted; reconcile uncertain delivery before authorizing another
reset.

The plan validity must end no later than `authorized.expires_at()`, and
dispatch fails before network access if the evidence expires or the transport
credential lineage differs. `decode_robot_reset` remains public for inspection
and custom response handling, but its `RobotReset` result cannot construct an
execute request.

Reset execution intentionally cannot be passed to generic helpers requiring
`PrepareOperation`, and its typed prepared wrapper cannot be converted with
`as_untyped()`. Keep it inside `RobotResetPlanConfirmation` and the reset
permit types. Generic plan construction also rejects the retained core marker
with `AuthorizationEvidenceRequired` as defense in depth.

For non-success responses, call `decode_failure` on the exact request. It
admits only that operation's source-locked status/code combinations.
