# v0.69.0 Threat Model Delta

Status: implementation complete; pentest required.

## New Boundary

v0.69 introduces the first provider facade that owns an endpoint-bound
authenticated transport and can execute associated read-only operations. It
therefore concentrates endpoint, credential scope, service selection, request
storage, response policy, and checked decoding in one public path.

## Threats And Controls

### Credential Exfiltration Through Endpoint Confusion

- Official constructors verify exact normalized scheme, host, effective port,
  and `/v1` base path before returning a client.
- Cloud, DNS, and security use `api.hetzner.cloud`; Console Storage uses
  `api.hetzner.com`. Cross-family construction fails closed.
- Custom constructors require explicit operator acknowledgement and HTTPS.
- Custom clients receive a separate type marker and expose no execution path,
  so an official operation cannot silently send credentials to a custom host.

### Cross-Service Operation Confusion

- Each client carries one `ServiceMarker` in its type.
- Each executable associated operation retains its operation marker's service.
- The operation-to-service trait is sealed against downstream implementations.
- Client execution requires exact service-type equality; Cloud, DNS, security,
  and Storage operations cannot be interchanged.

### Mutation Or Billing Bypass

- Only `ReadOnlyOperation` associations implement the client operation bridge.
- Mutation, destructive, and cost-bearing operations retain the existing
  plan-confirm permit boundary and cannot call direct client execution.
- The client performs exactly one transport attempt and owns no retry policy.

### Buffer Residue And Capacity Confusion

- Named profiles validate target, request body, response body, and response
  headers as one complete storage policy.
- All four borrowed regions are cleared before profile rejection.
- Every execution clears all four regions before preparation and on lease
  drop, including cancellation and error paths.
- Optional owned storage allocates exact bounded profiles fallibly and clears
  all four complete allocations on drop.

### Unbounded Concurrency Or Hidden Runtime Policy

- The client is borrowed through `&self`; concurrency requires a caller-owned
  finite workspace pool and a transport that satisfies the selected mode.
- Pool exhaustion is immediate. There is no queue, task spawn, timer, clock,
  executor, backoff, or implicit retry.

## Unchanged Boundaries

Token generation, storage, rotation, and caller-owned source-buffer cleanup
remain transport/application responsibilities. TLS and network I/O remain in
optional transport crates. Default provider builds remain transport-free,
allocation-free, and `no_std`.
