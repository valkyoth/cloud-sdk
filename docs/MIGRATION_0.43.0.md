# Migrating To v0.43

v0.43 completes the authenticated raw-wire migration for every active Hetzner
operation. Prepared execution now carries all policy needed by the bounded raw
HTTP engine and cannot fall back to a legacy transport path.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.43.0"
cloud-sdk-hetzner = "0.33.0"
cloud-sdk-reqwest = { version = "0.30.0", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.25.0"
```

`cloud-sdk-sanitization` is unchanged and is not published for this release.

## Prepared Requests

`PreparedRequest::new` now requires both an `AuthenticationScopePolicy` and a
`RawResponsePolicy` and returns `Result`. The provider owns these values;
applications preparing custom operations must supply complete policies
explicitly. A `Protected` or `Retain` request-ID lifecycle is rejected unless
the raw policy admits `x-request-id`, preventing metadata policy from referring
to a header that transport would discard.

`PreparedRequest::execute_blocking` and `execute_async` now require
`BlockingAuthenticatedTransport` and `AsyncAuthenticatedTransport`
respectively. They no longer accept the credential-free legacy transport
traits. `PreparedRequest::authenticated_request` exposes the exact request and
both wire policies for adapters that need explicit execution.

All 208 active Hetzner operations bind:

- the exact Cloud, DNS, Security, or Storage service identity;
- the exact official endpoint and required provider/service/endpoint scope;
- forbidden audience, account, and tenant scope;
- independent success and error body bounds;
- required JSON media or forbidden no-content media;
- admitted `content-type`, `x-request-id`, and complete `ratelimit-*` response
  metadata plus an informational-response limit.

## Authenticated Requests

`AuthenticatedRequest::new` now takes a third argument:

```rust
let authenticated = AuthenticatedRequest::new(
    transport_request,
    authentication_policy,
    raw_response_policy,
);
```

This is a deliberate source break. An authenticated adapter must not infer
response limits, media policy, admitted headers, or informational handling.

## Reqwest Adapter

Bearer and Basic clients now execute through the same bounded raw Hyper HTTP/1
engine as `RawBlockingClient` and `RawAsyncClient`. Authorization remains
transport-owned and is inserted only after complete scope validation.

Authenticated failures now preserve `NotSent`, `PossiblySent`, or
`ResponseStarted` in `AuthenticatedTransportFailure`. Callers must use the
delivery phase together with operation retry metadata; mutations must not be
retried from an ambiguous phase without an explicit policy.

The transport retains only operation-admitted response headers. Provider quota
decoding is intentionally not performed in the neutral adapter and remains a
later provider-policy milestone. Hetzner policies retain all three quota fields
so complete and incomplete sets remain available for strict provider decoding;
duplicate fields are rejected at the raw HTTP boundary.

The async raw engine rechecks informational rejection after the final response
future becomes ready. A final response therefore cannot win a multithreaded
runtime race against `101` or informational-count rejection.

## Testkit

`MockTransport` implements both authenticated transport traits.
`PreparedRequestRecord` exposes authentication and raw response policies so
fixtures can assert the complete wire contract without copying target, body,
or credential values.

See [`RAW_HTTP_EXECUTOR.md`](RAW_HTTP_EXECUTOR.md),
[`AUTHENTICATION_POLICY.md`](AUTHENTICATION_POLICY.md), and
[`HETZNER_EXAMPLES.md`](HETZNER_EXAMPLES.md).
