# Migrating Source Users To v0.94.0

v0.94.0 is an additive internal milestone. It is tagged after review but is
not separately published to crates.io. Source users can select the exact tag:

```toml
[dependencies]
cloud-sdk = { git = "https://github.com/valkyoth/cloud-sdk", tag = "v0.94.0" }
cloud-sdk-hetzner = { git = "https://github.com/valkyoth/cloud-sdk", tag = "v0.94.0", version = "0.45.0", features = ["serde"] }
```

## Robot Client

Construct `cloud_sdk_hetzner::client::RobotClient` with a transport that
implements `BoundTransport` and `BoundCredentialTransport`. `official` rejects
every destination except `https://robot-ws.your-server.de/` and captures the
transport credential binding.

All 45 read-only operations provide direct blocking, `Send` async, and
local-async execution. Their output is
`RobotClientResponse<request-specific-success>`, separating checked success
from a strict bounded `RobotFailure`.

All 44 state-changing operations remain permit-gated. Use the existing
operation-specific permit flow where available. Server rename and rescue,
Linux, VNC, and Windows activation/deactivation use the new sealed Robot
mutation plan and permit family.

After any exact or malformed `401`, the current credential generation is
closed. Further calls fail before network access. Call
`reconfirm_credentials` only after explicitly deciding the same credentials
are valid again, or replace the transport with a different credential binding.
The client does not retry automatically.

Robot lists and transaction snapshots are bounded single responses, not
paginated resources. Robot has no Cloud-style action objects. Continue using
the explicit operation-specific reconciliation flows for uncertain mutations
and billable orders.

## Core Status Errors

`PreparedExecutionError` now includes `UnexpectedStatus(StatusCode)` so direct
prepared execution preserves the actual status in all execution modes. Update
exhaustive pre-1.0 matches to handle the new variant. High-level client
diagnostics continue to expose the bounded response-policy category.

## Compatibility

The workspace MSRV remains Rust 1.92.0, default features remain empty, and no
dependency was added. The first crates.io package containing this cumulative
work remains v0.95.0.
