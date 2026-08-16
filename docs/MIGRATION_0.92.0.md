# Migrating Source Users To v0.92.0

v0.92.0 is an additive internal milestone. It is tagged after review but is
not published to crates.io; the next planned public checkpoint is v0.95.0.

Source users should update the core dependency to the v0.92 tag while keeping
the provider package at its current independent `0.45.0` version:

```toml
[dependencies]
cloud-sdk = { git = "https://github.com/valkyoth/cloud-sdk", tag = "v0.92.0" }
cloud-sdk-hetzner = { git = "https://github.com/valkyoth/cloud-sdk", tag = "v0.92.0", version = "0.45.0", features = ["serde"] }
```

No existing API is removed or changed. New Robot transaction requests and
strict models are under `cloud_sdk_hetzner::robot`; owned response decoding
requires `serde`.

Robot transaction lists represent only the provider's fixed 30-day window and
are not pagers. They do not authorize a purchase or establish an independent
audit record. Detail requests should be preferred when reconciling one known
transaction ID.

All six request types expose `quota()`, which returns
`ROBOT_ORDER_TRANSACTION_QUOTA`: 500 requests per one-hour account window.
Enforce this as one shared account budget, including explicit retries, rather
than allocating 500 requests independently to each request type.

Use `PreparationStorageGuard` for every transaction preparation. Reachable
validation and encoding failures clear both buffers immediately. The guard also
owns cleanup if future invariant drift makes a typed late target or policy error
reachable after immutable target binding. Callers using raw
`PreparationStorage` remain responsible for clearing their buffers.
