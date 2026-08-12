# v0.83.0 Threat Model Delta

Status: implementation stop; pentest required.

## New Boundary

v0.83 admits Robot failover topology and can prepare requests that move or
remove an active route. A wrong target or duplicate transition can interrupt
service, expose traffic to the wrong server, or make a failover address
unreachable. Route and owner topology may be operationally sensitive.

## Threats And Controls

### Wrong Or Widened Route

- Request addresses are canonical protected IP values. Reroute construction
  rejects cross-family route/destination pairs.
- Response masks must be contiguous and family-matched. Route host bits are
  rejected, preventing a noncanonical identity from silently widening.
- Owner IPv4 and IPv6 fields have fixed families; a non-null active target
  must use the route family.
- Checked detail and mutation responses must match the exact request route.
  Reroute must echo the exact destination; deletion must return a null target.

### Unauthorized Or Replayed Mutation

- Reroute is mutation authority and deletion is destructive authority. Their
  permit scopes cannot be substituted.
- Sensitive reroute forms require collision-resistant plan digests. Plans,
  permits, attempts, and responses retain exact request provenance.
- Both transitions are non-idempotent and never automatically retried.
  Uncertain delivery requires external reconciliation.

### Hostile Or Contradictory Responses

- Exact envelopes reject duplicate, unknown, missing, mistyped, malformed,
  noncanonical, cross-family, and contradictory data.
- Lists are capped at 4,096 distinct routes and list bodies at 2 MiB. Item
  bodies are capped at 16 KiB before strict JSON allocation.
- Provider failures are admitted only for the documented operation and status;
  cross-operation conflict codes fail closed.
- DELETE requires the official JSON object with `active_server_ip: null`;
  empty/no-content responses do not bypass outcome verification.

### Data Lifetime

- Addresses and server numbers use protected ownership and redacted
  diagnostics.
- Preparation pre-clears caller target/body storage and clears it after any
  failure. The reroute body is marked sensitive.
- Strong-digest scratch is cleared by the common plan builder, checked decoding
  consumes the cleanup-owning response guard, and unpolled attempts use the
  common response-buffer cleanup boundary.

## Residual Boundaries

The SDK cannot determine whether all possible destination servers are
correctly configured for the failover address, as required by Hetzner, or
whether a destination is healthy. It cannot serialize independent processes
changing the same route. Callers must coordinate mutations, verify target
configuration, and reconcile ambiguous delivery before another transition.

This milestone adds no live destructive failover test. Credential custody,
transport endpoint binding, process abort, allocator exhaustion, and caller-
owned copies retain their existing documented boundaries.
