# v0.80.0 Threat Model Delta

Status: implementation stop reached; pentest required.

## New Boundary

v0.80 prepares Robot traffic-warning and separate-MAC mutations and admits
bounded IP assignment, network, lock, threshold, and MAC state. Addresses,
server associations, thresholds, and MAC values may be operationally
sensitive. MAC generation and deletion can change network reachability.

## Threats And Controls

### Wrong Target Or Mutation

- Paths and the optional list filter accept only canonical protected IP values.
- Named request types fix method, route, form, operation ID, and response
  policy; no arbitrary field or endpoint is public.
- Traffic updates cannot be empty and serialize only explicitly selected
  fields. Their bodies are sensitive and require digest plan fingerprints.
- MAC generation is non-idempotent and MAC deletion is destructive; both deny
  automatic retry. Traffic update requires an explicit retry policy.
- Request-bound direct/shared permits preserve the exact mutation through
  blocking, Send-async, and local-async execution.

### Hostile Or Contradictory Responses

- Every operation requires checked `200` JSON under the source-locked body and
  content-type bounds.
- Exact envelopes reject unknown and duplicate fields; lists are bounded to
  4,096 entries and duplicate-free by address, including a valid empty state.
- List filters and exact resource identities are verified after decoding.
- Detail network family, prefix, gateway, and broadcast values must agree.
- Update acknowledgement must match every requested field. MAC get/generate
  requires a canonical value, while delete requires null.

### Data Lifetime

- IP and MAC identities use stable protected storage, redacted diagnostics,
  and closure-scoped inspection.
- Request preparation pre-clears caller path/body storage and clears both on
  reachable validation or capacity failure.
- Checked decode consumes the cleanup-owning response guard. Unpolled permit
  attempts clear request buffers through the core attempt lifecycle.

## Residual Boundaries

Caller-owned copies, transport/TLS/OS buffers, provider-side state, allocator
abort behavior, credential custody, and post-delivery reconciliation remain
outside this module. v0.80 performs no live mutation and adds no high-level
Robot client or automatic reconciliation.
