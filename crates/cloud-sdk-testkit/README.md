<p align="center">
  <b>provider-neutral no_std testkit for cloud-sdk.</b><br>
  Deterministic mock transport, prepared-request records, bounded response fixtures, and adversarial corpora.
</p>

<div align="center">
  <a href="https://crates.io/crates/cloud-sdk">cloud-sdk crate</a>
  |
  <a href="https://docs.rs/cloud-sdk-testkit">Docs.rs</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/docs/threat-model.md">Threat Model</a>
  |
  <a href="https://github.com/valkyoth/cloud-sdk/blob/main/SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <a href="https://github.com/valkyoth/cloud-sdk">
    <img src="https://raw.githubusercontent.com/valkyoth/cloud-sdk/main/.github/images/cloud-sdk.webp" alt="cloud-sdk Rust crate overview">
  </a>
</p>

# cloud-sdk-testkit

Provider-neutral testing support for the main
[`cloud-sdk`](https://crates.io/crates/cloud-sdk) crate and its provider crates.
The default graph is no_std, allocation-free, network-free, filesystem-free,
and runtime-free.

Static ordered exchanges are the simplest choice for fixed request sequences.
Use bounded dynamic scenarios when a response must depend on the current
request or when pagination, polling, cancellation, partial I/O, or injected
failures need deterministic multi-request coverage.

## Install

```toml
[dev-dependencies]
cloud-sdk = "0.96.0"
cloud-sdk-testkit = "0.31.0"
```

## Mock Transport

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingTransport, RequestTarget, ResponseBuffer, TransportRequest,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

let Ok(target) = RequestTarget::new("/resources?page=1") else {
    return;
};
let Ok(body) = FixtureBody::new(br#"{"resources":[]}"#) else {
    return;
};
let exchanges = [MockExchange::new(
    ExpectedRequest::new(Method::Get, target),
    ResponseFixture::success(body),
)];
let transport = MockTransport::new(&exchanges);
let mut output = [0_u8; 64];
let output_capacity = output.len();
let mut response_headers = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
let mut response =
    ResponseBuffer::new(&mut output, output_capacity, &mut response_headers);

if transport
    .send(
        TransportRequest::new(Method::Get, target),
        response.writer(),
    )
    .is_err()
{
    return;
}

assert!(response
    .with_response(|view| {
        view.status().get() == 200
            && view.body() == br#"{"resources":[]}"#
    })
    .is_ok_and(core::convert::identity));
assert!(transport.is_complete());
```

The same mock implements the executor-neutral async contract without adding a
runtime dependency:

```rust,no_run
# async fn example() {
use cloud_sdk::Method;
use cloud_sdk::transport::{
    RequestTarget, ResponseBuffer, TransportRequest, drive_async,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

let Ok(target) = RequestTarget::new("/resources/42") else { return };
let Ok(body) = FixtureBody::new(br#"{"id":42}"#) else { return };
let exchanges = [MockExchange::new(
    ExpectedRequest::new(Method::Get, target),
    ResponseFixture::success(body),
)];
let transport = MockTransport::new(&exchanges);
let mut output = [0_u8; 32];
let output_capacity = output.len();
let mut response_headers = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
let mut response =
    ResponseBuffer::new(&mut output, output_capacity, &mut response_headers);
if drive_async(
    &transport,
    TransportRequest::new(Method::Get, target),
    response.writer(),
)
.await
.is_err()
{
    return;
}

assert!(response
    .with_response(|view| view.body() == br#"{"id":42}"#)
    .is_ok_and(core::convert::identity));
# }
# fn main() {}
```

`LocalMockTransport` is deliberately `!Sync` and exercises the local async
basic and authenticated contracts without a runtime. Use `drive_local` or
`PreparedRequest::execute_local_async` to compile-check browser, embedded, and
single-threaded workflows. Existing `MockTransport` automatically satisfies
the local contract through its transaction-wrapped `Send` async implementation.

## Raw Delivery Faults

`RawFaultExecutor` injects a deterministic `NotSent`, `PossiblySent`,
`ResponseStarted`, or unknown-delivery failure into both raw executor traits.
Unknown delivery deliberately becomes `PossiblySent`:

```rust,no_run
use cloud_sdk_testkit::{RawFault, RawFaultExecutor};

let executor = RawFaultExecutor::new(RawFault::Unknown);
assert_eq!(executor, RawFaultExecutor::new(RawFault::Unknown));
```

Each exchange is consumed only after method, target, ordered headers, body,
and complete response capacity match. Failures are distinct and payload-free.
Debug output redacts request targets, header values, request bodies, and
response bodies.

## Streaming Fixtures

`StreamFixtureSource` preserves ordered chunk boundaries, including explicit
empty chunks. `StreamFixtureSink` uses caller storage and a deterministic
maximum write size to exercise short writes and backpressure:

```rust
use cloud_sdk::transport::{
    StreamFraming, StreamKind, StreamLimits, StreamOutcome, StreamPolicy,
    StreamSinkMode, drive_blocking_stream,
};
use cloud_sdk_testkit::{StreamFixtureSink, StreamFixtureSource};

let chunks: &[&[u8]] = &[b"ab", b"", b"cde"];
let Ok(mut source) = StreamFixtureSource::new(chunks) else { return };
let mut output = [0_u8; 5];
let Ok(mut sink) = StreamFixtureSink::new(&mut output, 2) else { return };
let Ok(limits) = StreamLimits::new(5, 3, 3, 7, 1) else { return };
let Ok(policy) = StreamPolicy::new(
    StreamKind::FiniteDownload,
    StreamFraming::Declared(5),
    StreamSinkMode::Transactional,
    limits,
) else { return };
let mut scratch = [0_u8; 3];
let mut outcome = StreamOutcome::new();

assert!(drive_blocking_stream(
    policy,
    &mut source,
    &mut sink,
    &mut scratch,
    &mut outcome,
).is_ok());
assert_eq!(sink.bytes(), b"abcde");
assert_eq!(sink.writes(), 3);
```

The same fixtures implement the Send async contracts and therefore the local
async contracts. Sources are non-replayable by default; use
`StreamFixtureSource::with_replayability` only with an exact
`StreamSourceId`. See the main
[`streaming contract`](https://github.com/valkyoth/cloud-sdk/blob/main/docs/STREAMING.md).

Use `with_fault_at_observation` and `with_fault_at_write` for exact one-based
I/O failures. A validated `StreamPatternSource` with `EndlessEmpty` models an
endless zero-progress peer, while `AlternatingEmptyData` requires a nonempty
borrowed chunk and alternates it with explicit empty chunks. Pattern sources
never emit `StreamRead::End`; the core stream policy must stop them through its
byte, chunk, observation, or zero-progress bounds.

## Prepared Request Assertions

Bind `MockTransport` with `with_endpoint` before executing a
`PreparedRequest`. Endpoint mismatches fail before an exchange is consumed.
`ExpectedRequest::with_headers` checks the exact ordered request-header block.
`ResponseFixture::with_headers` adds complete bounded raw metadata, while
`with_content_type` models missing, accepted, unexpected, or malformed typed
content metadata.

Core clears the complete caller buffer independently of the mock, so prepared
tests can assert cleanup even when endpoint or fixture validation fails before
a response is returned. The mock's sanitizer implementation remains available
only for tests that deliberately exercise the additive hook.

`PreparedRequestRecord::capture` records method, redacted target/body lengths,
provider service and endpoint policy, complete operation metadata, checked
response policy, authentication scope, raw response policy, and explicit body
replayability without copying request values. Tests can therefore assert both
safety classification and the complete authenticated wire contract.

## Fixture Builders

`ResponseFixture` builds deterministic success, paginated, action, rate-limit,
and error responses. `PaginationFixture`, `ActionFixture`, and
`RateLimitFixture` reject incoherent metadata before a fixture can be used.
Use `ResponseFixture::with_rate_limit` and `with_content_type` to attach
transport metadata to paginated, action, success, or error responses.

`FixtureBody` supports borrowed bytes and compact repeated-byte bodies up to
8 MiB plus one byte. Writes preflight capacity and leave undersized destination
buffers unchanged.

## Dynamic Scenarios

`DynamicMockTransport` invokes a borrowed `ProviderFixtureBuilder` for each
request and records only successful steps. `DynamicResponder` adapts a closure;
provider crates can implement the trait directly for reusable fixture builders.
Selection failures, undersized response storage, and cancellation do not
consume a sequence number or create a record.

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingTransport, RequestTarget, ResponseBuffer, TransportRequest,
};
use cloud_sdk_testkit::{
    DynamicMockTransport, DynamicRequest, DynamicResponder, FixtureBody,
    RequestRecordSlot, ResponseFixture,
};

let Ok(target) = RequestTarget::new("/resources") else { return };
let Ok(body) = FixtureBody::new(br#"{"resources":[]}"#) else { return };
let fixture = ResponseFixture::success(body);
let responder = DynamicResponder::new(|request: DynamicRequest<'_>| {
    if request.method() == Method::Get && request.target() == target {
        Ok(&fixture)
    } else {
        Err(())
    }
});
let records = [const { RequestRecordSlot::new() }; 2];
let Ok(transport) = DynamicMockTransport::new(responder, &records) else {
    return;
};
let mut output = [0_u8; 32];
let output_capacity = output.len();
let mut headers = [0_u8; 64];
let mut response = ResponseBuffer::new(&mut output, output_capacity, &mut headers);

assert!(transport
    .send(
        TransportRequest::new(Method::Get, target),
        response.writer(),
    )
    .is_ok());
assert_eq!(transport.recorded(), 1);
let Some(record) = transport.record(0) else { return };
assert_eq!(record.body_len(), 0);
assert_eq!(record.status().get(), 200);
```

`PaginationScript` requires page one through the declared last page with stable
page size and totals. `ActionScript` requires nondecreasing progress, running
intermediate steps, and exactly one final success or error. Both are finite,
bounded to `MAX_DYNAMIC_RECORDS`, and implement `ProviderFixtureBuilder`.

`RequestRecordSlot` uses caller-owned atomic storage. Its public observation
contains a finite method class, encoded target length, body length, header
count, response status, and sequence number. It never retains target bytes,
header names or values, request bodies, response bodies, or extension-method
tokens.

## Adversarial Corpus

`adversarial_corpus()` returns reusable cases for:

- malformed JSON;
- additive unknown fields;
- missing required fields;
- an oversized response represented without an 8 MiB static allocation;
- invalid pagination metadata;
- an invalid action state and progress value.

Provider crates consume applicable cases in their own parser tests. The
Hetzner Serde boundary exercises this corpus without making the testkit depend
on `cloud-sdk-hetzner`.

## Features

| Feature | Default | Effect |
| --- | --- | --- |
| `default` | yes | Empty; keeps the testkit allocation-free, runtime-free, and `no_std`. |
| `alloc` | no | Enables allocation-bearing test helpers and `cloud-sdk/alloc`. |
| `std` | no | Enables `alloc` and standard-library integration without selecting a runtime. |

Docs.rs builds with all features. The mock transport remains network-free in
every configuration.

## Security Notes

This crate is test infrastructure, not a production transport. Core
secret-capable header types do not expose ordinary equality. The mock uses a
private exact byte matcher solely for deterministic expectations; it must not
be exposed as a remote secret comparison oracle. Authentication, base URLs,
headers, timeout policy, TLS, retry behavior, and secret ownership remain
responsibilities of concrete transport adapters.

The testkit stores only borrowed expectations and fixture bodies. Dynamic
responders execute synchronously during a transport poll and must remain
deterministic, bounded, and free of blocking I/O. Callers must keep borrowed
data alive and must still sanitize secret-bearing test buffers when their
threat model requires it.
