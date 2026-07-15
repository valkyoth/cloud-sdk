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
`blocking-rustls`, `blocking-rustls-webpki-roots`, `blocking-rustls-fips`, and
`async-rustls` features provide reviewed HTTPS implementations for every
provider without adding transport dependencies to provider crates.

## Install

```toml
[dependencies]
cloud-sdk = "0.30.0"
cloud-sdk-reqwest = { version = "0.20.1", features = ["blocking-rustls"] }
```

The examples use Hetzner as a concrete endpoint, but the adapter contains no
provider-specific routing, authentication, or response logic.
Response metadata changes from the previous release are listed in the
[v0.29 migration guide](https://github.com/valkyoth/cloud-sdk/blob/main/docs/MIGRATION_0.29.0.md).

## Blocking Example

```rust,no_run
# #[cfg(feature = "blocking-rustls")]
# fn main() {
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, RequestTarget, TransportRequest};
use cloud_sdk_reqwest::blocking::{
    BearerToken, BlockingClientBuilder, HttpsEndpoint, RequestTimeouts,
    UserAgent,
};

// Custom endpoints are bearer-token destinations. Keep this value in trusted
// operator configuration; never accept it from tenant-controlled input.
let Ok(endpoint) = HttpsEndpoint::new_custom("https://api.hetzner.cloud/v1") else { return };
let Ok(token) = BearerToken::new("replace-with-scoped-token") else { return };
let Ok(user_agent) = UserAgent::new("my-service/1.0") else { return };
let Ok(timeouts) = RequestTimeouts::new(
    Duration::from_secs(30),
    Duration::from_secs(10),
) else { return };
let Ok(client) = BlockingClientBuilder::new(endpoint, token, user_agent, timeouts).build()
else { return };

let Ok(target) = RequestTarget::new("/servers?page=1") else { return };
let request = TransportRequest::new(Method::Get, target);
let mut response_body = [0_u8; 65_536];
let Ok(response) = client.send(request, &mut response_body) else { return };

assert!(response.status().is_success());
# }
# #[cfg(not(feature = "blocking-rustls"))]
# fn main() {}
```

Responses retain one validated, bounded `Content-Type` value for prepared
response policy. Missing content type remains explicit; malformed, non-textual,
or duplicate values fail closed before response bytes are returned.

Both adapters implement `ResponseStorageSanitizer` through
`cloud-sdk-sanitization`. Prepared execution therefore volatile-clears the
complete caller buffer before endpoint checks and before lending the smaller
operation-admitted response window. Direct transport sends continue clearing
the complete slice passed to `send`.

## Deterministic Root Snapshot

The standard blocking feature follows the host trust store. Select the
separate deterministic feature to use only the reviewed Mozilla root snapshot
compiled into `webpki-roots`:

```toml
[dependencies]
cloud-sdk = "0.30.0"
cloud-sdk-reqwest = { version = "0.20.1", features = ["blocking-rustls-webpki-roots"] }
```

The blocking API is identical to the example above. The custom rustls client
configuration receives only the compiled snapshot, even though reqwest still
compiles its platform-verifier dependency. Host and enterprise roots are not
consulted by this client. Root changes require a reviewed dependency update.
This mode does not add CRL/OCSP revocation checking, private roots, pinning, or
FIPS status. When combined with `blocking-rustls-fips`, the FIPS policy wins.

## Blocking FIPS Example

Use the same blocking API with the dedicated feature:

```toml
[dependencies]
cloud-sdk = "0.30.0"
cloud-sdk-reqwest = { version = "0.20.1", features = ["blocking-rustls-fips"] }
rustls = "=0.23.42"
```

```rust,no_run
# #[cfg(feature = "blocking-rustls-fips")]
# fn main() {
use rustls::RootCertStore;
use rustls::pki_types::{CertificateDer, CertificateRevocationListDer};
use cloud_sdk_reqwest::blocking::{BlockingClientBuilder, FipsTlsPolicy};

# fn configure(
#     builder: BlockingClientBuilder,
#     root_der: Vec<u8>,
#     crl_der: Vec<u8>,
# ) {
let mut roots = RootCertStore::empty();
let Ok(()) = roots.add(CertificateDer::from(root_der)) else { return };
let Ok(policy) = FipsTlsPolicy::new(
    roots,
    vec![CertificateRevocationListDer::from(crl_der)],
) else { return };
let Ok(_client) = builder.with_fips_tls_policy(policy).build() else { return };
# }
# }
# #[cfg(not(feature = "blocking-rustls-fips"))]
# fn main() {}
```

The application must authenticate, refresh, and supply complete CRLs for every
issuer in an accepted chain. Construction rejects missing roots, missing or
malformed CRLs, and a missing policy; handshakes reject unknown revocation
status and expired CRLs. Client construction also fails closed unless both the
provider and complete TLS client configuration report FIPS operation. If both
blocking features are enabled, this explicit FIPS configuration wins.

A crate feature is not an application or deployment compliance claim; callers
remain responsible for the validated module's security policy, approved
operating environment, reviewed application lockfile or vendored sources,
toolchain, entropy, deployment, and operational controls. See
[`docs/dependency-admission-reqwest-fips.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/dependency-admission-reqwest-fips.md).

## Async Example

The async adapter uses reqwest's Tokio-based execution internally but does not
create or own a runtime. Call it from an active Tokio executor:

```rust,no_run
# #[cfg(feature = "async-rustls")]
# async fn example() {
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{AsyncTransport, RequestTarget, TransportRequest};
use cloud_sdk_reqwest::asynchronous::{
    AsyncClientBuilder, BearerToken, HttpsEndpoint, RequestTimeouts, UserAgent,
};

// Custom endpoints are bearer-token destinations. Keep this value in trusted
// operator configuration; never accept it from tenant-controlled input.
let Ok(endpoint) = HttpsEndpoint::new_custom("https://api.hetzner.cloud/v1") else { return };
let Ok(token) = BearerToken::new("replace-with-scoped-token") else { return };
let Ok(user_agent) = UserAgent::new("my-service/1.0") else { return };
let Ok(timeouts) = RequestTimeouts::new(
    Duration::from_secs(30),
    Duration::from_secs(10),
) else { return };
let Ok(client) = AsyncClientBuilder::new(endpoint, token, user_agent, timeouts).build()
else { return };

let Ok(target) = RequestTarget::new("/servers?page=1") else { return };
let request = TransportRequest::new(Method::Get, target);
let mut response_body = [0_u8; 65_536];
let Ok(response) = AsyncTransport::send(&client, request, &mut response_body).await
else { return };

assert!(response.status().is_success());
# }
# fn main() {}
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

## Shared Clients And Credential Rotation

Blocking and async clients are `Clone + Send + Sync`. Clones share one
credential state and one immutable endpoint identity, while every request body
and response buffer remains caller-owned. The SDK does not create tasks,
queues, semaphores, retries, sleeps, or an executor; callers must bound their
own blocking threads or async task sets.

Both core transport traits send through `&self`. A request takes a short-lived
token snapshot before I/O and releases the credential lock before network work
or `.await`. Rotation changes the token for newly started requests atomically;
an in-flight request keeps its previous snapshot, and retired adapter-owned
storage is sanitized after its last snapshot is dropped.

```rust,no_run
# #[cfg(feature = "blocking-rustls")]
# fn example(client: &cloud_sdk_reqwest::blocking::BlockingClient) {
use cloud_sdk::transport::{BoundTransport, EndpointScheme};

let official = client.endpoint_identity().is_ok_and(|identity| {
    identity.scheme() == EndpointScheme::Https
        && identity.host() == "api.hetzner.cloud"
        && identity.effective_port() == 443
        && identity.base_path() == "/v1"
});
assert!(official);

let mut replacement = *b"replace-with-scoped-token";
let result = client.rotate_bearer_token_from_mut_bytes(&mut replacement);
assert!(result.is_ok());
assert!(replacement.iter().all(|byte| *byte == 0));
# }
# fn main() {}
```

`BearerToken::from_mut_bytes` and the matching client rotation method clear the
complete mutable source on success or rejection. `BearerToken::from_secret_buffer`
and `rotate_bearer_token_from_secret_buffer` consume a
`cloud_sdk_sanitization::SecretBuffer`, which provides the same cleanup on
every return path. The compatibility `BearerToken::new(&str)` constructor
cannot clear its immutable source. Construct a replacement before calling
`rotate_bearer_token`, or use one of the source-clearing rotation methods;
rejected input leaves the active credential unchanged.

## Enforced Policy

- HTTPS-only production endpoints with no embedded credentials, query, or
  fragment.
- Rustls with TLS 1.2 minimum; platform certificate verification for standard
  transports, deterministic Mozilla roots for the snapshot feature, and
  mandatory deployment roots plus CRLs for FIPS.
- Explicit total and connect timeouts, each nonzero and at most 300 seconds.
- Explicit validated user agent and bounded bearer token.
- HTTP/1 and the system resolver are forced even under downstream reqwest
  HTTP/2 or Hickory DNS feature unification.
- No redirects, automatic retries, proxies, referer generation, or response
  decompression.
- Exact scheme, host, and port preservation after target composition.
- Immutable normalized scheme, host, effective port, and base-path identity for
  provider-side official-endpoint checks.
- Shared-reference sends with cloneable clients, caller-bounded concurrency,
  and no credential lock held across I/O or `.await`.
- Atomic token rotation with in-flight snapshots and source-clearing mutable or
  guarded constructors.
- Caller-sized response buffers with overflow detection and cleanup.
- Strict all-or-none decimal parsing and propagation of exactly one
  `RateLimit-Limit`, `RateLimit-Remaining`, and `RateLimit-Reset` response
  header; duplicates fail closed.
- Async responses are buffered within the caller's capacity and copied only
  after complete success; cancellation leaves the caller buffer cleared.
- Payload-free errors and redacted client, token, target, and body diagnostics.

`BearerToken` clears its adapter-owned authorization bytes through
`cloud-sdk-sanitization`. Rotation cannot clear authorization copies already
owned by reqwest, TLS, the operating system, or remote services. Keep tokens
scoped, rotate and revoke them, and use mutable or guarded ingestion whenever
the source can be cleared.

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps the crate transport-free and `no_std`. |
| `std` | no | Enables only std support in first-party boundary crates. |
| `blocking-rustls` | no | Enables the hardened blocking reqwest/rustls adapter and sanitization boundary. |
| `blocking-rustls-webpki-roots` | no | Enables the blocking adapter with a deterministic reviewed Mozilla root snapshot. |
| `blocking-rustls-fips` | no | Enables the blocking adapter with runtime-verified AWS-LC FIPS plus mandatory deployment roots and CRLs. |
| `async-rustls` | no | Enables the hardened async reqwest/rustls adapter; callers provide an active Tokio runtime. |

Reqwest's default features are disabled. The complete dependency and security
decision is recorded in
[`docs/dependency-admission-reqwest.md`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/dependency-admission-reqwest.md).

Provider crates retain ownership of authentication, base URLs, request models,
response interpretation, and provider-specific errors. This crate must not
branch on provider names.
