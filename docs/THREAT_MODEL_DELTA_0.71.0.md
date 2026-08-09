# v0.71.0 Threat Model Delta

Status: implementation stop reached; pentest required before tagging.

## New Boundary

v0.71 makes all active DNS operations discoverable through named official
client methods. DNS requests can contain zonefiles, record data, labels, and
TSIG credentials, so operation identity, endpoint trust, permit authority, and
complete-buffer cleanup remain security boundaries.

## Threats And Controls

### Classification Drift

- Methods are generated from the same 24 source-locked DNS associations used
  by request preparation and response decoding.
- Tests require exactly eight read-only, nine mutation, seven destructive, and
  four numbered-pagination operations.
- Generated output freshness runs in ordinary and release gates.

### Unauthorized DNS Mutation

- Read-only methods accept only operations classified with no permit.
- Mutation and destructive methods separate cleanup-owning preparation from
  execution and accept only a matching `AssociatedPermitAttempt`.
- The client creates no permit, retry, reconciliation, or idempotency policy.

### Credential Destination Confusion

- Named methods exist only for official `DnsService` clients.
- Official construction requires exact `https://api.hetzner.cloud:443/v1`
  identity before credentials or requests are used.
- Custom-trust clients receive no generated DNS execution methods.

### TSIG And Zonefile Exposure

- Sensitive request types expose no raw string accessor and redact `Debug`.
- Named preparation requires `PreparationStorageGuard`; complete target and
  body regions clear on failure and drop.
- Tests prove TSIG material appears only in the prepared wire body and not in
  operation or prepared-request diagnostics.

### Cancellation Residue

- Reads consume a complete workspace lease across every executor mode.
- Send-async and local-async mutation methods clear complete response storage
  during future construction. Dropping an unpolled future leaves the permit in
  conservative reconciliation state and performs no transport call.

### Misleading FIPS Claims

- The experimental FIPS feature and dependency graph are removed.
- CI rejects their reintroduction and requires the public deferment policy.
- No crate, application, deployment, or organization receives a FIPS claim.

## Unchanged Boundaries

Credential generation, entropy, storage, rotation, caller-source cleanup,
provider pricing, and live mutation testing remain caller or operator
responsibilities. Default builds perform no allocation or I/O.
