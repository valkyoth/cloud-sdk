# v0.86.0 Threat Model Delta

Status: implementation stop; pentest required.

## New Boundary

v0.86 can prepare Robot reverse-DNS mutations. A wrong PTR can disrupt reverse
identity checks, mail delivery reputation, access policy, auditing, and
operator attribution. Deletion removes an existing mapping. Provider responses
and caller-supplied names are untrusted.

## Threats And Controls

### Wrong Address Or PTR

- Paths accept only canonical protected IPv4/IPv6 values; free path fragments
  and noncanonical address spellings are unrepresentable.
- PTR values are bounded canonical lowercase DNS names with strict label and
  total-length checks.
- Typed checked responses retain the exact request. Set and update must return
  the requested address and PTR; list/get identities cannot cross requests.

### Duplicate Or Ambiguous Mutation

- Set, update, and delete are non-idempotent and never automatically retryable.
- Every mutation consumes short-lived request-bound authorization; delete uses
  a distinct destructive permit.
- The SDK performs no implicit fallback from set to update, no reconciliation,
  and no provider-state guess after uncertain delivery.

### Hostile Provider Data

- Item responses are capped at 16 KiB and list responses at 2 MiB. Lists are
  capped at 4,096 entries.
- Unknown, duplicate, missing, malformed, noncanonical, oversized, and
  identity-substituted values fail closed. Duplicate IP identities in a list
  are rejected.
- Provider error codes are narrowed by exact operation and status.
- Delete admits only an empty `200` response; alternate status/body shapes do
  not silently succeed.

### Credential And Endpoint Scope

- Requests retain the official Robot endpoint and Basic authentication scope.
- This milestone does not accept custom Robot endpoints or credentials from
  response data. Credential custody, rotation, transport copies, and caller
  cleanup remain existing boundaries.

## Residual Boundaries

Provider acceptance does not prove forward/reverse consistency, public DNS
propagation, domain control, mail acceptance, or later state persistence.
Callers must reconcile state after uncertain mutation delivery. Endpoint-bound
authenticated transport, authorization policy, credential custody,
allocator/process abort, and live mutation testing remain outside this
milestone.
