# Compile-Time Hetzner Operation Associations

v0.50 adds one sealed marker for each of the 208 active source-locked Hetzner
operations. The association layer is allocation-free, transport-free,
available without features, and additive to the existing runtime-checked
`PrepareOperation` API.

## Purpose

The existing prepared API rejects a mismatched endpoint, query, or body at
runtime before writing request bytes. The association API additionally gives
all components the same nominal operation parameter `O`, so components bound
to different operations cannot be combined by safe Rust.

Each `HetznerOperation` marker owns these associated policies:

- provider service, official endpoint family, authentication class, and scope;
- HTTP method, query presence, body shape, request headers, and request media;
- success status, success/error body and media policy, and response caps;
- pagination, quota, retry, and buffered-streaming policy;
- success and provider-error response families; and
- required no-op, mutation, destructive, or cost permit class.

The inspectable `OperationDescriptor` exposes the runtime values for logging,
policy review, and future client-kernel integration without exposing payloads.

## Read Example

```rust
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::actions::{ActionEndpoint, ActionId};
use cloud_sdk_hetzner::association::AssociatedOperation;
use cloud_sdk_hetzner::association::operations::GetAction;

let id = ActionId::new(7).ok_or("invalid action ID")?;
let operation = AssociatedOperation::<GetAction, _>::endpoint(
    ActionEndpoint::Get(id),
)?;
let mut target = [0_u8; 64];
let mut body = [0_u8; 1];
let prepared = operation.prepare_typed(PreparationStorage::new(
    &mut target,
    &mut body,
))?;

assert_eq!(prepared.association().operation_id().as_str(), "get_action");
assert_eq!(
    prepared.as_untyped().transport_request().target().as_str(),
    "/actions/7",
);
# Ok::<(), Box<dyn core::error::Error>>(())
```

## Query And Body Construction

Use `AssociatedOperation::query` for an operation with a concrete query and
`AssociatedOperation::json` for an operation with a JSON request body.
Optional-query operations may use `AssociatedOperation::endpoint` when the
query is absent. Required queries and bodies fail closed before preparation.

The individual `EndpointFor<O, E>`, `QueryFor<O, Q>`, and `BodyFor<O, B>`
wrappers support independent validation followed by `from_parts`. Their common
`O` parameter prevents cross-operation assembly. Concrete component traits are
sealed to provider-owned wire types, so downstream crates cannot forge a
component that merely reports a chosen operation string.

## Prepared Requests

`prepare_typed` first uses the existing transactional fixed-buffer encoder,
then checks the complete prepared request against `O::DESCRIPTOR`. This check
includes provider/service identity, official endpoint policy, authentication
scope, method, query/body presence, request headers, retry and impact metadata,
permit class, success status, response media, and body limits.

`Prepared<O>` delegates checked response validation and blocking, Send-async,
and local-async authenticated execution. `as_untyped` borrows the underlying
provider-neutral request. `into_untyped` is a deliberately explicit operation
type erasure.

This release associates response model families but does not add the future
high-level client decoder. Typed resource decoding remains on the roadmap.

## Source Lock

`scripts/generate_operation_associations.py` reconstructs the registry from:

- `docs/API_FINGERPRINTS.tsv` for active operation, method, service, and
  pagination facts;
- `docs/PREPARED_BODY_OPERATIONS.txt` for request-body presence; and
- `crates/cloud-sdk-hetzner/src/serde/response_operations.tsv` for success
  status and response family.

Generation requires exact 208-operation response coverage, exact 91-operation
body coverage, globally unique operation IDs, and no inactive body binding.
`scripts/generate_operation_associations.py --check` runs in normal and
release checks. The compact generated registry is intentionally excluded from
rustfmt expansion so it remains below the repository 500-line limit; rustfmt
still formats the handwritten generator macro and every other Rust file.

Permit classification is provider-reviewed metadata and is rechecked against
the existing prepared operation metadata whenever a typed operation is
prepared. A disagreement fails closed with `PreparedPolicyMismatch`.
