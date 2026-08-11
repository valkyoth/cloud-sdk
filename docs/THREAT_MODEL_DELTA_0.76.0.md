# v0.76.0 Threat Model Delta

Status: implementation complete; pentest required.

## New Boundary

v0.76 admits Robot usernames and passwords into protected owned storage and
tracks whether one credential generation remains safe to execute after an
authentication response. Robot warns that three failed logins block the
caller's source IP for ten minutes.

## Threats And Controls

### Credential Cross-Use

- Robot credentials have a provider-owned nominal type and immutable scope.
- Provider, `robot` service, HTTPS host, port 443, and `/` base path are fixed.
- Cloud, DNS, Security, Storage, custom-host, alias, HTTP, port, and base-path
  policies reject the Robot scope.
- Secret parts have no unrestricted accessor or conversion into bearer tokens.

### Repeated Rejected Credentials

- Every credential owner begins with one open nonzero generation.
- Every attempt borrows the exact issuing state and carries the generation
  that began execution.
- Validation and rejection check owner identity before status or generation;
  a foreign attempt fails even when both owners have equal generations.
- Authentication rejection atomically and idempotently closes that generation
  for all later execution and secret access.
- Only newly supplied replacement material or a consumed explicit
  `CredentialReconfirmation` opens another generation.
- Reconfirmation of an open generation fails, preventing a caller from
  advancing unchanged credentials ahead of a concurrent rejection.
- Stale rejection, replacement, and reconfirmation transitions fail closed;
  generation exhaustion never wraps.

Already in-flight concurrent attempts cannot be recalled after one response
reports rejection. Callers must bound concurrency, and later Robot clients
must propagate the first authentication rejection without retry, pager,
poller, or workflow repetition.

### Secret Lifetime And Diagnostics

- Mutable and guarded username/password sources clear completely on success or
  rejection during ingestion and rotation.
- Username, password, and retired replacement allocations use the reviewed
  protected string type and clear complete allocation capacity on drop.
- Invalid replacement material leaves the existing secrets and generation
  unchanged.
- Closure-scoped access revalidates the generation and cannot return a borrow.
- The attempt lifetime prevents safe code from detaching owner identity from
  the generation token.
- Debug, Display, and error source chains contain only public state and static
  messages.

Immutable source copies, allocator internals, intentionally copied closure
values, process dumps, transport/TLS/kernel buffers, and provider-side storage
remain operational boundaries.

### Live Lockout Testing

No v0.76 test sends a credential. The source fixture verifies HTTP 401, three
failures, and a 600-second source-IP lockout without calling Robot. Live tests
must never intentionally submit invalid credentials.

## Unchanged Boundaries

Default crates remain `no_std`, allocation-free, transport-free, runtime-free,
filesystem-free, clock-free, and unsafe-free. v0.76 adds no authorization
encoding, response classification, retry, endpoint operation, client, or live
execution.
