# Migrating Source Users To v0.91.0

v0.91.0 is an additive internal milestone. It is tagged after review but is
not published to crates.io; the next planned public checkpoint is v0.95.0.

Source users should update the core dependency to the v0.91 tag while keeping
the provider package at its current independent `0.45.0` version:

```toml
[dependencies]
cloud-sdk = { git = "https://github.com/valkyoth/cloud-sdk", tag = "v0.91.0" }
cloud-sdk-hetzner = { git = "https://github.com/valkyoth/cloud-sdk", tag = "v0.91.0", version = "0.45.0", features = ["serde"] }
```

No existing API is removed or changed. New Robot ordering catalogs are under
`cloud_sdk_hetzner::robot` and require `serde` for strict owned response
models. Request preparation itself remains transport-free.

Catalog-derived plans are intentionally non-executable. Do not treat their
prices as quotes or attempt to convert them into raw purchase forms. Fetch and
review current catalog data again when the later billable ordering API becomes
available.
