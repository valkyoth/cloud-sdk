# v0.84.0 Public API Review

Status: implementation stop; pentest required.

Scope: changes after signed v0.83.0 through the v0.84.0 implementation stop.

## Requests And Intent

`cloud_sdk_hetzner::robot` adds `RobotWolGetRequest` for authenticated
capability discovery and `RobotWolSendRequest` for packet sending. Both use a
canonical positive `RobotServerNumber`; no API accepts the deprecated server
IPv4 path alias.

`RobotWolSendRequest::from_checked` requires short-lived
`AuthorizedRobotWol` plus explicit `RobotWolIntent::Send`. The send type does
not implement generic `PrepareOperation`, and its prepared wrapper does not
expose `as_untyped`. Every request fixes the official Robot endpoint,
Basic-auth scope, operation ID, method, checked JSON response policy, and
source quota. `RobotWolQuota`, `ROBOT_WOL_DISCOVERY_QUOTA`, and
`ROBOT_WOL_SEND_QUOTA` expose the documented allowances without acquiring a
clock, sleeping, or choosing caller account scope.

## Models And Association

`RobotWol` retains canonical main IPv4, IPv6 network, and positive server
number identities in protected storage with redacted diagnostics. The decoder
requires exactly those three fields inside the `wol` envelope and rejects
unknown, missing, duplicate, malformed, noncanonical, wrong-family, or
request-mismatched values. Its exported free decoder independently enforces
the 16 KiB source-specific body limit.

`PreparedRobotWol` and `CheckedRobotWol` preserve exact request association.
Both discovery and send success must return `200` JSON with a nonempty body.
Send acknowledgements must equal all three identity values included in the
authenticated evidence and strong plan digest.
Operation-specific failure decoding admits only `SERVER_NOT_FOUND`,
`WOL_NOT_AVAILABLE`, and send-only `WOL_FAILED` under their documented status
codes.

## Execution Authority

Only execution of the exact authenticated discovery request can construct
`AuthorizedRobotWol`. Evidence binds server identity, opaque transport
credential lineage, observation time, and an exclusive 30-second expiry.
Plans require a strong digest containing that evidence. Dispatch rechecks the
same credential and freshness before any blocking, Send-async, or local-async
transport call.

Sending is non-idempotent mutation intent with automatic retries disabled.
`RobotWolMutationPermit` and `RobotWolSharedMutationPermit` retain request and
response provenance through the common fail-closed lifecycle.

## Semver And Publication

This is additive pre-1.0 provider API. `cloud-sdk` source advances to v0.84.0;
`cloud-sdk-hetzner` remains package version 0.43.0 while cumulative code waits
for v0.85.0. No package is selected for v0.84 publication.

## Explicit Non-Claims

The SDK does not prove that a powered-off server will receive or honor the
packet, infer power state, retry ambiguous delivery, or reconcile whether the
provider sent a packet after transport failure. Boot configuration remains
assigned to v0.85.0.
