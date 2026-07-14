# Provider-Neutral Quickstart

`cloud-sdk` separates provider request models from transport execution. The
default graph performs no I/O, selects no runtime, stores no token, and remains
usable in `no_std` environments.

## Install

```toml
[dependencies]
cloud-sdk = "0.21.0"
```

Provider-specific request models are separate dependencies. For Hetzner:

```toml
[dependencies]
cloud-sdk = "0.21.0"
cloud-sdk-hetzner = "0.18.0"
```

## Build A Transport Request

The provider-neutral request contract carries a validated origin-form target,
method, optional body, and optional content type:

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{RequestTarget, TransportRequest};

let target = RequestTarget::new("/servers?page=1")?;
let request = TransportRequest::new(Method::Get, target);

assert_eq!(request.target().as_str(), "/servers?page=1");
# Ok::<(), cloud_sdk::transport::RequestTargetError>(())
```

The complete compile-checked source is
[`provider_neutral.rs`](../crates/cloud-sdk/examples/provider_neutral.rs). Run
it with:

```sh
cargo run -p cloud-sdk --example provider_neutral
```

## Select A Transport

- Use `cloud-sdk-testkit` for deterministic blocking and async unit tests.
- Implement `BlockingTransport` or `AsyncTransport` for a platform-native
  transport.
- Enable `cloud-sdk-reqwest/blocking-rustls` or `async-rustls` when the
  supported native reqwest boundary fits the target.

Provider crates do not depend on transport crates. This keeps cloud request
models portable to Linux, Windows, BSD, macOS, Android, iOS, WASM, embedded
targets, and future operating systems while allowing each application to own
its networking and runtime policy.

## Continue

- [Hetzner workflow examples](HETZNER_EXAMPLES.md)
- [Security recipes](SECURITY_RECIPES.md)
- [Platform support](PLATFORM_SUPPORT.md)
- [Live smoke testing](LIVE_SMOKE_TESTING.md)
