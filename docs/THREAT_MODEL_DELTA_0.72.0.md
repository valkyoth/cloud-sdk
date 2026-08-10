# v0.72.0 Threat Model Delta

Status: implementation stop reached; pentest required before tagging.

## New Boundary

v0.72 exposes every active certificate and SSH-key operation through named
official Security client methods. Requests can contain uploaded certificate
private keys, public keys, labels, and destructive deletion intent.

## Threats And Controls

### Classification Drift

- Methods are generated from the same 14 source-locked Security associations
  used by request preparation and checked response decoding.
- Tests require exactly seven read-only, five mutation, two destructive, and
  four numbered-pagination operations.
- Generated output freshness runs in ordinary and release gates.

### Unauthorized Key Or Certificate Mutation

- Read-only methods accept only no-permit operations.
- Mutation and destructive methods separate cleanup-owning preparation from
  execution and accept only the matching `AssociatedPermitAttempt`.
- The client creates no permit, rollback, retry, rotation, or reconciliation
  authority.

### Private-Key Disclosure

- `PrivateKeyPem` exposes no raw string accessor and redacts `Debug`.
- Named create preparation requires `PreparationStorageGuard`; complete target
  and body regions clear on failure and drop.
- Tests prove escaped key material is confined to the prepared body and absent
  from operation and prepared-request diagnostics.
- Caller-owned source material remains an explicit caller cleanup boundary.

### Unsafe Rotation Ordering

- SSH-key replacement and old-key deletion remain separate source operations
  with separate mutation and destructive permits.
- The SDK does not delete an old key automatically after create, infer
  deployment state, or retry either step.

### Credential Destination And Cancellation

- Named methods exist only for official `SecurityService` clients bound to
  exact `https://api.hetzner.cloud:443/v1` identity.
- Read workspaces and mutation response buffers clear across blocking,
  Send-async, local-async, and unpolled cancellation paths.

## Unchanged Boundaries

Credential generation, entropy, token rotation, caller-source cleanup,
certificate/key cryptographic validation, deployment verification, provider
pricing, and live mutation testing remain caller or operator responsibilities.
