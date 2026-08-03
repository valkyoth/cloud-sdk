# v0.50.0 Public API Review

Date: 2026-08-03

Scope: neutral compile-time operation IDs and exhaustive Hetzner operation
associations.

## Added Neutral API

`OperationId::new` is now a `const fn`. The new `operation_id!` macro performs
the same bounded lowercase ASCII, digit, and underscore validation in a const
context. It does not expose the private representation or admit dynamic,
unbounded identifiers.

## Added Provider API

`cloud_sdk_hetzner::association` exports:

- 208 sealed operation markers under `association::operations`;
- sealed `HetznerOperation` and alias `OperationAssociation` traits;
- `EndpointFor<O, E>`, `QueryFor<O, Q>`, and `BodyFor<O, B>`;
- `AssociatedOperation<O, E, Q, B>`, cleanup-owning
  `prepare_typed_guarded`, and typed `Prepared<O>`;
- inspectable `OperationDescriptor` and non-exhaustive policy enums; and
- zero-sized public policy markers used by associated types.

The trait is sealed so downstream code can consume and constrain operation
associations but cannot forge provider policy. Policy enums are
`#[non_exhaustive]`; downstream exhaustive matches require a wildcard so new
policy variants remain additive.

## Accepted Design

The association layer wraps existing endpoint, query, body, preparation,
authentication, response, and operation-metadata implementations. It does not
fork wire encoding or create another network client. Concrete component traits
are public only to satisfy public generic bounds and are hidden and sealed.

`Prepared<O>` exposes explicit `as_untyped` and `into_untyped` routes because
provider-neutral transports consume the existing prepared contract. Typed
response-family associations are present now; high-level typed decode and
client ergonomics remain later roadmap work.

## Rejected Designs

- A public operation trait implemented by downstream crates was rejected
  because it could forge authentication, endpoint, response, or permit policy.
- Hand-maintaining 208 implementations was rejected in favor of deterministic
  generation from reviewed source locks.
- Encoding policy directly in endpoint-name heuristics at runtime was rejected;
  preparation validates generated metadata against existing provider policy.
- Returning an association mismatch after serialization was rejected. Storage
  is cleared first, then exact write-free validation returns one immutable
  policy token consumed directly by request construction.
- Inferring security classifications from names, tags, or HTTP methods was
  rejected in favor of the strict reviewed `OPERATION_ASSOCIATIONS.tsv`.
- Removing existing `PrepareOperation` APIs was rejected as unnecessary churn
  during the pre-1.0 migration window.

## Compatibility And Semver

All additions are feature-independent and preserve default no_std,
allocation-free provider behavior. Existing public signatures remain intact.
The neutral const qualification is source-compatible. New policy enums are
forward-compatible by construction, and associated types may only be added to
the sealed provider trait before 1.0 under the documented version policy.
