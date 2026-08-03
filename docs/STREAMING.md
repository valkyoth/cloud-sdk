# Streaming Transport Contract

`cloud-sdk` provides allocation-free blocking, Send-async, and local-async
boundaries for bounded uploads, downloads, and caller-cancelled event streams.
The core owns policy and accounting, not networking, TLS, buffering, an
executor, retries, clocks, or cancellation scheduling.

## Complete Policy

Every attempt fixes a `StreamPolicy` before its first source observation:

| Field | Required decision |
| --- | --- |
| `StreamKind` | finite upload, finite download, or caller-cancelled event |
| `StreamFraming` | exact declared length or executor-owned unknown-length framing |
| `StreamSinkMode` | transactional hidden bytes or direct externally visible bytes |
| `max_bytes` | actual accepted-byte ceiling |
| `max_chunk_bytes` | per-source-chunk ceiling |
| `max_chunks` | source chunk count, including empty chunks |
| `max_observations` | source, sink, and `Wait` observations |
| `max_consecutive_zero_progress` | bounded empty, waiting, or zero-write streak |

Declared length cannot exceed the operation byte limit. Event streams require
executor-owned framing and cannot report normal finite completion; callers end
them by cancellation. Global ceilings prevent an operation from selecting
effectively unbounded counters.

## Backpressure And Progress

`StreamAttempt::begin_source_observation` and `begin_sink_observation` reserve
budget before external source or sink code is called. The matching
`begin_chunk`, `observe_wait`, `finish`, or `advance` call classifies that
observation. `begin_chunk` admits one complete source chunk but does not count
its bytes as transferred; `advance` records only bytes actually accepted by
the sink. A second source chunk is rejected until the pending chunk is fully
accepted. This is a strict no read-ahead contract: short writes cannot trigger
another source read or cause declared-length accounting to use offered instead
of transferred bytes.

Each chunk, sink acceptance, and explicit `StreamRead::Wait` consumes an
observation. Empty chunks, `Wait`, and zero-byte sink writes consume the
consecutive zero-progress budget. Alternating empty/data sources reset the
streak only when the sink accepts positive bytes. All arithmetic, operation
limits, declared-length overrun, and end-of-stream mismatch checks happen
before the violating state is accepted; a rejected attempt is terminal.

## Drivers

| Mode | Source | Sink | Driver |
| --- | --- | --- | --- |
| Blocking | `BlockingStreamSource` | `BlockingStreamSink` | `drive_blocking_stream` |
| Send async | `AsyncStreamSource` | `AsyncStreamSink` | `drive_async_stream` |
| Local async | `LocalAsyncStreamSource` | `LocalAsyncStreamSink` | `drive_local_stream` |

The caller supplies one nonempty scratch slice. Drivers limit the source view
to the policy chunk size, never allocate or retain a chunk, and volatile-clear
the complete scratch slice before first use and on success, error, or future
cancellation. The caller-owned outcome is reset before scratch validation, so
an empty-scratch error cannot preserve stale success from an earlier attempt.
Send source and sink futures are required to be `Send`; Send implementations
automatically satisfy the local traits.

Both async drivers force one self-waking cooperative yield after every 64
completed source or sink callbacks. An always-ready source and sink therefore
cannot process the complete global stream ceiling in one executor poll. This
adds no runtime, executor, timer, or allocation dependency and does not replace
caller-owned timeout and cancellation policy.

```rust
use cloud_sdk::transport::{
    StreamAttempt, StreamFraming, StreamKind, StreamLimits, StreamOutcome,
    StreamPolicy, StreamSinkMode,
};

let Ok(limits) = StreamLimits::new(8, 4, 2, 4, 1) else {
    return;
};
let Ok(policy) = StreamPolicy::new(
    StreamKind::FiniteDownload,
    StreamFraming::Declared(4),
    StreamSinkMode::Transactional,
    limits,
) else {
    return;
};
let mut outcome = StreamOutcome::new();
let mut attempt = StreamAttempt::new(policy, &mut outcome);
assert!(attempt.begin_source_observation().is_ok());
assert!(attempt.begin_chunk(4).is_ok());
assert!(attempt.begin_sink_observation().is_ok());
assert!(attempt.advance(2).is_ok());
assert!(attempt.begin_sink_observation().is_ok());
assert!(attempt.advance(2).is_ok());
assert!(attempt.begin_source_observation().is_ok());
let completion = attempt.finish();
assert!(completion.is_ok_and(|value| value.requires_sink_commit()));
assert!(attempt.commit_sink().is_ok());
drop(attempt);
assert_eq!(outcome.progress().bytes(), 4);
```

The manual accounting API is for transport adapters. Applications normally
use one driver with a source and sink implementation. `cloud-sdk-testkit`
provides `StreamFixtureSource` and `StreamFixtureSink` for deterministic
empty-chunk, chunk-boundary, short-write, commit, and rollback tests.

## Completion And Cancellation

End validation and sink commitment are separate. `StreamAttempt::finish`
checks pending bytes and declared length but leaves the outcome active.
`commit_sink` records `Complete` only after the sink's commit operation has
succeeded. Cancellation while an async commit is pending is therefore not
misreported as completion.

Dropping an active attempt records:

- `Clean` when no sink write was attempted;
- `RollbackRequired` when a transactional sink write was attempted; or
- `Dirty` when a direct sink write may have produced an external effect.

`StreamAttempt::begin_sink_observation` makes this sink-attempt state sticky in
the authoritative `StreamOutcome` before external sink code runs. The supplied
drivers also arm their abort guard at that boundary. A first-write error,
invalid or zero progress, or cancellation while the first sink future is
pending therefore cannot claim the attempt remained clean. Error or async
cancellation synchronously invokes the sink's abort method with that same
partial state.
Transactional sinks must discard every hidden partial byte;
`RollbackRequired` is a contract obligation, not a claim that arbitrary
external storage can be rolled back. Direct sinks remain dirty and require
caller reconciliation.

## Replay

Every source reports `StreamReplayability`. Non-replayable sources cannot be
retried. A replayable source supplies a bounded exact `StreamSourceId`, and
`validate_stream_replay` rejects a changed identity between attempts. The
source owner must change this identity whenever any stream byte changes. The
SDK compares exact identity bytes and does not use Rust's non-cryptographic
`Hash` machinery.

The streaming drivers execute one attempt and never retry. Mutations with
non-replayable bodies must remain single-attempt operations. A later retry
owner must validate the operation's retry/idempotency policy and exact source
identity before constructing another attempt.

## Event Streams

Long-lived events use `CallerCancelledEvent`, executor-owned framing, and hard
observation/zero-progress policy. `StreamRead::Wait` makes an observed turn
without a chunk countable. A source future that remains internally pending is
bounded by caller-owned cancellation, timeout, and executor policy; core owns
no clock or task scheduler. Event end is treated as unexpected failure rather
than successful finite completion.

## Security Boundaries

- Source and sink implementation errors must be payload-free; driver `Debug`
  and `Display` never format their payloads.
- Source identity is redacted from `Debug` and remains caller-owned.
- Caller source storage, committed destination storage, transport/TLS copies,
  operating-system buffers, and remote partial effects need separate lifecycle
  controls.
- Cancellation does not prove a remote mutation was not delivered. Delivery
  classification and provider reconciliation remain transport and operation
  policy.
- Executor-owned framing must not imply an unbounded body. Actual byte, chunk,
  observation, and zero-progress limits remain mandatory.
