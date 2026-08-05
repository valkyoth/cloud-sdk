# Migrating Source Users To v0.56.0

v0.56.0 is a tagged source milestone. The latest crates.io checkpoint remains
v0.55.0, and no Rust public API changes in this milestone.

Repositories consuming `cloud-sdk` from Git may update their pinned revision
without source changes. Continue using the published package versions:

```toml
cloud-sdk = "0.55.0"
cloud-sdk-hetzner = "0.39.0"
cloud-sdk-reqwest = "0.32.4"
cloud-sdk-sanitization = "0.18.0"
cloud-sdk-testkit = "0.29.0"
```

Provider maintainers should adopt the strict documents and workflow described
in [`PROVIDER_DRIFT.md`](PROVIDER_DRIFT.md) when source-locking a new provider.
Existing Hetzner source-lock workflows remain authoritative and are checked by
the neutral compatibility bridge.
