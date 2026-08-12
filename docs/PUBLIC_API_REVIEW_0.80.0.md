# v0.80.0 Public API Review

Status: implementation stop reached; pentest required.

Scope: cumulative public changes from signed v0.75.0 through the v0.80.0
implementation stop.

## Robot IP API

`cloud_sdk_hetzner::robot` adds six named request types covering list, detail,
traffic-policy update, and separate-MAC get, generate, and delete. The list
request accepts either all addresses or one canonical IPv4 server-address
filter. The fallible filter constructor rejects IPv6 before transport. The
update request requires a non-empty `RobotIpTrafficUpdate`, preventing an empty
mutation while preserving explicit partial updates.

`RobotMacAddress` admits only canonical lowercase EUI-48 text and provides
closure-scoped inspection. With `serde`, `RobotIpSummary`, `RobotIp`,
`RobotIpList`, `RobotIpMac`, and `RobotIpTrafficPolicy` expose typed bounded
results without public payload-bearing diagnostics. List and detail models
retain assignment, server, lock, warning threshold, network, and separate-MAC
state. MAC equality is constant-time, and list duplicate detection uses only
fallible sorted index scratch rather than copying protected identities.

All requests implement `PrepareOperation` and bind the official Robot origin,
Robot service, Basic scope, exact method/path/query/form, operation metadata,
and checked `200` JSON policy. Reads are safe and explicitly retryable. Traffic
update is idempotent but requires explicit retry policy. MAC generation is
non-idempotent and MAC deletion is destructive; both deny automatic retry.

## Response And Permit Association

With `serde`, `prepare_bound` returns `PreparedRobotIp<R>` and checked
admission returns `CheckedRobotIp<R>`. Only the exact request type exposes the
matching decoder. List filters, resource identity, requested traffic fields,
and expected nullable-MAC state remain bound through decoding.

Mutations use `RobotIpPlanConfirmation`, exact or digest fingerprints,
direct/shared mutation or destructive permits, and `RobotIpPermitAttempt`.
Blocking, Send-async, and local-async execution return the request-bound
checked response directly. Sensitive traffic forms require the digest builder;
bodyless MAC operations permit exact canonical fingerprints.

## Cumulative Publication

The public checkpoint also publishes the accumulated v0.76-v0.79 protected
Robot credentials, lockout-aware attempts, strict error/quota protocol, server
operations, and cancellation operations. The neutral core's accumulated
authentication-attempt changes and sanitization's protected fixed-byte
re-export are therefore included. Reqwest and testkit patch only their exact
internal core dependency.

## Semver And Non-Claims

All changes are additive pre-1.0 API. This milestone does not add a high-level
Robot client, authorization-header encoding, automatic retry, live mutation,
or later Robot subnet/reset/failover families. Network execution remains an
explicit optional adapter boundary.
