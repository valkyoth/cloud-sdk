# v0.82.0 Public API Review

Status: implementation stop; pentest required.

Scope: changes after signed v0.81.0 through the v0.82.0 implementation stop.

## Requests And Capabilities

`cloud_sdk_hetzner::robot` adds `RobotResetListRequest`,
`RobotResetGetRequest`, and `RobotResetExecuteRequest`. The read requests admit
only the exact `/reset` and `/reset/{server-number}` routes. Execution uses
POST on the latter route with one bounded `type` form field.

`RobotResetType` is finite: software, hardware, power, long power, or manual.
`RobotResetIntent` makes disruptive caller choice explicit.
Raw decoding returns a non-authorizing `RobotReset`. Only
`PreparedRobotReset<RobotResetGetRequest>::execute_authorizing_*` can produce
`AuthorizedRobotReset`, after exact authenticated transport execution. That
state binds the transport credential lineage and a fixed 30-second caller-clock
observation window. `RobotResetExecuteRequest::from_checked` accepts only that
authorizing type and rejects a capability it does not advertise. There is no
raw-detail or server-number-only execute constructor.

Every request fixes the official Robot endpoint, Basic-auth scope, operation
ID, method, response policy, and source quota. Read operations are safe and
idempotent. Execution is destructive, non-idempotent, sensitive-body, and
never automatically retryable.

Unlike the two read requests, `RobotResetExecuteRequest` deliberately does not
implement `PrepareOperation`. It exposes only `prepare_bound`, and
`PreparedRobotReset<RobotResetExecuteRequest>` deliberately has no
`as_untyped`. The core `PreparedRequest` retained inside that wrapper carries
an irreversible authorization-evidence requirement.

List, detail, and action success bodies have separate 2 MiB, 4 KiB, and 2 KiB
limits. The list allowance is 512 bytes for each of the maximum 4,096 items.

## Models And Association

The `serde` feature adds `RobotResetSummary`, `RobotReset`, `RobotResetList`,
`RobotResetAction`, and `RobotResetOperatingStatus`. Models use protected
address, number, and status storage. Lists are bounded and duplicate-free by
server number. Capability lists are nonempty, finite, and duplicate-free.

`PreparedRobotReset` and `CheckedRobotReset` preserve exact request
association. Detail decoding binds the returned server number to the request.
Action decoding binds IPv4, IPv6 network, any returned server number, and the
exact selected reset type to checked preflight state. The action number alone
is optional because the official example omits it while the table requires
it.

Unknown, duplicate, missing, mistyped, noncanonical, oversized, contradictory,
and cross-request response data fail closed. Exact request-specific failure
decoders admit only the documented status/code pairs.

## Destructive Authority

`RobotResetPlanConfirmation::new` is available only for
`RobotResetExecuteRequest`; list and detail requests cannot enter the reset
permit boundary. Sensitive forms reject exact canonical fingerprints and
require `build_robot_reset_plan_digest` with an admitted strong hasher.

Request-bound direct and shared destructive permits preserve the exact execute
request through blocking, Send-async, and local-async execution. Uncertain
delivery consumes authority according to the common permit lifecycle and does
not trigger an automatic retry.

The core crate adds opaque `CredentialBinding` and `BoundCredentialTransport`
contracts. The reqwest Basic transports generate a 256-bit binding with the
admitted operating-system CSPRNG and preserve it across clones. Reset evidence
places credential binding, server identities, capability, observation, and
expiry only in the strong digest. Permit validity cannot outlive the evidence;
credential lineage and expiry are rechecked immediately before dispatch.
It also adds `PreparedRequest::with_required_authorization_evidence` and
`PlanFingerprintBuildError::AuthorizationEvidenceRequired`. Generic exact and
digest plan builders reject marked requests; only the evidence-aware digest
builder can construct their plan fingerprint.

The marker does not alter ordinary `cloud-sdk/plan-confirm/v1` bytes.
Evidence-required requests use `cloud-sdk/plan-confirm/v2`, where field 31
explicitly binds the requirement. Fixed SHA-256 golden vectors lock both
complete canonical inputs.

## Semver And Publication

This is additive pre-1.0 provider API. `cloud-sdk` source advances to v0.82.0;
`cloud-sdk-hetzner` remains package version 0.43.0 while cumulative code waits
for v0.85.0. No package is selected for v0.82 publication.

## Explicit Non-Claims

v0.82 does not add a Robot high-level client, live reset execution, automatic
reconciliation, deprecated server-IP reset aliases, or undocumented reset
types. Failover remains assigned to v0.83.0.
