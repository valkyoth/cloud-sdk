# Migrating To v0.28.0

The workspace facade moves to `0.28.0`. `cloud-sdk-reqwest` moves to `0.19.0`
and `cloud-sdk-testkit` moves to `0.17.0`; the Hetzner provider receives a
code release at `0.22.0`.

## Transport Receivers

`BlockingTransport::send` and `AsyncTransport::send` now receive `&self`:

```rust,ignore
let response = transport.send(request, &mut response_body)?;
let response = AsyncTransport::send(&transport, request, &mut response_body).await?;
```

Custom transport implementations must change their receiver from `&mut self`
to `&self`. Sequential state can use safe interior mutability. A transport that
supports overlapping requests must also provide the `Sync`, `Send`, executor,
and task-lifetime properties required by its caller. The SDK does not add a
mutex, concurrency limit, task set, queue, retry, sleep, or runtime.

## Endpoint Identity

Credential-bound transports can implement `BoundTransport`. The reqwest
clients expose their immutable normalized `EndpointIdentity`, containing the
scheme, host, effective port, and base path. Provider code should compare all
four fields with official constants before execution. A custom endpoint remains
an explicit credential destination and must never come from tenant input.

`cloud-sdk-hetzner::verify_official_endpoint` performs this exact comparison
for `ApiBaseUrl::CloudV1` and `ApiBaseUrl::HetznerV1`. It accepts any
provider-neutral `BoundTransport` and therefore does not couple the provider
crate to reqwest.

## Token Ingestion And Rotation

`BearerToken::new(&str)` remains available for compatibility, but cannot clear
the immutable source. Prefer:

- `BearerToken::from_mut_bytes`, which clears the complete source on success or
  rejection;
- `BearerToken::from_secret_buffer`, which consumes and drops a
  `cloud_sdk_sanitization::SecretBuffer`;
- `rotate_bearer_token_from_mut_bytes` or
  `rotate_bearer_token_from_secret_buffer` on a built client.

Blocking and async clients are now cloneable shared handles. Rotation is atomic
for newly started requests and never holds the credential lock across network
I/O or `.await`. Rejected replacement input leaves the active token unchanged.
An in-flight request retains its previous snapshot; revoke the old provider
credential after the application's bounded in-flight window closes.

The internal credential lock recovers from poisoning while its guard is held.
Because the protected value is always one complete `Arc<BearerToken>`, recovery
cannot expose partially initialized credential state or permanently disable
every cloned client.

## Testkit

`MockTransport` now accepts shared references and uses an atomic ordered cursor.
Its new `MockError::ConcurrentRequest` variant reports overlapping exchanges
that race for the same ordered expectation. Keep one distinct response buffer
per request.
