# v0.84.0 Threat Model Delta

Status: release review complete; pentest and final retest passed.

## New Boundary

v0.84 can prepare a Robot request that asks provider infrastructure to send a
Wake-on-LAN packet. An unintended, duplicated, or misdirected send can power
on hardware, alter operational state, consume quota, or disrupt maintenance.
The response also contains sensitive server topology.

## Threats And Controls

### Wrong Server Or Unsupported Capability

- Paths accept only canonical positive server numbers. The deprecated IPv4
  alias is absent from constructors and encoders.
- Raw response decoding is non-authorizing. Only authenticated execution of
  the exact discovery operation mints `AuthorizedRobotWol`.
- Strict decoding requires fixed address families and the exact requested
  server number under an exact three-field envelope. Mutation acknowledgement
  decoding additionally requires both addresses to equal the authenticated
  discovery identity included in the plan digest.

### Stale Or Replayed Authorization

- Capability evidence is tied to opaque transport credential lineage and
  expires after 30 seconds. Both properties are rechecked immediately before
  transport dispatch.
- Evidence is included in a collision-resistant plan digest. Generic plan
  construction without evidence fails closed.
- Sending requires direct or shared mutation authority. The request is
  non-idempotent, permits one attempt, and is never automatically retried.
  Unknown delivery consumes authority and requires caller reconciliation.
- Request types expose the documented 500 discovery/hour and 10 send/hour
  allowances as machine-readable metadata. Enforcement, scope selection,
  sleeping, and clocks remain explicit caller policy.

### Hostile Responses And Failure Widening

- Success decoding rejects duplicate, unknown, missing, mistyped, malformed,
  noncanonical, cross-family, oversized, and identity-substituted data.
- The decoder independently repeats the 16 KiB limit after checked-response
  admission.
- Failure decoding narrows documented codes per operation. `WOL_FAILED` is
  admitted only for sending, never discovery.

### Data Lifetime

- Server addresses and number use protected ownership and redacted
  diagnostics. Plans, permits, attempts, and responses retain exact request
  association.
- Preparation pre-clears caller target/body storage and clears both on every
  failure. Plan scratch and response guards use existing cleanup boundaries.

## Residual Boundaries

Robot acknowledgement proves only that its endpoint accepted the request and
returned the expected identity. It does not prove packet delivery, server
power state, network reachability, or successful boot. After uncertain
delivery callers must observe authoritative state before another send.

Credential custody, transport endpoint binding, process abort, allocator
exhaustion, and caller-owned copies retain their existing documented
boundaries. This milestone adds no live mutating WOL test.
