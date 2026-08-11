# Migrating Source Users To v0.77.0

v0.77.0 is an internal source milestone. The latest crates.io checkpoint is
v0.75.0, and cumulative publication is deferred to v0.80.0.

## Robot Error Decoding

Source users enabling `cloud-sdk-hetzner/serde` can classify a committed Robot
error response without exposing provider text:

```rust
use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};
use cloud_sdk_hetzner::robot::{RobotDecodeError, RobotFailure, decode_robot_failure};

fn decode(
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    decode_robot_failure(response, workspace)
}
```

Match `RobotFailure::AuthenticationRejected` and report rejection against the
exact `RobotCredentialAttempt` that sent the request. Never retry that
generation. `RobotFailure::QuotaExceeded` exposes a relative interval and a
fallible provider-neutral exhausted bucket. Maintenance and transport failures
still require explicit caller policy.

Do not convert `RobotDecodeError::UnknownCode` or `UnsupportedStatus` into
transient errors. Update the source lock and finite decoder together after an
upstream protocol change.

## Published Dependencies

Crates.io users remain on the v0.75 checkpoint until v0.80:

```toml
[dependencies]
cloud-sdk = "0.75.0"
cloud-sdk-hetzner = "0.42.0"
```
