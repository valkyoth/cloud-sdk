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

Custom clients carry the separate `CustomEndpointTrust` type marker. v0.70
does not expose execution methods on that marker: existing Hetzner operations
remain source-locked to official endpoint identities. This avoids presenting a
custom client that appears usable but only fails after request preparation.
Custom execution requires a future explicit operation-policy binding; it will
not silently loosen official policies.

## Named Client Methods

With the `serde` feature, the official Cloud client exposes named methods for
all 139 active Cloud operations. Each read operation has blocking, `Send`
async, and local-async methods. Mutation, destructive, and cost-bearing
operations have a named cleanup-owning preparation method plus three execution
methods that accept only the operation's matching `AssociatedPermitAttempt`.

The official DNS client applies the same contract to all 24 active DNS
operations: eight read-only, nine mutation, and seven destructive methods.
The four list operations retain numbered-pagination policy, action endpoints
decode through the checked action models, and zonefile or TSIG-bearing
preparation stays in caller-owned cleanup-guarded storage.

The split is intentional: preparation creates no authority. The caller must
review the exact prepared request, build an `AssociatedPlanConfirmation`,
fingerprint it, create the required mutation/destructive/cost permit, and begin
one attempt before the client can send the request. The client never creates a
permit or retries an attempt implicitly.

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

`CLOUD_CLIENT_METHODS` and `DNS_CLIENT_METHODS` expose the exhaustive operation
descriptors behind the named surfaces for auditing and tooling. Their 139 and
24 rows are generated from the source-locked operation association manifest
and checked for exact permit and pagination classifications.

Security and Console Storage Box named methods are delivered in v0.72-v0.73.
Their generic associated-operation execution remains available; construct
requests with the types documented in
[`OPERATION_ASSOCIATIONS.md`](OPERATION_ASSOCIATIONS.md).
