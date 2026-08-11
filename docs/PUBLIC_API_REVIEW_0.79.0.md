# v0.79.0 Public API Review

Status: implementation stop reached; pentest required.

Scope: changes from signed v0.78.0 through the v0.79.0 implementation stop.

## Allocation-Gated Requests

`cloud_sdk_hetzner::robot` adds nine named cancellation requests covering GET,
POST, and DELETE for server, IP, and subnet identities. `RobotIpAddress`,
`RobotSubnetAddress`, and `RobotCancellationDate` use fallible stable protected
storage, redacted diagnostics, canonical grammar, and closure-scoped access.

POST accepts `RobotCancellationSchedule::Immediate` or `On(date)`. Server POST
also accepts a bounded `RobotCancellationReason` and requires an explicit
`RobotLocationReservationIntent`, including explicit omission when reservation
is unavailable. No arbitrary form field, route, method, or endpoint is public.

All requests implement `PrepareOperation`, bind the official Robot endpoint,
service, and Basic scope, and carry operation IDs plus exact response policy.
Create and revoke metadata is destructive and never retry eligible. The
provider-neutral execution layer therefore requires a destructive permit.

## Serde Response Models

The `serde` feature adds `RobotServerCancellation`, `RobotIpCancellation`,
`RobotSubnetCancellation`, `RobotServerCancellationReason`,
`PreparedCancellation`, and `CheckedCancellation`. `prepare_bound` retains the
exact request type and instance through response-policy validation; only the
matching checked type exposes its decoder. Standalone low-level decoders still
require a checked response plus expected identity and decoder workspace.

Destructive execution uses `CancellationPlanConfirmation`, the exact or digest
fingerprint builders, `CancellationDestructivePermit` or
`CancellationSharedDestructivePermit`, and `CancellationPermitAttempt`. These
wrappers carry the private exact-request binding beside the provider-neutral
plan and permit state. Their blocking, Send-async, and local-async execute
methods return `CheckedCancellation` directly, so the authorized path never
erases response provenance or asks callers to reconstruct it.

Models preserve protected IDs, addresses, dates, names, reasons, and state.
They reject unknown fields, identity mismatch, impossible date presence,
scheduled dates before the earliest date, invalid reservation combinations,
wrong reason shape, noncanonical subnet host bits, and both or neither of the
two officially documented IP/subnet date-field spellings.

Mutation decoders additionally require POST acknowledgement to match active
schedule, exact requested date when supplied, server reason, and reservation
intent. Omission requires reservation to be unavailable and inactive; reserve
requires available and active reservation; explicit non-reservation requires
inactive reservation. IP/subnet DELETE returns and validates the documented
inactive JSON model. Server DELETE remains the only empty success response.

## Semver And Publication

This is additive pre-1.0 provider API. `cloud-sdk` source advances to v0.79.0;
`cloud-sdk-hetzner` remains package version 0.42.0 while cumulative code waits
for v0.80.0. No package is selected for v0.79 publication.

## Explicit Non-Claims

v0.79 does not add a Robot high-level client, authorization-header encoding,
network execution, automatic retry, live mutation, or general IP/subnet
management. Client integration remains assigned to v0.94.0.
