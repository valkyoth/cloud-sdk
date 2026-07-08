# cloud-sdk-hetzner-testkit

Testkit boundary for
[`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner), which belongs
to the main [`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace.

This crate is reserved for deterministic mock transports, source-locked
fixtures, and adversarial Hetzner API response cases. It intentionally stays
small until endpoint models and response parsing are implemented.

Most users should start with:

```toml
[dependencies]
cloud-sdk-hetzner = "0.3.0"
```

Use this crate for tests once fixture helpers are admitted.

## Current Example

```rust
use cloud_sdk_hetzner_testkit::FixtureKind;

let fixture = FixtureKind::Pagination;
assert_eq!(fixture, FixtureKind::Pagination);
```

## Planned Fixture Areas

- Pagination responses.
- Action polling responses.
- Error envelopes.
- Rate-limit metadata.
- Malformed and oversized API responses.
- Deprecated endpoint behavior.
