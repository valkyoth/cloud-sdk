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
- Each call admits one credential attempt and revalidates it immediately before
  dispatch. A concurrent rejection therefore prevents later queued sends.
- An exact or malformed `401` closes the generation after the first wire
  attempt. Reuse fails before transport access until explicit reconfirmation or
  replacement with a different credential binding.
- Only sealed read-only operations expose direct execution. Every mutation,
  destructive operation, and billable order requires its matching permit.
- Existing specialized evidence routes cannot be converted into the generic
  server/boot mutation permit.
- Checked request-bound decoders enforce status, content type, response size,
  success identity, typed provider failures, and cleanup ownership.
- Exact unexpected status is preserved in every execution mode for correct
  authentication and delivery classification.
- Dropping an unpolled async future admits no attempt, performs no wire call,
  and releases all workspace storage.

## Residual Boundaries

The SDK cannot determine whether a transport failure after dispatch reached
Robot. Existing delivery and reconciliation policy remains authoritative.
Robot has no revision-bound pagination or Cloud-style action resources;
bounded lists are single snapshots, and callers must serialize conflicting
mutations where the operation-specific documentation requires it.

Explicit credential reconfirmation is a caller security decision. The SDK does
not verify that unchanged credentials were corrected provider-side. Process
abort, allocator exhaustion, transport-owned copies, source credential cleanup,
provider lockout policy, and provider-side authorization remain covered by the
repository threat model.
