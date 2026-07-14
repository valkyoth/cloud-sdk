# Security Recipes

This guide records application responsibilities at the SDK boundary. It does
not replace the repository [threat model](threat-model.md) or provider account
security controls.

## Token Handling

- Create a dedicated project token with the minimum required permission.
- Generate tokens through the provider or another cryptographically secure
  process; do not invent token values from predictable application state.
- Do not place tokens in source, command arguments, URLs, logs, or ordinary
  environment variables.
- Prefer a private regular file, protected secret manager, or platform-native
  credential channel with explicit ownership and permissions.
- Rotate and revoke tokens on a defined schedule and immediately after
  suspected exposure.
- Clear caller-owned mutable secret buffers after the transport no longer
  needs them. `cloud-sdk-sanitization::SecretBuffer` can guard those buffers,
  but cannot clear immutable strings, operating-system copies, remote copies,
  swap, or crash dumps.

## Logging

Log stable operation names and payload-free error variants. Do not log bearer
tokens, authorization headers, request or response bodies, complete request
targets, DNS secrets, cloud-init data, private keys, passwords, or provider
resource identifiers. Treat `Debug` output as untrusted unless its type
explicitly documents redaction.

## Timeouts

Set nonzero connect and total timeouts for every network client. Bound response
storage before sending. A timeout is an indeterminate outcome for a mutation:
the provider may have accepted the request even when the response was not
received. Reconcile provider state before repeating the operation.

## Retries

The SDK does not retry or sleep automatically. Retry only after classifying the
method, provider error, rate-limit metadata, and operation idempotency. Apply a
bounded attempt budget, caller-owned deadline, backoff, and cancellation path.
Do not retry create or action requests merely because transport delivery was
uncertain.

Action polling is different from retrying a mutation. Send a mutation once,
retain its validated action identifier, and poll the action endpoint through a
caller-owned `PollPolicy`. Reject progress regression and stop at explicit
timeout or cancellation limits.

## Live Smoke Tests

The repository smoke harness is read-only and ignored by default. Its secure
workflow builds from a clean commit before credentials exist, installs a
root-owned sealed bundle, and executes the authenticated binary through an
already-open descriptor without invoking Cargo. Follow
[Live Smoke Testing](LIVE_SMOKE_TESTING.md) exactly; do not simplify it by
passing a token through the shell environment.

Destructive live tests are not implemented. Any future mutation harness must
use a dedicated account or project, unique resource prefixes, explicit opt-in,
cost limits, and verified cleanup.

## Incident Response

On suspected exposure or incorrect mutation, stop automated callers, revoke
the affected token, inspect provider audit and resource state, contain costs,
and preserve redacted evidence. Report SDK vulnerabilities through
[SECURITY.md](../SECURITY.md), not a public issue.
