# v0.79.0 Rejected Abstractions

Status: implementation stop reached; pentest required.

## One Generic Cancellation Request

Rejected because a free resource-kind enum would permit method, route, field,
and response-policy confusion. Nine named request types make each source row
and identity domain explicit.

## String Schedules And Addresses

Rejected because arbitrary strings defer destructive target/date validation
until the provider sees the request. Canonical protected address owners and an
immediate-or-date schedule make invalid states unrepresentable.

## Inferred Location Reservation

Rejected because a stale GET response could silently select a destructive POST
field. Callers must choose omit, reserve, or explicitly decline based on their
current reviewed state.

## Automatic Retry For Revoke

Rejected even though DELETE is idempotent. Provider conflicts and uncertain
delivery require read-after-write reconciliation before another action.

## One Permissive Date Field

Rejected because accepting arbitrary aliases would hide upstream drift. The
two documented spellings are source-locked, mutually exclusive, and limited
to IP/subnet responses.

## High-Level Robot Client

Rejected for this milestone. Endpoint-family completion and client integration
have separate review stops; Robot client execution remains v0.94.0.
