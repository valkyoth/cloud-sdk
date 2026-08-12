# v0.81.0 Threat Model Delta

Status: implementation stop; pentest required.

## New Boundary

v0.81 prepares Robot subnet traffic and MAC mutations and admits provider
subnet topology, assignment metadata, and selectable MAC mappings. Addresses,
server associations, traffic thresholds, and MAC values may be operationally
sensitive.

## Threats And Controls

### Wrong Target Or Mutation

- Paths accept only canonical protected subnet route identities.
- Request types fix method, route, operation ID, form shape, and response type.
- PUT requires an explicit canonical selected MAC; no default is inferred.
- DELETE authority consumes same-resource checked subnet and MAC snapshots,
  a fixed 30-second observation window, and a same-resource external mutation
  lease; address-only construction is unavailable.
- Server, MAC, both observation timestamps, evidence expiry, lock generation,
  and lease expiry are bound into digest-only authorization evidence. Permit
  validity cannot exceed either evidence lifetime. Creation, every
  start/recheck path, and immediate pre-dispatch validation reject stale
  evidence using one clock sample; async checks occur on first poll.
- Traffic update is idempotent with explicit-policy retries. MAC PUT is
  non-idempotent and MAC DELETE is destructive; both deny automatic retry.
- Request-bound plan, fingerprint, permit, and checked-response wrappers retain
  the exact request through blocking, Send-async, and local-async execution.

### Hostile Or Contradictory Responses

- Exact envelopes reject duplicate, unknown, missing, and mistyped fields.
- Lists and selectable MAC maps are bounded and duplicate-free where identity
  semantics require it.
- Prefix limits and gateway family/membership are validated before admission.
- Response subnet identity and optional list filter must match the request.
- The current MAC must be advertised; PUT acknowledgement must equal the
  selected MAC; traffic acknowledgement must include each requested value.
- DELETE acknowledgement must equal the default MAC mapped to the checked
  assigned server, and the returned map must preserve that association.
- Provider errors are decoded through the exact request type; documented `404`
  and `500` codes cannot cross operation boundaries.
- Nullable `server_ip`, integer/string mask differences, and host-bits-set
  route identities are admitted only as explicit source-locked exceptions.

### Data Lifetime

- Addresses and MACs use stable protected ownership and redacted diagnostics.
- Preparation validates every fallible cross-policy invariant before writing,
  pre-clears caller storage, and clears target/body on write failure.
- Traffic and MAC assignment forms are sensitive and require digest plans.
- Authorization-evidence scratch and digest output are cleanup-guarded before
  evidence encoding, algorithm selection, or digest callbacks can run, so
  errors and unwind-enabled panics clear both complete caller buffers.
- Traffic policy/update aggregates redact diagnostics, cannot be copied as
  aggregates, and clear their owned scalar fields on drop.
- Checked decode consumes the cleanup-owning response guard.
- Unpolled permit execution clears caller response buffers and preserves only
  the core reconciliation state required after uncertain delivery.

## Residual Boundaries

The official route identity is not necessarily the mathematical network base.
Callers must use the derived network/broadcast accessors when range boundaries
matter and must not reinterpret the route identity. No live mutation or
automatic reconciliation is introduced.

Robot exposes no ETag, generation, or conditional subnet-MAC mutation. The SDK
therefore requires caller-provided external-lock evidence but cannot operate a
distributed lock service. Callers must obtain each lease from a system that
serializes every mutation for the same subnet. Scalar values explicitly read
from traffic accessors become caller-owned copies outside SDK cleanup control.
