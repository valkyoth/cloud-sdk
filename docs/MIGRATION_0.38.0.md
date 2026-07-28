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

let mut body_storage = [0_u8; 4_096];
let mut header_storage =
    [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
let capacity = body_storage.len();
let response =
    ResponseBuffer::new(&mut body_storage, capacity, &mut header_storage);
drop(response);
assert_eq!(body_storage, [0_u8; 4_096]);
assert_eq!(
    header_storage,
    [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES],
);
```

The separate header destination is mandatory. Sensitive header bytes remain at
a stable caller-owned address for the complete response lifecycle rather than
moving inside a by-value metadata object. `PreparedRequest::execute_blocking`
and `execute_async` likewise require body and header destinations.

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
policy. Prepared execution supplies it automatically. Provider decoders must
call `PreparedRequest::apply_response_metadata_policy` before decoding an
error-status response that bypasses the success policy; the Hetzner decoder
does so for every provider error.

## Retaining Request Identifiers

For a `Retain` operation, call
`CheckedResponseGuard::retain_metadata_into` with caller-owned destination
storage and an explicit byte limit. The request ID copies directly from its
stable header destination into the stable retention destination, then the
source clears immediately on successful or rejected transfer. The returned
`RetainedResponseMetadata<'_>` is neither `Copy` nor `Clone`, provides only
closure-scoped access, and clears the complete destination on drop.

`ResponseHeaders` is no longer implicitly copyable. `ResponseMetadata`
contains only interpreted non-sensitive scalar values. Transports populate
headers through `ResponseWriter::headers_mut`; `ResponseContentType<'_>` is a
borrowed validated view over those stable bytes. Tests that deliberately need
a second header owner must supply another caller buffer to
`ResponseHeaders::retain_copy_into`.

`TransportResponse::content_type` now returns
`Result<Option<ResponseContentType<'_>>, ContentTypeError>`. `Ok(None)` means
the header is absent; a present invalid UTF-8 or malformed value returns
`Err`. Custom transports only capture raw bounded headers. Core
`ResponsePolicy` performs this validation and rejects malformed values with
`ResponsePolicyError::InvalidContentType` under every content-type policy,
including `Optional` and `Forbidden`.

The strict Hetzner JSON parser now also protects object-key allocations.
Unknown field names are wiped on drop just like string values; this matters
because extension keys can contain tenant-controlled text even when ignored.

## Decoder Workspace

`CheckedResponseGuard::decode_owned_with_workspace` lends
`ResponseDecodeWorkspace` to provider decoders. It contains fixed decoder,
cursor, and provider-link staging arrays. The Hetzner direct parser now uses
this scratch rather than independent ordinary local storage.

The cursor and provider-link regions reserve the cleanup contract for the
v0.44 bounded pagination strategy family. They are not continuation parsers in
v0.38 and carry no length or validity state yet.

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
