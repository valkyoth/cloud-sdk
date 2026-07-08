<p align="center">
  <b>no_std-first multi-provider cloud SDK for Rust.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://docs.rs/cloud-sdk">Docs.rs</a>
  |
  <a href="docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="docs/threat-model.md">Threat Model</a>
  |
  <a href="SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/cloud-sdk">
    <img src="https://raw.githubusercontent.com/valkyoth/cloud-sdk/main/.github/images/cloud-sdk.webp" alt="cloud-sdk Rust crate overview">
  </a>
</p>

# cloud-sdk

Provider-neutral foundation crate for the `cloud-sdk` workspace.

This crate belongs to the main [`cloud-sdk`](https://github.com/valkyoth/cloud-sdk)
project. It contains shared, no_std-first domains that provider crates can use
without pulling in HTTP clients, TLS, async runtimes, token storage, serde, or
filesystem dependencies.

Most Hetzner users should read and depend on
[`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner). That crate is
the main documentation surface for Hetzner Cloud, DNS, security, and Storage Box
API planning.

## Install

```toml
[dependencies]
cloud-sdk = "0.5.0"
```

## Example

```rust
use cloud_sdk::{ApiFamily, Method, Provider};

let provider = Provider::Hetzner;
let family = ApiFamily::Cloud;
let method = Method::Get;

assert_eq!(provider, Provider::Hetzner);
assert_eq!(family, ApiFamily::Cloud);
assert_eq!(method, Method::Get);
```

## Scope

- Default features are empty.
- The crate is `no_std` by default.
- It is provider-neutral and intentionally small.
- Provider-specific examples live in provider crates such as
  `cloud-sdk-hetzner`.
