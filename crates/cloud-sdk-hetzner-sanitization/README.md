# cloud-sdk-hetzner-sanitization

Optional secret-handling boundary for
[`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner), which belongs
to the main [`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace.

This crate exists so future token-adjacent sanitization helpers can be reviewed
outside the default no_std provider crate. It intentionally does not depend on a
third-party sanitization crate yet.

Most users should start with:

```toml
[dependencies]
cloud-sdk-hetzner = "0.3.0"
```

Use this crate only when the release notes say sanitization helpers have been
admitted.

## Current Example

```rust
use cloud_sdk_hetzner_sanitization::SanitizationStatus;

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
