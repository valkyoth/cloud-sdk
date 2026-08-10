# v0.73.0 Threat Model Delta

Status: release candidate; pentest passed with no findings.

## New Boundary

v0.73 exposes every active Console Storage Box operation through named
official Storage client methods. Requests can create billable boxes, mutate
access controls, reset credentials, delete data, roll back snapshots, and
change product types.

## Threats And Controls

### Classification Drift

- Methods are generated from the same 31 source-locked Storage associations
  used by request preparation and checked response decoding.
- Tests require exactly 12 read-only, nine mutation, eight destructive, two
  cost-authorized, and four numbered-pagination operations.
- Generated output freshness runs in ordinary and release gates.

### Unauthorized Cost, Mutation, Or Destruction

- Read-only methods accept only no-permit operations.
- State-changing methods separate cleanup-owning preparation from execution
  and accept only their exact mutation, destructive, or cost permit attempt.
- Creating a box and changing its product type require cost authorization.
- Deletion, rollback, protection reduction, snapshot-plan disablement, and
  credential resets require destructive authorization.
- The client creates no permit, retry, rollback, or reconciliation authority.

### Password Disclosure

- `StorageBoxPassword` has no raw string accessor and redacts `Debug`.
- Create and password-reset bodies carry sensitive-body metadata and require
  digest plan fingerprints; exact canonical fingerprints fail closed.
- Guarded request storage, canonical scratch, response bodies, and response
  headers clear on return, error, and unpolled async cancellation.
- Caller-owned source material remains the caller's cleanup responsibility.

### Large Or Malformed Responses

- Checked Storage models retain source field, collection, pagination, text,
  decimal, and aggregate bounds before exposing data.
- Large list responses use the existing bounded incremental decoder and one
  caller-selected response capacity.
- Runtime evidence crosses 32 KiB with 32 complete nested Storage Box models
  through blocking, `Send` async, and local-async execution.

### Credential Destination

- Named methods exist only for official `StorageService` clients bound to
  exact `https://api.hetzner.com:443/v1` identity.
- Explicitly acknowledged custom clients remain non-executable by generated
  methods.
- The live smoke is read-only, opt-in, and uses list operations only.

## Unchanged Boundaries

Password generation, entropy, credential rotation, caller-source cleanup,
backup validation, snapshot consistency, pricing review, data migration, and
live mutation testing remain caller or operator responsibilities.
