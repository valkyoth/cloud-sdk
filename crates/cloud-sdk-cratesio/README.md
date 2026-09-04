<p align="center">
  <b>Security-first, no_std crates.io API provider for cloud-sdk.</b><br>
  Provider-owned identities and bounded API domains on a transport-neutral foundation.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk-cratesio">Crates.io</a>
  |
  <a href="https://docs.rs/cloud-sdk-cratesio">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk">cloud-sdk</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/cratesio-commit-plan.md">Implementation Plan</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/SECURITY.md">Security</a>
</div>

# cloud-sdk-cratesio

`cloud-sdk-cratesio` is the crates.io provider crate for the main
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) project. It owns crates.io
provider identities, API models, request preparation, checked response
decoding, authentication rules, and high-level workflows while reusing the
provider-neutral execution contracts from `cloud-sdk`.

The crate is currently an unreleased `1.1.0` candidate. Provider identity and
the endpoint, request-target, and static-download redirect boundaries are now
implemented. Operations and credentials remain unavailable until their
independently reviewed checkpoints are complete.

## Current Boundary

| Area | State |
| --- | --- |
| Provider identity | `crates-io` |
| Service identity | `registry` |
| Default features | empty |
| Default target | `no_std` |
| Official endpoints | production API, staging API, and anonymous static downloads |
| Request targets | bounded `/api/v1/` API and `/crates/` static-download forms |
| Redirects | atomic production source proof plus atomic credential-free download execution |
| Custom API endpoints | HTTPS plus explicit trusted-operator acknowledgement |
| Credentials | deferred to Commit 5 |
| API operations | deferred to their source-locked implementation commits |

The public modules reserve ownership without claiming executable coverage:
`catalog`, `accounts`, `ownership`, `publishing`, and `trusted_publishing`.
The complete 51-operation scope is maintained in the
[crates.io source lock](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_SOURCE_LOCK.md).

## Endpoint Example

Official routing remains allocation-free and transport-neutral:

```rust
use cloud_sdk_cratesio::endpoint::{
    ApiRequestTarget, OfficialCratesIoEndpoint,
};

let endpoint = OfficialCratesIoEndpoint::production_api();
assert_eq!(endpoint.base_url(), "https://crates.io");

let target = ApiRequestTarget::new("/api/v1/crates?q=serde");
assert!(target.is_ok());
```

Custom API origins require
`CustomEndpointAcknowledgement::trusted_operator_configuration()` and an
already validated HTTPS `EndpointIdentity`. Values must never come from tenant,
request, webhook, or other attacker-controlled input. Static download
redirects accept only the exact `https://static.crates.io` authority, correlate
the crate and version with the source API target, and can be followed only
through a raw executor using an SDK-created bodyless `GET` with empty headers.
Source proof creation also dispatches through the exact production-bound raw
executor using an SDK-created bodyless `GET`, empty headers, and an exact
response policy; callers cannot combine an unrelated response with a separately
verified transport. The redirect does not expose endpoint or target pieces
that could be reused by an authenticated request. The complete contract is
documented in the [crates.io endpoint policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/CRATESIO_ENDPOINT_POLICY.md).

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps the provider allocation-free and `no_std`. |
| `alloc` | no | Enables future provider models that require `alloc`. |
| `serde` | no | Enables future bounded model serialization with no Serde `std` feature. |
| `std` | no | Enables `alloc` and standard-library integration. |
| `blocking` | no | Reserves provider-owned blocking execution integration; no transport dependency is added. |
| `async` | no | Reserves provider-owned async execution integration; no runtime or transport dependency is added. |

Networking and TLS remain opt-in provider-neutral concerns. This crate does
not depend on `cloud-sdk-reqwest`, an async runtime, a TLS implementation, a
filesystem, or a clock.

## Identity Example

```rust
use cloud_sdk::{ProviderMarker, ServiceMarker};
use cloud_sdk_cratesio::{
    CRATES_IO_PROVIDER_ID, CratesIo, REGISTRY_SERVICE_ID, RegistryService,
};

assert_eq!(CratesIo::ID, CRATES_IO_PROVIDER_ID);
assert_eq!(RegistryService::ID, REGISTRY_SERVICE_ID);
assert_eq!(
    <<RegistryService as ServiceMarker>::Provider as ProviderMarker>::ID,
    CRATES_IO_PROVIDER_ID,
);
```

## Security And Policy

The provider will not support browser-session cookies or undocumented private
routes. Credential contexts, one-request-per-second scheduling, identifying
user agents, mutation permits, and bounded response decoding are introduced
only in their assigned reviewed checkpoints.

Direct crates.io API use must follow the service's data-access policy. Prefer
the sparse index, static downloads, RSS feeds, or database dumps when those
sources fit the task.

See the main project's
[threat model](https://github.com/valkyoth/cloud-sdk/blob/main/docs/threat-model.md),
[release governance](https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_GOVERNANCE.md),
and [versioning policy](https://github.com/valkyoth/cloud-sdk/blob/main/docs/VERSIONING_POLICY.md).

## License

Licensed under either the MIT License or Apache License, Version 2.0, at your
option.
