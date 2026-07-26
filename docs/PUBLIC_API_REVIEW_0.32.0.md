# v0.32.0 Public API Review

Date: 2026-07-26

Scope: provider and service identity contracts in `cloud-sdk`, provider-owned
Hetzner markers, prepared request bindings, checked response decoding, and
external provider compatibility.

## Decision

The v0.32 identity API is accepted for pentest with these boundaries:

- `ProviderId` and `ServiceId` are distinct validated static token types.
- Both domains are allocation-free, `no_std`, `Copy`, ordered, and hashable.
- Each ID is bounded to 63 bytes.
- Canonical syntax is lowercase ASCII letters and digits with single internal
  hyphens.
- Private fields prevent safe callers from constructing an unvalidated ID.
- Literal macros perform the same validation during constant evaluation.
- `ProviderMarker` is open for implementation by provider crates.
- `ServiceMarker` requires an associated `ProviderMarker`, making ownership
  explicit.
- `ProviderService::from_marker` derives both IDs from one service marker.
- Direct `ProviderService::new` accepts only already validated IDs.

## Provider Ownership

`cloud-sdk` contains no provider registry, provider enum, API-family enum, or
Hetzner variant. `cloud-sdk-hetzner` owns:

- `Hetzner`
- `CloudService`
- `DnsService`
- `SecurityService`
- `StorageService`

It also exports the corresponding canonical IDs for comparisons that do not
need marker types. Existing prepared operations bind Cloud and Console Storage
requests to the same official endpoint identities and logical API services as
before.

## Compatibility Evidence

- An integration test compiled as an external crate implements an independent
  provider and service without changing core source.
- Compile-fail doctests reject forged ID tuple construction.
- Compile-fail doctests reject invalid ID macro literals inside function
  bodies, where validation must never become a runtime panic path.
- Compile-fail doctests reject a service marker without an associated provider.
- Boundary tests cover empty, oversized, uppercase, Unicode, whitespace,
  underscore, leading/trailing separator, and repeated-separator inputs.
- Hetzner checked decoding rejects the wrong validated service ID.
- Default, all-feature, documentation, MSRV, and package checks are release
  requirements.

## Breaking Changes

The pre-1.0 `Provider` and `ApiFamily` enums are removed. `ProviderService`
accessors now return `provider_id()` and `service_id()`. Migration is explicit
in [`MIGRATION_0.32.0.md`](MIGRATION_0.32.0.md).

No compatibility alias is retained because aliases to closed enums would keep
the central-registration design alive and make later provider additions appear
supported through a misleading catch-all variant.

## Rejected Alternatives

- **A larger central enum:** still requires core releases for every provider
  and service.
- **Unvalidated `&str`:** admits case, Unicode normalization, delimiter, and
  unbounded-length ambiguity.
- **Owned `String`:** adds allocation and prevents an allocation-free default
  graph.
- **Integer registries:** require central allocation and make diagnostics and
  independent provider development less transparent.
- **A blanket string-only service marker:** cannot express which provider owns
  a service.

## Security Notes

Provider and service IDs are routing and policy metadata, not credentials.
They must not replace endpoint identity verification or authorization policy.
Static bounded storage avoids attacker-controlled allocation. Canonical syntax
makes equality byte-exact and independent of locale, Unicode normalization, or
case folding.
