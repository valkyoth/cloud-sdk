# v0.68.0 Threat Model Delta

Status: implementation complete; pentest required.

## Assurance Gap Closed

Before v0.68, complete operation evidence was split between request adapters,
response locks, provider-authentication evidence, and generated marker rows.
Each boundary was checked, but reviewers had no single exact artifact proving
their combined association for all 208 active operations.

## Controls

- A canonical 28-column manifest binds each operation to its exact method,
  path, service, endpoint, authentication, query, body, media, status, success
  and error policy, byte caps, pagination, quota, retry, streaming, permit, and
  response-identity policy.
- Generation requires exact active-operation equality across fingerprints,
  associations, request-body locks, response locks, and authentication locks.
- An independent gate compares the committed manifest byte-for-byte with its
  regenerated form, compares operation sets with the Markdown API matrix, and
  compares every column with evidence emitted by compiled Rust descriptors and
  operation-associated marker types.
- Every prepared path must be a valid `RequestPath` and match the source-locked
  descriptor template. Raw or encoded query/fragment delimiters are rejected.
  Mismatch clears complete caller-owned request storage and fails closed.
- A separate response-identity lock distinguishes no identity, exact-resource,
  and parent-resource checks instead of flattening every row to one label.
- The Rust AST checker proves every active operation has an endpoint adapter,
  every required JSON body has a body adapter, and every endpoint declares an
  explicit response-identity policy.
- Twelve reviewed body-enum variants for bodyless server actions must remain
  the exact set admitted by AST but forbidden by typed body policy. Any change
  fails CI instead of silently broadening body support.
- All 13 source-locked deprecated operations must remain absent from marker,
  response, endpoint, and body registries.
- Compile-fail documentation proves query, body, checked-response, and permit
  associations cannot be cross-wired through safe Rust.

## Unchanged Boundaries

No request is sent, no provider response is parsed, and no credential is read
by the new tooling. The preparation path performs bounded path validation and
segment comparison without allocation. Transport, TLS, filesystem, clock,
secret, and dependency boundaries are unchanged.
