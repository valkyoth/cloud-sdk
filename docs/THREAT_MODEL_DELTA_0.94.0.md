# Threat Model Delta 0.94.0

Status: implementation stop; incremental pentest required.

## New Assets And Threats

v0.94 centralizes Robot dispatch. Protected assets include credentials,
credential-generation authority, exact official endpoint identity, permit
authority, guarded request and response bytes, provider errors, and the
distinction between requests that were never sent and requests that reached a
response.

Primary threats are credential exfiltration through a custom endpoint,
repeated invalid credentials causing source-IP lockout, a queued stale attempt
executing after another request receives `401`, mutation execution without the
required evidence, unchecked response acceptance, status-code collapse, and
secret retention after decode or cancellation.

## Controls

- Robot construction accepts only the exact official HTTPS endpoint and binds
  every call to the credential lineage observed at construction.
- Each call reserves the generation's only in-flight dispatch guard immediately
  before preparation or permit timing. Admission, rejection, replacement, and
  reconfirmation share one atomic state, so another call cannot pass validation
  while an earlier response remains unclassified.
- An exact or malformed `401`, a transport failure after dispatch may have
  begun, or a response-transaction failure closes the generation. Final status
  survives later header/body/trailer/capacity failures, so an observed `401`
  cannot be lost. Reuse fails before transport access until explicit
  reconfirmation or replacement with a different credential binding.
- Completing classification releases dispatch admission. Cancellation or any
  other unclassified guard drop rejects the generation fail closed.
- Only sealed read-only operations expose direct execution. Every mutation,
  destructive operation, and billable order requires its matching permit.
- Existing specialized evidence routes cannot be converted into the generic
  server/boot mutation permit.
- Checked request-bound decoders enforce status, content type, response size,
  success identity, typed provider failures, and cleanup ownership.
- Exact unexpected status and any final status observed before a transport
  failure are preserved for authentication and delivery classification.
- Replacement endpoint verification retains its typed cause instead of being
  collapsed into credential-binding drift.
- Dropping an unpolled async future admits no attempt, performs no wire call,
  and releases all workspace storage.

## Residual Boundaries

Dispatch serialization and credential rejection are scoped to one
`RobotClient` lifecycle state. Independently constructed clients and separate
processes do not share that state, even when they use the same Robot credential.
Applications must share one client or provide an external credential-keyed
dispatch coordinator when process- or fleet-wide serialization is required.

The SDK cannot determine whether a transport failure after dispatch reached
Robot. The credential generation therefore closes, while existing mutation
delivery and reconciliation policy remains authoritative.
Robot has no revision-bound pagination or Cloud-style action resources;
bounded lists are single snapshots, and callers must serialize conflicting
mutations where the operation-specific documentation requires it.

Explicit credential reconfirmation is a caller security decision. The SDK does
not verify that unchanged credentials were corrected provider-side. Process
abort, allocator exhaustion, transport-owned copies, source credential cleanup,
provider lockout policy, and provider-side authorization remain covered by the
repository threat model.

Operational monitoring should distinguish authentication rejection,
indeterminate delivery, cancellation or another unclassified guard drop, and
concurrent `DispatchBusy`; each requires a different operator response.
