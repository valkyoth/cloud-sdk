<p align="center">
  <b>Hetzner testkit boundary for cloud-sdk.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">cloud-sdk crate</a>
  |
  <a href="https://docs.rs/cloud-sdk-hetzner-testkit">Docs.rs</a>
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
cloud-sdk-hetzner = "0.8.0"
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
