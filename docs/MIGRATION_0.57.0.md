# Migrating Source Users To v0.57.0

v0.57.0 is a tagged source milestone. The latest crates.io checkpoint remains
v0.55.0, and this milestone changes no Rust public API or runtime behavior.

Repositories consuming `cloud-sdk` from Git may update their pinned revision
without source changes. Continue using the published package versions:

```toml
cloud-sdk = "0.55.0"
cloud-sdk-hetzner = "0.39.0"
cloud-sdk-reqwest = "0.32.4"
cloud-sdk-sanitization = "0.18.0"
cloud-sdk-testkit = "0.29.0"
```

Provider maintainers can inspect the excluded source-lock probe under
[`provider-probes/ovhcloud-v2`](../provider-probes/ovhcloud-v2/README.md). It
is architecture evidence for later milestones, not a package, supported
provider, client, credential workflow, or stable integration surface.
