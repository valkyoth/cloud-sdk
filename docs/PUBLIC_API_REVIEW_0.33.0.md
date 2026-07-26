# v0.33.0 Public API Review

Date: 2026-07-26

Scope: provider-neutral HTTP method construction, transport mapping, request
targets, and provider-owned operation safety metadata.

## Decision

The v0.33 method API is accepted for pentest with these boundaries:

- `Method` remains allocation-free, `no_std`, `Copy`, ordered, and hashable.
- GET, POST, PUT, DELETE, PATCH, HEAD, and OPTIONS have one canonical
  associated constant each.
- `Method::extension` accepts provider-owned static tokens up to 32 bytes.
- Extensions accept only uppercase ASCII HTTP token bytes and cannot alias a
  known method.
- Private storage prevents safe callers from forging a method.
- CONNECT and TRACE cannot be constructed.
- `RequestTarget` remains origin-form-only, so `OPTIONS *` cannot be
  constructed.
- Protocol upgrade, authority-form tunnelling, and asterisk-form requests are
  outside the current transport contract.
- Operation impact, request semantics, retry eligibility, and cost intent are
  independent metadata. They are never inferred from the HTTP method.

Provider method sets are finite source-locked API contracts, so static
extension storage is intentional. Runtime user-controlled method names are not
an SDK requirement and do not gain an allocation or lifetime surface.

## Adapter Contract

The blocking and async reqwest adapters convert validated method bytes with
`reqwest::Method::from_bytes`. Conversion failure is payload-free and fails
before network I/O. Exact loopback tests cover PATCH, HEAD, OPTIONS, and PURGE.

The adapter does not add method-specific redirects, retries, bodies, content
types, or response behavior. Existing redirect and retry denial remains
unchanged.

## Provider Migration

Hetzner prepared operations now declare one private `OperationClass`:

- read-only;
- idempotent mutation;
- non-idempotent mutation;
- idempotent destructive; or
- non-idempotent destructive.

That class selects impact, semantics, and retry eligibility independently from
the endpoint's wire method. Existing metadata remains unchanged. A source gate
rejects reintroduction of the removed `method_metadata` helper.

## Rejected Alternatives

- **Add more enum variants per release:** still forces a core release for each
  provider extension.
- **Accept arbitrary `&str`:** admits case aliases, controls, separators,
  Unicode ambiguity, and unbounded values.
- **Allocate method strings:** adds an unnecessary default-graph allocator for
  finite provider contracts.
- **Infer safety from GET/PUT/POST/DELETE:** HTTP methods do not prove provider
  idempotency, destructive effect, billing impact, or retry safety.
- **Admit CONNECT, TRACE, upgrade, or `OPTIONS *`:** the current request and
  response contracts do not model tunnelling, protocol switching, or
  non-origin targets safely.

## Compatibility

Existing `Method::Get`, `Method::Post`, `Method::Put`, and `Method::Delete`
expressions remain source compatible. Exhaustive matching on the former enum is
intentionally no longer possible. See
[`MIGRATION_0.33.0.md`](MIGRATION_0.33.0.md).
