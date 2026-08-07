# Migrating To v0.61.0

v0.61.0 is an internal source milestone after the published v0.60.0
checkpoint. No crate is published and no public library API changes.

Applications remain on the published versions:

```toml
[dependencies]
cloud-sdk = "0.60.0"
cloud-sdk-hetzner = "0.39.1"
cloud-sdk-reqwest = "0.33.0"
cloud-sdk-sanitization = "0.18.0"
cloud-sdk-testkit = "0.29.1"
```

The repository-only `ovhcloud-v2-probe` package is test evidence. It is not a
supported provider, cannot be published, and must not be used as an
application dependency. The next cumulative crates.io checkpoint is v0.65.0.
