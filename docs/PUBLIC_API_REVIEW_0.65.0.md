# v0.65.0 Public API Review

Status: implementation complete; incremental pentest required.

Scope: cumulative public changes from signed and published v0.60.0 through
v0.65.0, with the v0.65 delta focused on Hetzner DNS responses.

## Added Provider API

- `DnsResource` and `DnsResourceKind` distinguish zones from RRSets without
  reducing either response to a generic identifier.
- `Zone` exposes every current source field through typed accessors, including
  mode, status, canonical creation time, TTL, record count, labels, protection,
  registrar, delegation state, and primary nameservers.
- `PrimaryNameserver` exposes validated address/port and response TSIG algorithm
  while restricting TSIG key access to `try_with_tsig_key`.
- `PrimaryNameserver`, `Zone`, `DnsResource`, and the checked success wrappers
  omit ordinary equality because TSIG-bearing state can flow through them.
- `MAX_ZONE_RECORD_COUNT` publishes the conservative checked-response ceiling
  so callers can reason about the accepted operational envelope.
- `DnsRrset` exposes ID, owner name, nullable TTL, labels, protection, records,
  and owning zone. `DnsRrsetType` retains additive future uppercase RR types and
  classifies source-known values through `known`.
- `CompositeResult::dns_resource` returns the dedicated object from zone and
  RRSet create responses.
- `HetznerSuccess::DnsResource` and `DnsResources` represent singleton and
  list/page DNS responses.

## Changed Provider API

- Zone and RRSet singleton/list responses now decode to dedicated DNS variants
  instead of `HetznerSuccess::Resource` or `Resources`.
- Zone and RRSet create composites expose their resource through
  `CompositeResult::dns_resource`; `resource` remains for generic families.
- DNS lists and zonefile exports receive incremental JSON admission before the
  protected duplicate-rejecting model parser.

## Compatibility

These are intentional pre-1.0 enum-pattern changes behind the existing
`cloud-sdk-hetzner/serde` feature. Default features and the transport-free
`no_std` graph do not change. No provider-neutral transport, runtime, TLS,
filesystem, clock, retry, or credential contract is added.
