# Migrating To v0.33

v0.33 replaces the closed four-variant `Method` enum with a validated method
value. Existing known-method construction remains source compatible.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.33.0"
cloud-sdk-hetzner = "0.26.0"
cloud-sdk-reqwest = { version = "0.21.0", features = ["blocking-rustls"] }
```

Dependency-only boundary releases are:

- `cloud-sdk-sanitization 0.15.1`
- `cloud-sdk-testkit 0.18.4`

## Known Methods

Existing code does not need to change:

```rust
use cloud_sdk::Method;

assert_eq!(Method::Get.as_str(), "GET");
assert_eq!(Method::Post.as_str(), "POST");
assert_eq!(Method::Put.as_str(), "PUT");
assert_eq!(Method::Delete.as_str(), "DELETE");
```

PATCH, HEAD, and origin-form OPTIONS are now available:

```rust
use cloud_sdk::Method;

assert_eq!(Method::Patch.as_str(), "PATCH");
assert_eq!(Method::Head.as_str(), "HEAD");
assert_eq!(Method::Options.as_str(), "OPTIONS");
```

## Provider Extension Methods

Provider crates can define a finite static extension without changing core:

```rust
use cloud_sdk::Method;

const PURGE: Method = match Method::extension("PURGE") {
    Ok(method) => method,
    Err(_) => panic!("invalid provider method"),
};

assert_eq!(PURGE.as_str(), "PURGE");
```

Extensions are bounded to 32 bytes, must be uppercase canonical HTTP tokens,
and cannot alias a known method. CONNECT and TRACE are denied.

## Exhaustive Matches

Code that exhaustively matched the old enum must use equality or the canonical
token:

```rust
use cloud_sdk::Method;

fn is_read_method(method: Method) -> bool {
    method == Method::Get || method == Method::Head
}

assert!(is_read_method(Method::Head));
```

Do not infer retry, idempotency, destructive effect, or cost from
`Method::as_str()`. Those properties belong to explicit provider operation
metadata.

## Unsupported Protocol Modes

`OPTIONS *`, CONNECT, TRACE, HTTP upgrade, and tunnelling remain unavailable.
They require request-target, header, connection ownership, and response
lifecycle contracts that are not represented by the current SDK.
