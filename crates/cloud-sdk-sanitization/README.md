<p align="center">
  <b>optional provider-neutral sanitization boundary for cloud-sdk.</b><br>
  Provider crates, explicit API domains, security-first release gates, and transport-free core types.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">cloud-sdk crate</a>
  |
  <a href="https://docs.rs/cloud-sdk-sanitization">Docs.rs</a>
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

# cloud-sdk-sanitization

Optional provider-neutral secret-handling boundary for the main
[`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace and
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate.

This crate exists so reusable token, credential, and caller-owned buffer
sanitization helpers can be reviewed outside the default no_std SDK and provider
crates. It intentionally does not depend on a sanitization implementation yet.

Most users should start with:

```toml
[dependencies]
cloud-sdk = "0.12.0"
```

Use this crate only when the release notes say sanitization helpers have been
admitted.

## Current Example

```rust
use cloud_sdk_sanitization::SanitizationStatus;

assert_eq!(
    SanitizationStatus::DependencyNotAdmitted,
    SanitizationStatus::DependencyNotAdmitted,
);
```

## Security Notes

Sanitization helpers do not replace review of token ownership, copies, logging,
environment variables, paging, crash dumps, compiler behavior, or process
boundaries. Any future dependency must be admitted with explicit release notes,
tests, and pentest evidence.
