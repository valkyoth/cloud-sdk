# Migrating Source Users To v0.74.0

v0.74.0 is an internal source milestone. The latest crates.io checkpoint
remains v0.70.0, and cumulative publication is deferred to v0.75.0.

## Published API

There is no source migration. Published Rust APIs, features, dependency
boundaries, and provider behavior are unchanged from v0.73.0.

## Robot Planning Evidence

Repository consumers can inspect the complete normalized inventory in
[`tests/fixtures/robot-api/v0.74.0.json`](../tests/fixtures/robot-api/v0.74.0.json)
and the grouped matrix in [`API_MATRIX.md`](API_MATRIX.md). The evidence does
not expose executable Robot operations yet.

Do not build integrations against the operation IDs as if they were stable
Rust APIs. Endpoint-family implementations begin after the form, credential,
and error protocol milestones, and each later release owns its public review.

Continue using `cloud_sdk_hetzner::storage` for Storage Boxes. The 16 legacy
Robot `/storagebox` operations and deprecated server-IP aliases will not be
added.
