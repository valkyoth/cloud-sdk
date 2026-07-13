<p align="center">
  <b>optional provider-neutral reqwest boundary for cloud-sdk.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">cloud-sdk crate</a>
  |
  <a href="https://docs.rs/cloud-sdk-reqwest">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/cloud-sdk">
    <img src="https://raw.githubusercontent.com/valkyoth/cloud-sdk/main/.github/images/cloud-sdk.webp" alt="cloud-sdk Rust crate overview">
  </a>
</p>

# cloud-sdk-reqwest

Optional provider-neutral transport adapter for the main
[`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace and
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate.

The crate remains no_std and transport-free by default. Its non-default
`blocking-rustls` feature provides the reviewed blocking HTTPS implementation
for every provider without adding transport dependencies to provider crates.

Most users should start with:

```toml
[dependencies]
cloud-sdk = "0.16.0"
cloud-sdk-reqwest = { version = "0.13.0", features = ["blocking-rustls"] }
```

## Blocking Example

```rust,no_run
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, RequestTarget, TransportRequest};
use cloud_sdk_reqwest::blocking::{
    BearerToken, BlockingClientBuilder, HttpsEndpoint, RequestTimeouts,
    UserAgent,
};

let Ok(endpoint) = HttpsEndpoint::new("https://api.hetzner.cloud/v1") else { return };
let Ok(token) = BearerToken::new("replace-with-scoped-token") else { return };
let Ok(user_agent) = UserAgent::new("my-service/1.0") else { return };
let Ok(timeouts) = RequestTimeouts::new(
    Duration::from_secs(30),
    Duration::from_secs(10),
) else { return };
let Ok(mut client) = BlockingClientBuilder::new(endpoint, token, user_agent, timeouts).build()
else { return };

let Ok(target) = RequestTarget::new("/servers?page=1") else { return };
let request = TransportRequest::new(Method::Get, target);
let mut response_body = [0_u8; 65_536];
let Ok(response) = client.send(request, &mut response_body) else { return };

assert!(response.status().is_success());
```

For a non-empty request body, set an explicit validated content type:

```rust
use cloud_sdk::transport::{ContentType, TransportRequest};
# use cloud_sdk::{Method, transport::RequestTarget};
# let Ok(target) = RequestTarget::new("/servers") else { return };

let request = TransportRequest::new(Method::Post, target)
    .with_body(br#"{"name":"example"}"#)
    .with_content_type(ContentType::JSON);
assert_eq!(request.content_type(), Some(ContentType::JSON));
```

## Enforced Policy

- HTTPS-only production endpoints with no embedded credentials, query, or
  fragment.
- Rustls with TLS 1.2 minimum and platform certificate verification.
- Explicit total and connect timeouts, each nonzero and at most 300 seconds.
- Explicit validated user agent and bounded bearer token.
- HTTP/1 and the system resolver are forced even under downstream reqwest
  HTTP/2 or Hickory DNS feature unification.
- No redirects, automatic retries, proxies, referer generation, or response
  decompression.
- Exact scheme, host, and port preservation after target composition.
- Caller-sized response buffers with overflow detection and cleanup.
- Payload-free errors and redacted client, token, target, and body diagnostics.

`BearerToken` clears its adapter-owned authorization bytes through
`cloud-sdk-sanitization`. It cannot clear the caller's original immutable
string or copies owned by reqwest, TLS, the operating system, or remote
services. Keep tokens scoped, rotate and revoke them, and erase caller-owned
mutable secret storage after transport use.

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `std` | no | Enables only std support in first-party boundary crates. |
| `blocking-rustls` | no | Enables the hardened blocking reqwest/rustls adapter and sanitization boundary. |

Reqwest's default features are disabled. The complete dependency and security
decision is recorded in
[`docs/dependency-admission-reqwest.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/dependency-admission-reqwest.md).

Provider crates retain ownership of authentication, base URLs, request models,
response interpretation, and provider-specific errors. This crate must not
branch on provider names.
