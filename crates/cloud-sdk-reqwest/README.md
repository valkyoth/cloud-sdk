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

Optional provider-neutral transport-adapter boundary for the main
[`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace and
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate.

This crate exists so one future reviewed reqwest adapter can serve every cloud
provider without adding transport dependencies to provider crates. It
intentionally does not depend on `reqwest` yet.

Most users should start with:

```toml
[dependencies]
cloud-sdk = "0.12.0"
```

Use this crate only when the release notes say a transport adapter has been
admitted.

## Current Example

```rust
use cloud_sdk_reqwest::ReqwestAdapterStatus;

assert_eq!(
    ReqwestAdapterStatus::DependencyNotAdmitted,
    ReqwestAdapterStatus::DependencyNotAdmitted,
);
```

## Admission Requirements

Before a real reqwest dependency is admitted, the workspace must review:

- HTTP client version, license, features, and maintenance status;
- TLS policy;
- timeout and retry behavior;
- authentication header redaction;
- default feature impact;
- mock and live-test strategy.

Provider crates retain ownership of authentication, base URLs, request models,
response interpretation, and provider-specific errors. This crate must not
branch on provider names.
