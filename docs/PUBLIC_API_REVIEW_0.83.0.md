# v0.83.0 Public API Review

Status: implementation stop; pentest required.

Scope: changes after signed v0.82.0 through the v0.83.0 implementation stop.

## Requests And Intent

`cloud_sdk_hetzner::robot` adds `RobotFailoverListRequest`,
`RobotFailoverGetRequest`, `RobotFailoverRerouteRequest`, and
`RobotFailoverDeleteRouteRequest`. Constructors accept canonical protected
`RobotIpAddress` values. Reroute construction rejects a destination using a
different address family before any request can be prepared.

Every request fixes the official Robot endpoint, Basic-auth scope, operation
ID, method, checked JSON response policy, and source quota. Reads are safe and
explicit-policy retryable. Reroute is non-idempotent mutation and deletion is
non-idempotent destructive intent; both deny automatic retry.

## Models And Association

The `serde` feature adds `RobotFailover` and `RobotFailoverList`. The model
retains the route prefix, owner IPv4, owner IPv6 network, positive server
number, and nullable active destination. Address data stays in protected
allocation-backed storage and all public diagnostics are redacted.

`PreparedRobotFailover` and `CheckedRobotFailover` preserve exact request
association. List responses are bounded to 4,096 distinct route identities.
`MAX_ROBOT_FAILOVER_LIST_RESPONSE_BYTES` and
`MAX_ROBOT_FAILOVER_ITEM_RESPONSE_BYTES` expose the exact 2 MiB and 16 KiB
decoder limits. Both request-bound and free decoding enforce those limits.
All envelopes require exactly the six source-locked fields. Route masks must
be family-matched and contiguous, route host bits are rejected, owner address
families are fixed, and non-null destinations must match the route family.

Reroute decoding requires the exact requested active destination. Delete
decoding requires `active_server_ip: null`. Both require the exact requested
route. Operation-specific failure decoding admits only documented status/code
pairs.

## Execution Authority

Reroute and delete requests implement sealed `RobotFailoverPermitRequest`.
`RobotFailoverPlanConfirmation` retains request provenance through exact or
strong-digest fingerprints. Sensitive reroute forms reject exact fingerprints
and require `build_robot_failover_plan_digest`.

Direct/shared mutation and destructive permits preserve the exact request
through blocking, Send-async, and local-async execution. Their impact scopes
are not interchangeable. Uncertain delivery consumes authority under the
common permit lifecycle and never causes an automatic retry.

## Semver And Publication

This is additive pre-1.0 provider API. `cloud-sdk` source advances to v0.83.0;
`cloud-sdk-hetzner` remains package version 0.43.0 while cumulative code waits
for v0.85.0. No package is selected for v0.83 publication.

## Explicit Non-Claims

v0.83 does not prove that a destination server is operational, serialize
concurrent provider-side route transitions, reconcile ambiguous delivery, or
execute live failover mutations. Wake-on-LAN remains assigned to v0.84.0.
