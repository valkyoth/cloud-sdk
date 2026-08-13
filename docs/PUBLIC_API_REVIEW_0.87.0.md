# Public API Review 0.87.0

Status: implementation stop; pentest required.

## Core Contract

`PreparedRequest::new_read_only_post_query` requires an
`ApprovedReadOnlyPostQuery` selected from a closed core registry. The Robot
traffic entry validates the exact provider, service, official endpoint,
authentication scope, method, target, required headers, non-empty body, and
sensitive-body classification plus complete safety metadata. It also binds
`robot_get_traffic` during construction. The selector cannot approve
caller-defined operations.

`PreparedRequest::with_operation_id` retains its existing behavior for ordinary
requests but cannot replace the operation identity installed by a closed
approval. Ordinary `PreparedRequest::new` continues to reject read-only POST,
and all unapproved POST requests continue to require mutation authority.
Permit checks revalidate the retained approval against the current request, so
an internal header transformation cannot carry stale permitless authority.

## Robot Traffic

`RobotTrafficInterval` owns protected exact bounds and exposes them only through
closures. `RobotTrafficTarget` owns protected canonical IP or subnet-base
identities. `RobotTrafficRequest` sorts targets by canonical address and kind,
then rejects adjacent duplicate or cross-kind ambiguous identities before
preparation. Response binding uses binary lookup into that canonical set and a
bounded seen bitmap instead of repeated linear scans.

`PreparedRobotTraffic` and `CheckedRobotTraffic` retain exact request
provenance. No raw public decoder can bypass that association. Result targets,
amounts, points, and reports are non-copyable and redact protected values from
`Debug`. Exact amounts are available only through a closure.

## Compatibility

The provider remains `no_std` with `alloc`/`serde` opt-in. No existing
constructor or feature changes. Raising the incremental parser's per-object
hard ceiling from 256 to 4,096 matches the existing aggregate field ceiling;
all input, token, field, string, number, and depth limits remain enforced.
