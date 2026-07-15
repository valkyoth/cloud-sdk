# Migrating To v0.27.0

The workspace facade moves to `0.27.0`. Code-bearing provider-neutral crates
move to `cloud-sdk-reqwest 0.18.0` and `cloud-sdk-testkit 0.16.0`; the Hetzner
provider moves to `0.21.0`.

## Required Request Inputs

Constructors no longer accept `Option<T>` for fields the Hetzner API requires.
Pass validated values directly and remove `Some(...)` plus result handling that
only checked `MissingRequiredField`.

```rust
# use cloud_sdk_hetzner::cloud::servers::{ServerCreateRequest, ServerName, ServerReference};
# let name = ServerName::new("web-1")?;
# let server_type = ServerReference::new("cx23")?;
# let image = ServerReference::new("ubuntu-24.04")?;
let request = ServerCreateRequest::new(name, server_type, image);
# Ok::<(), cloud_sdk_hetzner::cloud::servers::ServerRequestError>(())
```

The same migration applies to required fields for firewalls, load balancers,
networks, placement groups, floating and primary IPs, volumes, certificates,
SSH keys, DNS zones and RRSets, and Console Storage Box request bodies.
Optional builder fields and explicit set/reset enums are unchanged.

## Action Intent

DNS pointer actions now take their explicit set/reset intent directly. Empty
alias-IP replacement and bodyless construction of body-requiring server actions
return `EmptyAliasIps` and `ActionBodyRequired` instead of a generic missing
field error.

## Custom Credential Endpoints

Rename `HttpsEndpoint::new(value)` to
`HttpsEndpoint::new_custom(value)`. This is intentionally conspicuous because
the configured origin receives the supplied bearer token.

```rust,no_run
# #[cfg(feature = "blocking-rustls")]
# fn example() {
use cloud_sdk_reqwest::blocking::HttpsEndpoint;

// Trusted operator configuration only; never tenant-controlled input.
let endpoint = HttpsEndpoint::new_custom("https://api.hetzner.cloud/v1")?;
# Ok::<(), cloud_sdk_reqwest::blocking::EndpointError>(())
# }
```

Official provider endpoint constructors are planned for the high-level Hetzner
client. Until then, review custom origins before binding credentials.

## Errors

All public first-party error families now implement payload-free `Display` and
`core::error::Error`. Applications may use standard error propagation without
formatting provider payloads or credentials. Programmatic matching should use
variants; human diagnostics may use the static Display message.
