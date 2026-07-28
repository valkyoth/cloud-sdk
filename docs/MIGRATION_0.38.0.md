# Migrating To v0.38

v0.38 makes volatile response cleanup a mandatory core property. Transports
may still add platform-specific cleanup, but they no longer own or establish
the baseline guarantee.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.38.0"
cloud-sdk-hetzner = "0.31.0"
cloud-sdk-reqwest = { version = "0.26.0", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.23.0"
```

`cloud-sdk-sanitization` no longer depends on `cloud-sdk`. The dependency now
points in the correct direction: `cloud-sdk` uses the neutral sanitization
boundary. Release automation publishes `cloud-sdk-sanitization` first.

## Response Buffers

The standard constructor no longer accepts a sanitizer:

```rust
use cloud_sdk::transport::ResponseBuffer;

let mut storage = [0_u8; 4_096];
let capacity = storage.len();
let response = ResponseBuffer::new(&mut storage, capacity);
drop(response);
assert_eq!(storage, [0_u8; 4_096]);
```

Use `ResponseBuffer::with_additive_sanitizer` only when the deployment has an
additional platform operation. Core clears before the hook and a drop guard
performs the final clear, including if the hook recontaminates storage or
unwinds.

## Operation Metadata

`OperationMetadata::new` now requires a fifth `RequestIdPolicy` argument:

```rust
use cloud_sdk::operation::{
    CostIntent, OperationImpact, OperationMetadata, RequestIdPolicy,
    RequestSemantics, RetryEligibility,
};

let metadata = OperationMetadata::new(
    OperationImpact::ReadOnly,
    RequestSemantics::Safe,
    RetryEligibility::ExplicitPolicy,
    CostIntent::NoKnownCost,
    RequestIdPolicy::Protected,
)?;
# Ok::<(), cloud_sdk::operation::OperationMetadataError>(())
```

- `Retain` permits an explicit transfer beyond the checked guard.
- `Protected` permits only guard-scoped inspection.
- `Discard` clears the identifier during policy admission.

`ResponsePolicy::validate` therefore also receives the operation's request-ID
policy. Prepared execution supplies it automatically.

## Retaining Request Identifiers

For a `Retain` operation, call `CheckedResponseGuard::retain_metadata` with an
explicit byte limit. The source is cleared immediately on successful or
rejected transfer. The returned `RetainedResponseMetadata` is neither `Copy`
nor `Clone`, provides only closure-scoped access, and clears its fixed storage
on drop.

`ResponseMetadata`, `ResponseHeaders`, and `ResponseContentType` are also no
longer implicitly copyable. Transport tests that deliberately need a second
metadata owner must call the explicit bounded `retain_copy` methods.

## Decoder Workspace

`CheckedResponseGuard::decode_owned_with_workspace` lends
`ResponseDecodeWorkspace` to provider decoders. It contains fixed decoder,
cursor, and provider-link staging arrays. The Hetzner direct parser now uses
this scratch rather than independent ordinary local storage.

The existing `decode_owned` method remains available when no workspace access
is needed. Both methods drop the complete body, metadata, identifier, and
scratch owner before returning the owned result.

## Guarantee Limits

The mandatory primitive uses volatile writes through the separately reviewed
`sanitization` crate. Tests proving a zero read-back verify state integrity;
they do not prove that an optional platform hook ran.

Cleanup applies to SDK-owned and caller-lent storage on ordinary returns,
errors, cancellation drop, and unwind where unwind is supported. It cannot
cover process abort, `mem::forget` or deliberately leaked guards, immutable or
external copies, TLS/allocator/kernel/device buffers, swap, crash dumps, or
remote systems.
