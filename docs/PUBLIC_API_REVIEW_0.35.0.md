# v0.35.0 Public API Review

Date: 2026-07-27

Scope: provider-neutral request path/query types, exact target assembly,
Hetzner path admission, reqwest composition, testkit matching, and fuzzing.

## Decision

The v0.35 request-target API is ready for independent security review with
these boundaries:

- `RequestPath`, `CanonicalQuery`, and `FormQuery` are borrowed,
  allocation-free, `no_std`, bounded values.
- `RequestQuery` makes absent, canonical-present, and form-present states
  explicit; present queries may be empty.
- `RequestTarget::assemble` preflights complete capacity and length before
  modifying caller storage.
- successful assembly initializes only `output[..target.len()]`; the unused
  tail remains caller-owned, untouched, and outside every target view.
- `RequestTarget::new` accepts only the canonical query dialect.
- `RequestTarget::path`, `query`, and `query_bytes` expose the exact admitted
  output without decoding, normalization, reordering, or re-encoding.
- all request-target and component `Debug` output remains redacted.

## Canonical Grammar

Paths use origin form, begin with exactly one slash, and reject adjacent
separators, dot segments, fragments, backslashes, raw
whitespace/control/non-ASCII bytes, malformed or lowercase percent triplets,
encoded structural separators, encoded controls, and encoded unreserved bytes.
One trailing slash remains an exact, distinct path and is preserved.

Canonical structured queries use nonempty keys, `&` pair separators, at most
one raw `=` per pair, uppercase percent hex, and `%20` for spaces. Exact pair
iteration preserves order, duplicate keys, and `key` versus `key=`. Raw `+`
is rejected. `FormQuery` admits `+` only through an explicit separate type.

Encoded non-ASCII octets remain uppercase percent triplets and are never
decoded by this layer. Encoded `%26` and `%3D` are query component data, while
raw `&` and `=` are syntax. Encoded fragment and control bytes fail closed.

## Adapter Contract

Reqwest no longer owns a second target validator. It appends only an already
validated `RequestTarget`, requires URL parsing to preserve every exact byte,
and verifies that scheme, host, effective port, and credentials remain bound
to the configured endpoint.

Testkit compares the complete target value, including absent versus
present-empty state and canonical versus form dialect. Hetzner provider paths
must satisfy both the provider-specific 1,024-byte bound and the canonical
core grammar. Prepared query bytes are directly observable through
`RequestTarget::query_bytes`.

## Scratch Buffer Boundary

The returned target borrows the caller's output storage and exposes only the
initialized prefix. Callers must use `target.as_str()`, `target.len()`, or the
component views and must never log, hash, sign, or transmit the entire backing
slice. The untouched tail may contain data from a prior request. Callers that
reuse storage across sensitive boundaries must apply their cleanup policy to
the complete buffer.

## Future Signing Boundary

v0.35 supplies exact final query bytes but does not add signing or request
fingerprints. Later domains must consume `query_bytes` directly and must not
decode, sort, normalize, or re-encode it. Their versioned domain separation,
body/header coverage, and collision-resistant digest policy remain assigned
to later roadmap releases.

## Compatibility

This is a pre-1.0 breaking release. `RequestTargetError` now contains precise
path/query errors, reqwest removes `EndpointError::InvalidTargetEncoding`, and
Hetzner adds `EndpointPathError::NonCanonical`. See
[`MIGRATION_0.35.0.md`](MIGRATION_0.35.0.md).
