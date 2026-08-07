# Migrating To v0.60.0

v0.60.0 is the cumulative public checkpoint for signed milestones v0.56.0
through v0.60.0. It follows the published v0.55.0 baseline.

```toml
[dependencies]
cloud-sdk = "0.60.0"
cloud-sdk-hetzner = "0.39.1"
cloud-sdk-reqwest = "0.33.0"
cloud-sdk-sanitization = "0.18.0"
cloud-sdk-testkit = "0.29.1"
```

Only add the crates and opt-in features the application uses.

## Public API Additions

- v0.58 adds exact endpoint-pair policy, expiring credential decisions, and
  atomic bearer refresh in the reqwest adapter.
- v0.59 adds prepared-request-bound header cursor sessions and reviewed schema
  versions. Exhaustive `PaginationError` matches must admit the variants
  documented in [`MIGRATION_0.59.0.md`](MIGRATION_0.59.0.md).
- v0.60 adds `cloud_sdk::async_resource`: bounded borrowed identifiers, text,
  links, timestamps, task/progress/error snapshots, generic event fixtures,
  and action-polling conversion.

No existing default feature is enabled. The default graph remains
allocation-free and `no_std`, and no provider, transport, runtime, clock, or
parser is selected automatically.

## OVHcloud Probe Boundary

The OVHcloud v2 material remains excluded conformance evidence. The new task
model is challenged against two source-locked production read routes, but it
is not an OVHcloud client or response decoder. Event models remain generic
fixtures because no event route is claimed in the reviewed source surface.

See [`ASYNC_RESOURCES.md`](ASYNC_RESOURCES.md) and
[`provider-probes/ovhcloud-v2/README.md`](../provider-probes/ovhcloud-v2/README.md).
