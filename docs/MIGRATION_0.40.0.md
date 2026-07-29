# Migrating To v0.40

v0.40 adds a credential-free raw HTTP executor beneath future authentication
and typed client policy.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.40.0"
cloud-sdk-hetzner = "0.32.1"
cloud-sdk-reqwest = { version = "0.27.0", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.24.0"
```

## Raw Execution

`BlockingRawHttpExecutor` and `AsyncRawHttpExecutor` execute one complete
validated `TransportRequest` through caller-owned `ResponseWriter` storage.
`RawResponsePolicy` defines independent success/error body limits and media
rules, admitted response headers, and an informational-response limit.
Trailers and `101 Switching Protocols` are always rejected.

`cloud-sdk-reqwest` provides `RawBlockingClientBuilder` and
`RawAsyncClientBuilder`. These builders take no bearer token. They do not add
authorization, JSON `Accept`, redirects, proxies, decompression, or retries.
The previous authenticated `BlockingClient` and `AsyncClient` remain available
while authentication policy moves into the v0.41-v0.42 layers.

## Delivery Failures

Raw adapter failures are `TransportFailure<RawHttpError>`. Check
`DeliveryPhase` before making any retry decision:

- `NotSent` proves failure before request delivery.
- `PossiblySent` means request bytes may have reached the peer.
- `ResponseStarted` means a response head was observed.

Unknown state is deliberately represented as `PossiblySent`. v0.40 does not
provide automatic retry or mutation reconciliation.

## Header Ownership

`user-agent` joins authorization, authority, framing, proxy, and upgrade fields
as a transport-owned request header. Constructing it through `RequestHeader`
now returns `HeaderError::ReservedRequestHeader`; supply it once through the
transport builder.

Unknown response headers are dropped. Operation policy must explicitly admit
each retained field, while credential, cookie, framing, proxy, and upgrade
fields cannot be admitted.

## Blocking Runtime

The raw blocking adapter drives the shared Hyper engine on a private
current-thread Tokio runtime for each call and disables idle connection
pooling. Do not invoke the blocking API from an async runtime thread; use
`RawAsyncClient` there.

Complete bounds and process-allocation exclusions are documented in
[`RAW_HTTP_EXECUTOR.md`](RAW_HTTP_EXECUTOR.md).
