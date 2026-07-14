<p align="center">
  <b>provider-neutral no_std testkit for cloud-sdk.</b><br>
  Deterministic mock transport, bounded response fixtures, and adversarial corpora.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">cloud-sdk crate</a>
  |
  <a href="https://docs.rs/cloud-sdk-testkit">Docs.rs</a>
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

# cloud-sdk-testkit

Provider-neutral testing support for the main
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate and its provider crates.
The default graph is no_std, allocation-free, network-free, filesystem-free,
and runtime-free.

```toml
[dev-dependencies]
cloud-sdk = "0.23.0"
cloud-sdk-testkit = "0.15.5"
```

## Mock Transport

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, RequestTarget, TransportRequest};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

let Ok(target) = RequestTarget::new("/servers?page=1") else {
    return;
};
let Ok(body) = FixtureBody::new(br#"{"servers":[]}"#) else {
    return;
};
let exchanges = [MockExchange::new(
    ExpectedRequest::new(Method::Get, target),
    ResponseFixture::success(body),
)];
let mut transport = MockTransport::new(&exchanges);
let mut output = [0_u8; 64];

let Ok(response) = transport.send(
    TransportRequest::new(Method::Get, target),
    &mut output,
) else {
    return;
};

assert_eq!(response.status().get(), 200);
assert_eq!(response.body(), br#"{"servers":[]}"#);
assert!(transport.is_complete());
```

The same mock implements the executor-neutral async contract without adding a
runtime dependency:

```rust,no_run
# async fn example() {
use cloud_sdk::Method;
use cloud_sdk::transport::{AsyncTransport, RequestTarget, TransportRequest};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

let Ok(target) = RequestTarget::new("/servers/42") else { return };
let Ok(body) = FixtureBody::new(br#"{"id":42}"#) else { return };
let exchanges = [MockExchange::new(
    ExpectedRequest::new(Method::Get, target),
    ResponseFixture::success(body),
)];
let mut transport = MockTransport::new(&exchanges);
let mut output = [0_u8; 32];
let Ok(response) = AsyncTransport::send(
    &mut transport,
    TransportRequest::new(Method::Get, target),
    &mut output,
).await else { return };

assert_eq!(response.body(), br#"{"id":42}"#);
# }
# fn main() {}
```

Each exchange is consumed only after the request matches and the complete
response body fits. Method, target, body, exhaustion, and response-capacity
failures are distinct and payload-free. Debug output redacts request targets,
request bodies, and response bodies.

## Fixture Builders

`ResponseFixture` builds deterministic success, paginated, action, rate-limit,
and error responses. `PaginationFixture`, `ActionFixture`, and
`RateLimitFixture` reject incoherent metadata before a fixture can be used.
Use `ResponseFixture::with_rate_limit` to attach validated transport metadata
to paginated, action, success, or error responses.

`FixtureBody` supports borrowed bytes and compact repeated-byte bodies up to
8 MiB plus one byte. Writes preflight capacity and leave undersized destination
buffers unchanged.

## Adversarial Corpus

`adversarial_corpus()` returns reusable cases for:

- malformed JSON;
- additive unknown fields;
- missing required fields;
- an oversized response represented without an 8 MiB static allocation;
- invalid pagination metadata;
- an invalid action state and progress value.

Provider crates consume applicable cases in their own parser tests. The
Hetzner Serde boundary exercises this corpus without making the testkit depend
on `cloud-sdk-hetzner`.

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps the testkit allocation-free, runtime-free, and `no_std`. |
| `alloc` | no | Enables allocation-bearing test helpers and `cloud-sdk/alloc`. |
| `std` | no | Enables `alloc` and standard-library integration without selecting a runtime. |

Docs.rs builds with all features. The mock transport remains network-free in
every configuration.

## Security Notes

This crate is test infrastructure, not a production transport. Exact request
matching uses ordinary byte equality and must not be exposed as a remote secret
comparison oracle. Authentication, base URLs, headers, timeout policy, TLS,
retry behavior, and secret ownership remain responsibilities of concrete
transport adapters.

The testkit stores only borrowed expectations and fixture bodies. Callers must
keep borrowed data alive and must still sanitize secret-bearing test buffers
when their threat model requires it.
