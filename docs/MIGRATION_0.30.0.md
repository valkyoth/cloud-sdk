# Migrating To v0.30

`cloud-sdk` `0.30.0` and `cloud-sdk-hetzner` `0.23.0` complete the prepared
request layer for the source-locked Hetzner Cloud, DNS, and Console Storage
surface. Default features remain empty and no transport, runtime, allocator,
TLS, filesystem, clock, or secret-storage dependency was added.

## Prepare Provider Requests

Aggregate request models now implement `PrepareOperation`. Supply bounded
target and body buffers and retain them for the lifetime of the prepared
request:

```rust
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk_hetzner::cloud::load_balancers::{
    LoadBalancerCreateRequest, LoadBalancerName, LoadBalancerType,
};

let name = LoadBalancerName::new("edge")?;
let load_balancer_type = LoadBalancerType::new("lb11")?;
let operation = LoadBalancerCreateRequest::new(name, load_balancer_type);
let mut target = [0_u8; 128];
let mut body = [0_u8; 512];
let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body))?;
assert_eq!(prepared.transport_request().target().as_str(), "/load_balancers");
# Ok::<(), Box<dyn core::error::Error>>(())
```

Request models that intentionally keep a resource endpoint separate from a
reusable query or action body use `HetznerPreparedOperation::query` or
`HetznerPreparedOperation::json`. Preparation verifies the operation-key pair
before writing bytes and rejects mismatches.

## Buffer Failure

Preparation clears the complete caller-owned target and body buffers before
writing. Any path, query, JSON, pairing, or capacity error clears both buffers
again and returns `HetznerPreparationError`; a partial request is never
returned as executable.

Sensitive fields such as user data, certificate private keys, TSIG keys,
Storage Box passwords, zonefiles, and record values use controlled JSON-string
writers. Their request model APIs do not expose raw secret getters for body
assembly.

## Firewall Resource Intent

`FirewallResourcesRequest::new(resources)` remains an apply-operation alias for
compatibility. Prefer `FirewallResourcesRequest::apply(resources)` for apply
actions and `FirewallResourcesRequest::remove(resources)` for destructive
removal so the body can only pair with the matching source-locked endpoint.

## Response Decoding

Prepared requests bind expected status, media type, body policy, and maximum
response size. Resource-specific typed success and provider-error decoding is
planned for v0.31; v0.30 does not claim that response-model coverage.
