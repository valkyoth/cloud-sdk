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
| Checked response decoding | [`checked_response.rs`](../crates/cloud-sdk-hetzner/examples/checked_response.rs) | `cargo run -p cloud-sdk-hetzner --example checked_response --features serde` |
| Complete Cloud client read | [`cloud_client.rs`](../crates/cloud-sdk-hetzner/examples/cloud_client.rs) | `cargo run -p cloud-sdk-hetzner --example cloud_client --features serde` |
| Complete DNS client read | [`dns_client.rs`](../crates/cloud-sdk-hetzner/examples/dns_client.rs) | `cargo run -p cloud-sdk-hetzner --example dns_client --features serde` |
| Complete Security client read | [`security_client.rs`](../crates/cloud-sdk-hetzner/examples/security_client.rs) | `cargo run -p cloud-sdk-hetzner --example security_client --features serde` |
| Global and certificate action queries | [`actions.rs`](../crates/cloud-sdk-hetzner/examples/actions.rs) | `cargo run -p cloud-sdk-hetzner --example actions` |
| DNS Zone request | [`dns.rs`](../crates/cloud-sdk-hetzner/examples/dns.rs) | `cargo run -p cloud-sdk-hetzner --example dns` |
| Storage Box list request | [`storage_box.rs`](../crates/cloud-sdk-hetzner/examples/storage_box.rs) | `cargo run -p cloud-sdk-hetzner --example storage_box` |

## Execution Boundary

Prepared operations combine the first three steps into one checked,
provider-owned contract:

1. Validate provider-specific input and endpoint policy.
2. Write the complete target and JSON body into caller-owned bounded buffers.
3. Bind the provider-neutral request, official endpoint, operation metadata,
   exact provider service, authentication scope, checked response policy, and
   raw wire policy.
4. Send only after the application has reviewed credentials, operation cost,
   timeout, retry, logging, response-size policy, and the exact plan-confirm
   execution permit required by state-changing metadata.

The provider crate covers preparation and checked typed envelope decoding for
all 208 active operations. v0.70 exposes named client workflows for all 139
active Cloud operations, and v0.71 does the same for all 24 active DNS
operations. v0.72 adds all 14 certificate and SSH-key operations. Read-only
methods prepare, execute, enforce response policy, and decode through one
caller-owned workspace lease. State-changing methods retain separate named
preparation and permit-authorized execution, so review and confirmation cannot
be skipped. Console methods arrive in v0.73; their generic associated-operation
paths remain available. The client selects no transport, retry policy, runtime,
clock, or secret store.

## Mutation Safety

Creating, updating, and deleting cloud resources may incur cost or downtime.
Before adding a transport call to a mutation example:

- use a dedicated project and least-privilege credential;
- inspect the final method, path, query, and body;
- set explicit connect and total timeouts;
- make retry behavior operation-specific and idempotency-aware;
- cap response bodies and redact credentials, bodies, and resource IDs;
- verify provider pricing and cleanup behavior.
- build an exact plan-confirm fingerprint and consume the matching mutation,
  destructive, or cost permit; direct prepared execution fails closed.

See [Security Recipes](SECURITY_RECIPES.md) before connecting these models to a
live account, and [Execution Permits](EXECUTION_PERMITS.md) for the mandatory
state-changing execution lifecycle.
