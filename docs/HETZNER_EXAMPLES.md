# Hetzner Workflow Examples

The `cloud-sdk-hetzner` examples build or decode reviewed SDK types without
performing network operations. Mutation examples stop before transport
execution so running them cannot create billable resources.

## Workflow Index

| Workflow | Source | Command |
| --- | --- | --- |
| Complete prepared mutation | [`prepared.rs`](../crates/cloud-sdk-hetzner/examples/prepared.rs) | `cargo run -p cloud-sdk-hetzner --example prepared` |
| Read-only catalog request | [`read_only.rs`](../crates/cloud-sdk-hetzner/examples/read_only.rs) | `cargo run -p cloud-sdk-hetzner --example read_only` |
| Server mutation request | [`mutation.rs`](../crates/cloud-sdk-hetzner/examples/mutation.rs) | `cargo run -p cloud-sdk-hetzner --example mutation` |
| Pagination response | [`pagination.rs`](../crates/cloud-sdk-hetzner/examples/pagination.rs) | `cargo run -p cloud-sdk-hetzner --example pagination --features serde` |
| Action polling | [`action_polling.rs`](../crates/cloud-sdk-hetzner/examples/action_polling.rs) | `cargo run -p cloud-sdk-hetzner --example action_polling --features serde` |
| Global and certificate action queries | [`actions.rs`](../crates/cloud-sdk-hetzner/examples/actions.rs) | `cargo run -p cloud-sdk-hetzner --example actions` |
| DNS Zone request | [`dns.rs`](../crates/cloud-sdk-hetzner/examples/dns.rs) | `cargo run -p cloud-sdk-hetzner --example dns` |
| Storage Box list request | [`storage_box.rs`](../crates/cloud-sdk-hetzner/examples/storage_box.rs) | `cargo run -p cloud-sdk-hetzner --example storage_box` |

## Execution Boundary

Prepared operations combine the first three steps into one checked,
provider-owned contract:

1. Validate provider-specific input and endpoint policy.
2. Write the complete target and JSON body into caller-owned bounded buffers.
3. Bind the provider-neutral request, official endpoint, operation metadata,
   and response policy.
4. Send only after the application has reviewed credentials, operation cost,
   timeout, retry, logging, and response-size policy.

The provider crate covers the first three steps for all 208 active operations.
It does not yet provide a high-level client that also performs transport and
typed response decoding. That boundary is explicit so application code cannot
silently inherit networking, retry, runtime, or secret-storage behavior.

## Mutation Safety

Creating, updating, and deleting cloud resources may incur cost or downtime.
Before adding a transport call to a mutation example:

- use a dedicated project and least-privilege credential;
- inspect the final method, path, query, and body;
- set explicit connect and total timeouts;
- make retry behavior operation-specific and idempotency-aware;
- cap response bodies and redact credentials, bodies, and resource IDs;
- verify provider pricing and cleanup behavior.

See [Security Recipes](SECURITY_RECIPES.md) before connecting these models to a
live account.
