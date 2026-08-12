# v0.82.0 Threat Model Delta

Status: implementation stop; pentest required.

## New Boundary

v0.82 admits Robot reset capabilities and can prepare a disruptive server
reset request. A reset can interrupt service or cause data loss. Capability,
address, server, and operating-state data may be operationally sensitive.

## Threats And Controls

### Unauthorized Or Wrong Reset

- Raw decoding is non-authorizing. Execute construction requires an exact
  authenticated detail execution and an explicit type currently advertised
  for that exact server.
- The authorizing state binds an opaque transport credential lineage and a
  fixed 30-second caller-clock lifetime. Credential changes during preflight,
  cross-account transport reuse, stale evidence, and future-dated evidence
  fail closed before destructive network access.
- The finite type prevents arbitrary provider form values; duplicate or
  unknown capabilities fail closed.
- Only execute requests can construct reset plan confirmation. The sensitive
  body requires a strong digest and a destructive permit.
- Execute requests do not implement generic `PrepareOperation`, and their
  typed prepared wrapper does not expose `as_untyped`. The underlying core
  request is permanently marked as requiring authorization evidence; generic
  canonical and digest plan builders reject that marker before authority can
  be created.
- Request, plan, fingerprint, permit, attempt, and checked response retain
  exact type association across blocking, Send-async, and local-async modes.
- Credential binding, both addresses, server number, selected capability,
  observation, and expiry enter digest-only authorization evidence. Permit
  validity cannot exceed evidence expiry, and credential plus time are checked
  again from one clock sample immediately before dispatch.
- Execution is non-idempotent and automatic retry is denied. Uncertain
  delivery requires caller reconciliation.

### Hostile Or Contradictory Responses

- Exact envelopes reject duplicate, unknown, missing, mistyped, oversized,
  noncanonical, and contradictory data.
- Lists and capabilities have hard bounds and duplicate rejection. Deterministic
  tests exercise 4,095, 4,096, and 4,097 entries.
- List, detail, and action success bodies are separately bounded at 2 MiB,
  4 KiB, and 2 KiB before strict JSON allocation.
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

The transport binding contract assumes an implementation returns the lineage
of the credential its authenticated send will use. The admitted reqwest Basic
clients satisfy this with immutable credential state shared by clones. Custom
transports are responsible for equivalent atomicity and CSPRNG-quality binding
generation.

The SDK cannot stop an application that deliberately bypasses the SDK and
constructs a raw HTTP reset request. The enforced boundary prevents the SDK's
generic operation and permit APIs from accidentally erasing reset evidence;
application access to credentials and lower-level transports remains part of
the caller's trust boundary.
