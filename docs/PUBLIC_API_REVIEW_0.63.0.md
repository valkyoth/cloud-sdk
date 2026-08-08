# v0.63.0 Public API Review

Status: implementation review complete; pentest required.

Scope: changes from signed v0.62.0 through v0.63.0.

## Added Provider API

- `CloudResource` and `CloudResourceKind` preserve the exact ordinary Cloud
  resource family after checked decoding.
- Dedicated `Firewall`, `FloatingIp`, `Image`, `Iso`, `LoadBalancer`,
  `LoadBalancerType`, `Network`, `PlacementGroup`, `PrimaryIp`, `Server`,
  `ServerType`, and `Volume` models expose positive IDs, optional names, and
  their complete retained field tree.
- `CloudObject`, `CloudValue`, and `CloudNumber` expose stable field lookup and
  iteration while preserving null, boolean, integer, finite fractional,
  string, list, and object distinctions.
- `HetznerSuccess::{Location, CloudResource, CloudResources}` and
  `CompositeResult::cloud_resource` preserve dedicated Cloud results.
- `Pricing::fields` exposes the complete pricing response while its existing
  currency, VAT, and count accessors remain available.
- `ResponseModelError::SchemaMismatch` reports malformed committed schema
  evidence without including provider payload data.

All new enums and response families remain non-exhaustive where downstream
matching could otherwise block additive provider evolution. Unknown enum
strings are retained as text in the field tree; source-known enum values are
evidence, not a closed runtime allowlist.

## Changed Decoding

Ordinary Cloud operations no longer return generic `Resource` or `Resources`
results. Single and list operations return their dedicated Cloud variants, and
Cloud create composites use `cloud_resource`. Locations now distinguish single
and paginated results. This is an intentional pre-1.0 development API change.

## Compatibility

No default feature, dependency, runtime, transport, executor, filesystem,
clock, TLS, or secret-store boundary changes. Models require the existing
optional `serde` and `alloc` boundary. No provider-neutral API changed.
