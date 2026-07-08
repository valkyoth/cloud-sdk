# cloud-sdk-hetzner-reqwest

Optional transport-adapter boundary for
[`cloud-sdk-hetzner`](https://crates.io/crates/cloud-sdk-hetzner), which belongs
to the main [`cloud-sdk`](https://github.com/valkyoth/cloud-sdk) workspace.

This crate exists so a future reviewed reqwest adapter can live outside the
default no_std provider crate. It intentionally does not depend on `reqwest`
yet.

Most users should start with:

```toml
[dependencies]
cloud-sdk-hetzner = "0.6.0"
```

Use this crate only when the release notes say a transport adapter has been
admitted.

## Current Example

```rust
use cloud_sdk_hetzner_reqwest::ReqwestAdapterStatus;

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
