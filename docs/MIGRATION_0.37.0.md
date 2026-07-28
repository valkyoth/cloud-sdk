# Migrating To v0.37

v0.37 replaces transport-created response values with a sealed writer tied to
caller-owned storage. Checked responses now retain the cleanup owner until
decoding or inspection finishes.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.37.0"
cloud-sdk-hetzner = "0.30.0"
cloud-sdk-reqwest = { version = "0.25.0", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.15.5"
cloud-sdk-testkit = "0.22.0"
```

## Transport Implementations

`BlockingTransport::send` and `AsyncTransport::send` no longer return
`TransportResponse`. They receive a sealed `ResponseWriter`, initialize only
its admitted prefix, and commit once:

```rust
use cloud_sdk::transport::{
    BlockingTransport, ResponseMetadata, ResponseWriter, ResponseWriterError,
    StatusCode, TransportRequest,
};

struct Transport;

impl BlockingTransport for Transport {
    type Error = ResponseWriterError;

    fn send(
        &self,
        _request: TransportRequest<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        let body = response.body_mut()?;
        body.get_mut(..2)
            .ok_or(ResponseWriterError::InitializedLengthTooLarge)?
            .copy_from_slice(b"{}");
        response.commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
    }
}
```

Safe external code cannot construct `ResponseWriter` or `TransportResponse`,
replace the response body with static or unrelated bytes, commit beyond the
admitted capacity, commit twice, or mutate through the writer after commit.
Returning `Ok(())` without committing fails closed during response validation.
Adapters reject a precommitted writer before network access; the testkit also
leaves its exchange cursor unchanged.

## Direct Transport Calls

Create `ResponseBuffer` with the complete caller storage, operation limit, and
a `ResponseStorageSanitizer`. Lend only `response.writer()` to transport:

```rust
# use cloud_sdk::Method;
# use cloud_sdk::transport::{
#     BlockingTransport, RequestTarget, ResponseBuffer,
#     ResponseStorageSanitizer, TransportRequest,
# };
# fn send<T>(
#     transport: &T,
# ) -> Result<(), T::Error>
# where
#     T: BlockingTransport + ResponseStorageSanitizer,
# {
let target = RequestTarget::new("/health").expect("static target is valid");
let request = TransportRequest::new(Method::Get, target);
let mut storage = [0_u8; 4_096];
let capacity = storage.len();
let mut response = ResponseBuffer::new(&mut storage, capacity, transport);
transport.send(request, response.writer())?;
response
    .with_response(|view| {
        assert!(view.status().is_success());
    })
    .expect("transport committed its response");
# Ok(())
# }
```

`ResponseBuffer::with_response` uses a higher-ranked closure, so response bytes
cannot escape. The complete original storage is sanitized before admission and
again when the buffer or later checked guard drops, including bytes outside the
operation-admitted prefix.

## Prepared Execution

`PreparedRequest::execute_blocking` and `execute_async` now return
`CheckedResponseGuard`. Use:

- `with_borrowed` for closure-scoped zero-copy inspection;
- `decode_owned` for owned decoding that drops and sanitizes the complete
  response storage before returning the owned result.

The guard is also dropped on endpoint rejection, transport error, response
policy rejection, decode error, early return, and ordinary async cancellation.
The sanitizer remains a separate explicit trait; production implementations
must use a non-elidable primitive.

## Hetzner Decoding

`cloud_sdk_hetzner::serde::decode_response` now consumes `ResponseBuffer`
instead of `TransportResponse`. Both typed success and provider-error paths
drop the buffer before returning owned models. Invalid writer state is reported
as `HetznerDecodeError::ResponseWriter`.

## Cleanup Scope

v0.37 proves caller-buffer provenance and makes cleanup ownership structural.
It does not claim that ordinary test-only `fill(0)` implementations are
non-elidable, nor that allocator, TLS, kernel, device, crash-dump, process
abort, or deliberately leaked storage is cleared. v0.38 owns the audited
non-elidable core cleanup primitive and cleanup-owning transfer of retained
sensitive metadata.
