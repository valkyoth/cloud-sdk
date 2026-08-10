# v0.74.0 Threat Model Delta

Status: implementation stop reached; pentest required before tagging.

## New Boundary

v0.74 introduces a complete repository-only source lock for the official
Hetzner Robot document. The lock influences future endpoint implementation but
is not compiled into a crate and cannot execute provider operations.

## Threats And Controls

### Incomplete Or Reordered Inventory

- The official document digest and all 105 HTTP operation headings are checked
  in exact source order.
- Operation IDs and method/path pairs must be unique and canonical.
- Exact group counts require 89 active and 16 deprecated headings.
- Every active operation has one reviewed implementation milestone.

### Deprecated Endpoint Revival

- Every `/storagebox` heading must be deprecated, assigned to the excluded
  group, and carry an upstream `@deprecated` marker.
- No active operation may use the legacy Storage Box prefix.
- The documented replacement is the implemented Console Storage Box API.
- Deprecated server-IP aliases and deprecated fields are not operation rows.

### Credential Lockout

- The lock preserves Basic authentication rejection as HTTP 401, three failed
  attempts, a ten-minute source-IP lockout, and no automatic retry.
- Fetch and fixture checks never send credentials or call the Robot API origin.
- Runtime credential-attempt generations remain deferred to v0.76.

### Untrusted Upstream Bytes

- Fetching is HTTPS-only, redirect-rejecting, timeout-bounded, and capped at
  8 MiB.
- Fetched bytes are decoded and compared but never compiled, imported,
  executed, or copied into a publishable package.
- Any digest or heading change stops the release for manual review.

### Protocol Ambiguity

- The lock binds form POSTs, JSON responses, success statuses, invalid-input
  fields, quota fields, maintenance, lockout, and possible empty success bodies.
- Optional YAML responses are explicitly outside the SDK plan.
- Concrete codecs and typed protocol errors remain separate later milestones.

## Unchanged Boundaries

No Robot credentials, network execution, request serialization, response
decoding, retry behavior, cost permit, or live smoke behavior is added.
