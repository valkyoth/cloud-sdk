<p align="center">
  <b>provider-neutral testkit boundary for cloud-sdk.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
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

Provider-neutral testkit boundary for the main
[`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace and
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate.

This crate is reserved for deterministic mock transports, source-locked
protocol fixtures, fault injection, and adversarial response cases reusable by
every provider. It intentionally stays small until provider-neutral transport
and response contracts are implemented.

Most users should start with:

```toml
[dependencies]
cloud-sdk = "0.12.0"
```

Use this crate for tests once fixture helpers are admitted.

## Current Example

```rust
use cloud_sdk_testkit::FixtureKind;

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

Provider-specific fixtures remain in their provider crates and compose these
generic primitives. This crate must not depend on provider crates or collect a
feature for every provider.
