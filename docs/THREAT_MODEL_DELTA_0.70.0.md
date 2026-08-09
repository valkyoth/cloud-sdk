# v0.70.0 Threat Model Delta

Status: implementation complete; pentest required.

## New Boundary

v0.70 makes the complete Hetzner Cloud client surface discoverable through
named methods. This reduces manual assembly but increases the importance of
preserving operation identity, endpoint trust, permit class, response policy,
and executor parity in generated code.

## Threats And Controls

### Classification Drift

- Methods are generated from the same 139 source-locked Cloud associations as
  request preparation and response decoding.
- The registry and generator tests require exact read, mutation, destructive,
  cost, and numbered-pagination counts.
- Stale generated code fails the ordinary and release gates.

### Mutation Or Billing Bypass

- Read-only methods accept only markers with `NoPermit` policy.
- Every state-changing method prepares into cleanup-owning guarded storage and
  executes only an `AssociatedPermitAttempt` with the same operation marker.
- Preparation cannot create authority, and the client cannot synthesize a
  permit, clock, cost approval, retry, or idempotency policy.

### Credential Destination Confusion

- Named methods exist only for `OfficialEndpointTrust` Cloud clients.
- Official construction validates exact HTTPS host, effective port, and `/v1`
  base path before returning a client.
- Custom clients remain type-distinct and receive no execution methods.

### Executor Or Policy Divergence

- Every operation row generates blocking, `Send` async, and local-async entry
  points from one macro definition.
- Representative read and permitted mutation scenarios traverse all modes.
- Each method sends one attempt through the existing transport and checked
  response policies; no fallback or implicit retry exists.

### Buffer Residue And Cancellation

- Reads consume a complete `ClientWorkspaceLease`, whose four regions are
  cleared on completion, error, cancellation, or an unpolled future drop.
- State-changing preparation borrows `PreparationStorageGuard`; the complete
  target and body regions are cleared on reuse and drop.
- State-changing Send-async and local-async entry points clear complete
  response-body and response-header storage synchronously before returning a
  future. Dropping that future without polling cannot retain an earlier
  response.
- Response and permit cleanup remain enforced by the existing core contracts.

## Unchanged Boundaries

Token generation, storage, rotation, and caller-source cleanup remain
application or transport responsibilities. Live mutation testing remains out
of scope. Default builds perform no allocation or I/O.
