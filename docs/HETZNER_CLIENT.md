# Hetzner Client Foundation

`cloud_sdk_hetzner::client` binds one authenticated transport to one Hetzner
service and one endpoint trust class. It adds no runtime, executor, queue,
clock, retry policy, filesystem access, or implicit allocation to the default
graph.

## Official Construction

Use the service-specific constructors:

- `HetznerClient::cloud(transport)`;
- `HetznerClient::dns(transport)`;
- `HetznerClient::security(transport)`;
- `HetznerClient::storage(transport)`.

Cloud, DNS, and security require the exact normalized
`https://api.hetzner.cloud/v1` identity. Console Storage requires
`https://api.hetzner.com/v1`. Construction fails before any request when the
scheme, host, effective port, or base path differs.

The aliases `CloudClient`, `DnsClient`, `SecurityClient`, and `StorageClient`
make stored client types concise. Each client exposes its compile-time service
ID and immutable bound transport. Its `Debug` output does not expose endpoint
or credential data.

## Custom Endpoint Trust

Custom constructors have conspicuous names such as
`HetznerClient::cloud_with_custom_endpoint`. They require
`CustomEndpointAcknowledgement::trusted_operator_configuration()` and reject
non-HTTPS identities. The configured host is a credential destination and
must never come from a tenant, request body, webhook, or other attacker-
controlled input.

Custom clients carry the separate `CustomEndpointTrust` type marker. v0.69
does not expose execution methods on that marker: existing Hetzner operations
remain source-locked to official endpoint identities. This avoids presenting a
custom client that appears usable but only fails after request preparation.
Custom execution requires a future explicit operation-policy binding; it will
not silently loosen official policies.

## Read-Only Execution

With the `serde` feature, official clients provide blocking, `Send` async, and
local async execution for associated read-only operations belonging to the
same service. A Cloud client cannot execute a DNS, security, or Console
Storage operation. State-changing operations do not implement this client
operation contract and remain behind plan-confirm permits.

Every call:

1. consumes one caller-owned `ClientWorkspaceLease`;
2. clears all four complete workspace regions;
3. prepares and endpoint-verifies exactly one associated operation;
4. sends exactly one authenticated attempt;
5. applies success or provider-error response policy;
6. decodes an owned typed `CheckedHetznerResponse`;
7. clears storage and releases the lease on return or cancellation.

The client is borrowed through `&self`, so a `Sync` transport supports caller-
bounded concurrency without holding a mutex across `.await`. The SDK creates
no tasks and performs no automatic retries.

## Workspace Profiles

`ClientCapacityProfile::EMBEDDED`, `DEFAULT`, and `LARGE` define complete
target, request-body, response-body, and response-header capacities. Use
`ClientWorkspace::for_profile` with caller-owned storage. With `alloc`,
`OwnedClientWorkspace::try_for_profile` fallibly allocates the same exact
bounded layout and wipes all four allocations on drop.

The generic associated-operation entry point is foundation-level API. Service-
specific convenience methods are delivered in v0.70 through v0.73. Until
those methods exist, construct requests with the associated operation types
documented in [`OPERATION_ASSOCIATIONS.md`](OPERATION_ASSOCIATIONS.md).
