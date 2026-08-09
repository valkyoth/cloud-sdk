# v0.71.0 Public API Review

Status: implementation stop reached; pentest required before tagging.

Scope: changes from signed v0.70.0 through the v0.71.0 candidate.

## Provider-Neutral Workspace API

v0.71 adds no provider-neutral runtime API. The `cloud-sdk` source version
advances with the tag and remains compatible with its documented Rust and
`no_std` boundaries.

## Hetzner DNS Client API

With `serde`, `cloud_sdk_hetzner::client` adds:

- one named blocking, `Send` async, and local-async method for each of the eight
  active read-only DNS operations;
- one named cleanup-owning preparation method and three permit-authorized
  execution methods for each of the nine mutation and seven destructive DNS
  operations;
- `DnsReadResult<E>` for complete checked read execution;
- `DnsClientMethodDescriptor` and `DNS_CLIENT_METHODS` for exhaustive policy
  inspection.

The exact 24-operation surface is generated from the reviewed operation
association manifest. Named methods exist only on
`HetznerClient<T, DnsService, OfficialEndpointTrust>` and preserve numbered
pagination, action, response, retry, and permit classifications.

## Retired FIPS API

`cloud-sdk-reqwest` removes the experimental `blocking-rustls-fips` feature,
`FipsTlsPolicy`, its builder controls, FIPS-specific status and construction
errors, and the `aws-lc-fips-sys` dependency graph. Ordinary blocking,
deterministic-root, and async rustls transports remain available.

FIPS is excluded from the cloud-sdk 1.0 scope. A future optional integration is
deferred until Brynja meets the admission conditions in
[`FIPS_DEFERMENT.md`](FIPS_DEFERMENT.md).

## Compatibility

Existing request types, generic associated execution, checked decoding, Cloud
client methods, and ordinary rustls adapters remain available. No default
feature changes, and the default workspace remains transport-free and
`no_std`.
