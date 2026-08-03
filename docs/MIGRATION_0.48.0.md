# Migrating To v0.48

v0.48 adds opt-in provider-neutral streaming contracts. Existing buffered
requests, transports, prepared operations, and provider behavior are
unchanged.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.48.0"
cloud-sdk-hetzner = "0.36.2"
cloud-sdk-reqwest = "0.32.1"
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.28.0"
```

`cloud-sdk-sanitization` is unchanged and is not published. Hetzner and
reqwest receive dependency-only patches. Testkit adds streaming fixtures.

## New Streaming Boundary

Buffered `TransportRequest` bodies and `ResponseBuffer` execution do not
change. Large or incremental I/O may instead use:

- `BlockingStreamSource` and `BlockingStreamSink`;
- `AsyncStreamSource` and `AsyncStreamSink`; or
- `LocalAsyncStreamSource` and `LocalAsyncStreamSink`.

Use the matching `drive_*_stream` function. Supply nonempty caller-owned
scratch storage and one `StreamOutcome`. The complete scratch slice is cleared
before use and on every exit. Drivers reset the outcome before validating
scratch, so reusing a previously complete outcome cannot leave stale success
after an empty-scratch error.

Send and local async drivers force a cooperative yield after 64 completed
source or sink callbacks. This is internal and needs no executor-specific API,
but callers must still provide timeout and cancellation policy.

Every source must report `StreamReplayability`. Default to `NotReplayable`.
Use `Replayable(StreamSourceId)` only when the source owner can reproduce exact
bytes and changes the identity whenever content changes.

## Policy Migration

There is no default stream policy. Construct `StreamLimits`, then select
`StreamKind`, `StreamFraming`, and `StreamSinkMode`. Declared and unknown
lengths remain distinct. Unknown length delegates HTTP framing to the executor,
not byte, chunk, observation, or progress bounds.

Transactional sinks must hide partial data and remove it from `abort` when the
driver reports `RollbackRequired`. Direct sinks may expose partial data and
must treat `Dirty` as requiring reconciliation. Calling
`begin_sink_observation` makes this classification sticky before sink code
runs, even if no accepted byte can later be recorded. Sink commit can be
async; cancellation while commit is pending aborts and never records
completion.

## Retry Migration

Streaming drivers never retry. Call `validate_stream_replay` before a later
attempt and retain the existing operation retry, idempotency, delivery-phase,
and mutation policy. A non-replayable mutation is never automatically retried.

See [`STREAMING.md`](STREAMING.md) for the complete contract and examples.
