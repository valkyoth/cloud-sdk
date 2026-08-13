# Public API Review 0.87.0

Status: implementation stop; pentest required.

## Core Contract

`PreparedRequest::new_read_only_post_query` is the only new provider-neutral
execution primitive. It accepts only `POST` with already validated read-only,
safe metadata. Ordinary `PreparedRequest::new` continues to reject read-only
POST, and all other POST requests continue to require mutation authority.

## Robot Traffic

`RobotTrafficInterval` owns protected exact bounds and exposes them only through
closures. `RobotTrafficTarget` owns protected canonical IP or subnet-base
identities. `RobotTrafficRequest` rejects empty, excessive, duplicate, and
cross-kind ambiguous sets before preparation.

`PreparedRobotTraffic` and `CheckedRobotTraffic` retain exact request
provenance. No raw public decoder can bypass that association. Result targets,
amounts, points, and reports are non-copyable and redact protected values from
`Debug`. Exact amounts are available only through a closure.

## Compatibility

The provider remains `no_std` with `alloc`/`serde` opt-in. No existing
constructor or feature changes. Raising the incremental parser's per-object
hard ceiling from 256 to 4,096 matches the existing aggregate field ceiling;
all input, token, field, string, number, and depth limits remain enforced.
