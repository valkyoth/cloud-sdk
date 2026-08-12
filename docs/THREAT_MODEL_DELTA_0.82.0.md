# v0.82.0 Threat Model Delta

Status: implementation stop; pentest required.

## New Boundary

v0.82 admits Robot reset capabilities and can prepare a disruptive server
reset request. A reset can interrupt service or cause data loss. Capability,
address, server, and operating-state data may be operationally sensitive.

## Threats And Controls

### Unauthorized Or Wrong Reset

- Execute construction requires checked provider detail and an explicit type
  currently advertised for that exact server.
- The finite type prevents arbitrary provider form values; duplicate or
  unknown capabilities fail closed.
- Only execute requests can construct reset plan confirmation. The sensitive
  body requires a strong digest and a destructive permit.
- Request, plan, fingerprint, permit, attempt, and checked response retain
  exact type association across blocking, Send-async, and local-async modes.
- Execution is non-idempotent and automatic retry is denied. Uncertain
  delivery requires caller reconciliation.

### Hostile Or Contradictory Responses

- Exact envelopes reject duplicate, unknown, missing, mistyped, oversized,
  noncanonical, and contradictory data.
- Lists and capabilities have hard bounds and duplicate rejection.
- Detail server identity must equal the requested server number.
- Action IPv4, IPv6 network, optional server number, and reset type must equal
  checked preflight state and caller intent.
- Provider failures are admitted only for the exact documented operation and
  status; cross-operation reset codes fail closed.
- The official action example's missing `server_number` is the only narrowly
  admitted output-table inconsistency.

### Data Lifetime

- Addresses, server numbers, and operating status use protected ownership and
  redacted diagnostics.
- Preparation pre-clears caller storage and clears target/body after failure.
- Sensitive form bytes require digest-only plan evidence; scratch and output
  cleanup follow the common plan builder contract.
- Checked decoding consumes the cleanup-owning response guard.
- Unpolled execution clears caller response buffers through the common permit
  attempt boundary.

## Residual Boundaries

The SDK cannot determine whether a provider-advertised reset type is
operationally appropriate, whether the guest has flushed storage, or whether
the provider accepted an uncertain request. Callers must confirm target and
impact, coordinate workload shutdown, and reconcile uncertain delivery before
issuing another reset. This milestone adds no live destructive test.
