# v0.79.0 Threat Model Delta

Status: implementation stop reached; pentest required.

## New Boundary

v0.79 prepares destructive Robot cancellation scheduling and revocation and
admits bounded cancellation state for servers, individual addresses, and
subnets. Identities, dates, reasons, server names, and reservation state may be
operationally sensitive.

## Threats And Controls

### Wrong Target Or Unintended Destruction

- Paths accept only canonical protected server, IP, or subnet identities.
- Request types fix resource family, method, route, and operation ID.
- POST requires explicit schedule; server POST requires explicit reservation
  intent and does not infer it from an earlier response.
- POST and DELETE are destructive and never automatically retryable. POST is
  non-idempotent; DELETE remains retry-denied despite idempotent semantics.
- The request-bound cancellation plan, fingerprint, and direct/shared permit
  wrappers apply the core permit boundary before execution and retain the exact
  request through blocking, Send-async, and local-async response admission.
- No public constructor can manufacture or replace the private request
  association. Permit execution returns `CheckedCancellation` directly.

### Hostile Or Contradictory Responses

- GET/POST and IP/subnet DELETE require checked `200` JSON; server DELETE
  requires checked empty `200`.
- Exact envelopes reject unknown and duplicate fields.
- Response identity must equal the request identity.
- `PreparedCancellation` and `CheckedCancellation` retain the exact request
  type and instance through direct validation and permit-authorized execution,
  preventing bare-guard decoder use.
- POST acknowledgement must be active and match the requested exact date,
  optional reason, and reservation intent. Immediate schedules require an
  active provider date. IP/subnet DELETE acknowledgement must be inactive.
- Date presence must equal cancellation state and cannot precede the earliest
  date; dates are calendar-valid.
- Reservation intent is acknowledged exactly: omission requires unavailable
  and inactive reservation, reserve requires available and active reservation,
  and explicit non-reservation requires inactive reservation.
- Server reason shape changes exactly from bounded array to string-or-null.
- IP/subnet spelling variants are mutually exclusive, and subnet host bits
  must be clear for the returned prefix.

### Data Lifetime

- Canonical identities and dates live in stable `SecretBoxBytes` allocations
  with redacted diagnostics and closure-scoped access.
- IP canonicalization streams formatted text into an equality sink instead of
  constructing an ordinary `String` copy.
- Request preparation pre-clears caller path/body storage and clears both on
  reachable validation or capacity failure. POST bodies are marked sensitive.
- Checked decode methods consume the cleanup-owning, request-associated guard.
- Request and response display text share one control, bidi, isolate, and
  zero-width rejection policy.
- Protected date allocation failures remain distinct from malformed dates.

## Residual Boundaries

The official source inconsistently uses `cancellation_date` and
`cancellation-date`, and string versus number server identity. The decoder
admits only the exact reviewed alternatives and rejects ambiguity. Callers
must reconcile uncertain destructive delivery before issuing another action.
No live mutation, automatic retry, or high-level Robot client is introduced.
