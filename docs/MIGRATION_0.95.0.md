# Migrating To v0.95.0

v0.95.0 is the cumulative public checkpoint for the v0.91.0-v0.95.0 Robot
work. Update independently versioned crates together when they are used in one
application:

```toml
[dependencies]
cloud-sdk = "0.95.0"
cloud-sdk-hetzner = { version = "0.46.0", features = ["serde"] }
cloud-sdk-reqwest = { version = "0.36.0", features = ["blocking-rustls"] }

[dev-dependencies]
cloud-sdk-testkit = "0.31.0"
```

`cloud-sdk-sanitization 0.19.0` is unchanged and is not republished.

## Cumulative Robot Surface

The provider release adds strict read-only ordering catalogs, bounded 30-day
transaction snapshots, cost-authorized order mutations, all 89 active Robot
operations through `RobotClient`, and one-generation authentication lockout.
Read-only operations execute directly through blocking, `Send` async, and
local-async transports. Every state change remains permit-gated, and Robot
lists remain bounded single responses rather than synthetic pagers.

After authentication rejection, indeterminate post-dispatch failure, or
cancellation, the client requires a different credential binding or explicit
reconfirmation before another wire attempt. Applications using one credential
through multiple clients or processes still need an external credential-keyed
coordinator.

See the source migration details for [v0.91](MIGRATION_0.91.0.md),
[v0.92](MIGRATION_0.92.0.md), [v0.93](MIGRATION_0.93.0.md), and
[v0.94](MIGRATION_0.94.0.md) before updating exhaustive pre-1.0 matches.

## Robot Live Evidence

The repository operator harness adds a separate
`cloud-sdk-hetzner-robot-smoke` launcher. It accepts paths to two distinct
private files through `CLOUD_SDK_HETZNER_ROBOT_USERNAME_FILE` and
`CLOUD_SDK_HETZNER_ROBOT_PASSWORD_FILE` only after a credential-free build and
privileged sealing phase. It can execute only the typed bodyless
`RobotServerListRequest` against the exact official endpoint.

Do not pass Robot credentials to Cargo, CI, command arguments, raw environment
variables, or the Cloud launcher. Do not intentionally test invalid Robot
credentials: three failed logins can block the caller's source IP. Follow
[`LIVE_SMOKE_TESTING.md`](LIVE_SMOKE_TESTING.md) for the complete operator
procedure and cleanup boundaries.

## Compatibility

The MSRV remains Rust 1.92.0, development remains pinned to Rust 1.97.1,
default features remain empty, and the provider default graph remains
transport-free and `no_std`. The live harness is an ignored integration test
behind existing development features and adds no published runtime dependency.
