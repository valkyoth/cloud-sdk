# Migrating To v0.34

v0.34 replaces one static expected endpoint with provider-owned endpoint
policies and makes custom bearer-token destinations explicitly acknowledged.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.34.0"
cloud-sdk-hetzner = "0.27.0"
cloud-sdk-reqwest = { version = "0.22.0", features = ["blocking-rustls"] }
```

Related boundary releases are:

- `cloud-sdk-sanitization 0.15.2` as a dependency-only patch;
- `cloud-sdk-testkit 0.19.0` with policy-aware prepared records.

## ProviderService

`ProviderService::new` and `from_marker` now take an `EndpointPolicy`:

```rust
use cloud_sdk::operation::ProviderService;
use cloud_sdk::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme,
};
# use cloud_sdk::{ProviderId, ProviderMarker, ServiceId, ServiceMarker, provider_id, service_id};
# enum Provider {}
# impl ProviderMarker for Provider { const ID: ProviderId = provider_id!("example"); }
# enum Service {}
# impl ServiceMarker for Service {
#     type Provider = Provider;
#     const ID: ServiceId = service_id!("compute");
# }

let identity = EndpointIdentity::new(
    EndpointScheme::Https,
    "api.example.invalid",
    443,
    "/v1",
)?;
let service =
    ProviderService::from_marker::<Service>(EndpointPolicy::fixed(identity));
assert_eq!(service.endpoint_policy().verify(identity), Ok(()));
# Ok::<(), Box<dyn core::error::Error>>(())
```

`ProviderService::endpoint()` is removed. Use `endpoint_policy()` and verify
the candidate identity. `ProviderService` and `PreparedRequestRecord` now
retain the endpoint-policy lifetime instead of requiring `'static`.

## Reqwest Endpoint Construction

Official or regional destinations should use a provider-owned policy:

```rust,no_run
# #[cfg(feature = "blocking-rustls")]
# fn example(
#     policy: cloud_sdk::transport::EndpointPolicy<'_>,
# ) -> Result<(), cloud_sdk_reqwest::blocking::EndpointError> {
use cloud_sdk_reqwest::blocking::HttpsEndpoint;

let _endpoint =
    HttpsEndpoint::new_with_policy("https://api.example.invalid/v1", policy)?;
# Ok(())
# }
# fn main() {}
```

Custom destinations require an explicit acknowledgement:

```rust,no_run
# #[cfg(feature = "blocking-rustls")]
# fn example() -> Result<(), cloud_sdk_reqwest::blocking::EndpointError> {
use cloud_sdk_reqwest::blocking::{
    CustomEndpointAcknowledgement, HttpsEndpoint,
};

let acknowledgement =
    CustomEndpointAcknowledgement::trusted_operator_configuration();
let _endpoint = HttpsEndpoint::new_custom(
    "https://proxy.operator.example/v1",
    acknowledgement,
)?;
# Ok(())
# }
# fn main() {}
```

Never create that acknowledgement at a tenant, request, webhook, or other
attacker-controlled boundary.

## Hetzner Policy

`cloud_sdk_hetzner::official_endpoint_policy(ApiBaseUrl)` returns the exact
fixed policy used by prepared Cloud or Console Storage operations.
`verify_official_endpoint` remains available for an already constructed bound
transport. `verify_any_official_endpoint` admits the finite two-endpoint set
only for provider-wide diagnostics.

## Canonical Authority Input

DNS input is lowercase ASCII without a trailing dot. Internationalized names
must already be lowercase A-label form. IPv6 literals are bracketed, compared
by address bits, and cannot carry zone identifiers. Userinfo, Unicode hosts,
percent-encoded hosts, unbracketed IPv6, ambiguous IPv4, uppercase DNS, and
non-canonical port forms fail before URL normalization.

`cloud-sdk-reqwest` bounds raw endpoint input with
`MAX_CONFIGURED_ENDPOINT_BYTES` before invoking the allocating URL parser.
Configured base paths must use printable canonical ASCII, and their parsed
form must match the exact input. Backslashes, controls, whitespace, non-ASCII
bytes, percent escapes, repeated slashes, and dot segments are rejected.

This is an application-layer credential destination policy. Core performs no
DNS resolution and no resolved-address or network-egress filtering. Apply
those optional controls in the transport, resolver, firewall, sandbox, or
deployment environment.
