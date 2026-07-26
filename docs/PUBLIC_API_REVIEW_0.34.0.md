# v0.34.0 Public API Review

Date: 2026-07-26

Scope: endpoint identity canonicalization, provider-owned trust policy,
prepared execution, Hetzner official policies, reqwest endpoint construction,
and testkit records.

## Decision

The v0.34 endpoint API is accepted after exact-commit pentest and final retest
with these boundaries:

- `EndpointPolicy` is allocation-free, `no_std`, borrowed, and non-static.
- Trust provenance is one fixed identity, a bounded finite official set, one
  provider-derived regional identity, or one acknowledged custom identity.
- Official sets are nonempty and bounded to 32 identities.
- Region IDs are bounded lowercase ASCII.
- Custom trust requires an opaque
  `CustomEndpointAcknowledgement::trusted_operator_configuration()` value.
- `ProviderService` owns the provider/service IDs and endpoint policy.
- Prepared blocking and async execution verify policy before transport I/O.
- Hetzner prepared operations bind the exact selected Cloud or Storage policy.

## Authority Contract

An endpoint identity includes scheme, canonical host, effective port, and
normalized base path. DNS input accepts lowercase ASCII labels and lowercase
A-label IDNA form only. IPv4 accepts four canonical decimal octets. IPv6 must
be bracketed; comparison uses parsed address bits. IPv6 zones, trailing DNS
dots, userinfo, percent-encoded hosts, Unicode hosts, uppercase DNS, malformed
labels, unbracketed IPv6, and ambiguous ports fail closed.

The reqwest adapter validates the raw authority before `url` parsing. This
prevents WHATWG URL normalization from silently converting Unicode, percent
escapes, case, trailing dots, unusual IPv4, or IPv6 text before trust policy is
applied. It also bounds the complete raw endpoint before allocation, validates
the raw base path as printable canonical ASCII, and requires the parsed path
to equal those exact configured bytes. Backslashes, controls, whitespace,
non-ASCII bytes, percent escapes, repeated slashes, and dot segments fail
before URL normalization. Redirects remain disabled and response origins
remain checked.

Hetzner prepared services obtain their fixed policies from
`official_endpoint_policy`; the public verifier, finite diagnostic set, and
prepared execution therefore share one source of official endpoint identity.

## Open Provider Model

Policy constructors remain public because independent provider crates must be
implementable without registration in `cloud-sdk`. The type system therefore
makes custom acknowledgement conspicuous but cannot prove that application
configuration is operationally trusted. Provider crates own official and
regional policy construction; applications own where custom acknowledgement
is permitted.

## Egress Boundary

Endpoint policy does not resolve DNS and does not classify resolved addresses.
Optional resolver pinning, private/link-local address denial, proxy controls,
firewall rules, and egress hooks belong to the transport or deployment
environment. Keeping them outside core preserves `no_std`, avoids stale DNS
policy, and prevents endpoint identity from claiming network-path guarantees
it cannot enforce.

## Compatibility

This is a pre-1.0 breaking release. `ProviderService::endpoint()` is replaced
by `endpoint_policy()`. Constructors accept `EndpointPolicy`, and
`HttpsEndpoint::new_custom` requires a second acknowledgement argument. See
[`MIGRATION_0.34.0.md`](MIGRATION_0.34.0.md).
