# v0.85.0 Threat Model Delta

Status: implementation stop; pentest required.

## New Boundary

v0.85 can prepare Robot boot mutations. Linux, VNC, and Windows activation can
cause data loss when a server reboots into an installer. Rescue activation and
all deactivations alter operational recovery state. Successful responses may
contain generated passwords and private host material.

## Threats And Controls

### Wrong Server Or Configuration

- Paths accept only canonical positive server numbers; deprecated IPv4 aliases
  are unrepresentable.
- Selectors, languages, keyboard layouts, and keys are bounded before form
  construction. Duplicate authorized-key fingerprints fail closed.
- Typed checked responses retain the exact request. Identity, family,
  response shape, selector, language, and final active state must match that
  request.
- Overview, current, last, activation, and deactivation responses enforce
  separate state invariants. Overview responses reject multiple active boot
  families and narrowly admit an inactive Windows null language.

### Duplicate Or Ambiguous Mutation

- Every mutation is non-idempotent and has `RetryEligibility::Never`.
- Linux, VNC, and Windows activation carries destructive metadata. Callers
  must reconcile current Robot state after uncertain delivery before another
  request.
- The SDK performs no implicit reboot, retry, fallback selection, or provider
  option guessing.

### Hostile Provider Data

- Success bodies are independently capped at 1 MiB and decoded with bounded
  allocation.
- Unknown, duplicate, missing, malformed, noncanonical, cross-family,
  oversized, contradictory, and identity-substituted values are rejected.
  Active current state requires a generated password and exact selected
  configuration; last-operation state retains an exact selection even after
  deactivation.
- Deprecated output fields are narrowly admitted only where source-locked,
  validated, and discarded.
- Failure decoding narrows source-locked codes by operation and status.

### Secret Lifetime

- Generated passwords, authorized keys, host keys, and provider selectors use
  cleanup-owning protected strings with redacted diagnostics and closure-only
  access.
- Sensitive forms are pre-cleared, atomically constructed, and cleared on
  preparation failure. Borrowed request values, transport copies, and any
  caller-created scalar copies remain caller cleanup boundaries.

## Residual Boundaries

Robot acceptance does not prove a future reboot, boot selection, installation
success, retained data, or eventual server health. Credential custody,
endpoint-bound transport, caller authorization policy, allocator/process abort,
reboot control, and live destructive testing remain outside this milestone.
