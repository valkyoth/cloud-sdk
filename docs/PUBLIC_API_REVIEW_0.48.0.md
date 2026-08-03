# v0.48.0 Public API Review

Date: 2026-08-03

Scope: allocation-free provider-neutral streaming policy, accounting, I/O
traits, drivers, replay identity, cancellation, and testkit fixtures.

## Added API

`transport` exports complete stream policy domains: `StreamKind`,
`StreamFraming`, `StreamSinkMode`, `StreamLimits`, `StreamPolicy`, their hard
ceilings, and payload-free validation errors.

`StreamAttempt` owns byte, chunk, observation, zero-progress, declared-length,
and pending-chunk accounting over caller-owned `StreamOutcome`. It exposes
deterministic preflight with `begin_source_observation` and
`begin_sink_observation`, source-result classification with `begin_chunk`,
actual-byte `advance`, bounded `observe_wait`, end validation, and post-sink
`commit_sink`. `StreamProgress`, `StreamState`, and `StreamPartialState` expose
only nonsensitive counters and lifecycle classes.

Blocking, Send-async, and local-async source/sink traits use caller-owned
scratch storage. `drive_blocking_stream`, `drive_async_stream`, and
`drive_local_stream` perform one bounded attempt, clear complete scratch,
enforce backpressure, commit after end validation, and abort on every ordinary
error or async cancellation. Send traits require Send futures and the Send
driver itself is compile-checked as Send.

`StreamReplayability`, redacted exact `StreamSourceId`, and
`validate_stream_replay` make changed or non-replayable sources explicit.
Testkit adds bounded borrowed `StreamFixtureSource` and caller-buffered
`StreamFixtureSink` with deterministic empty chunks and short writes.

`buffer::sanitize_bytes` re-exports the workspace's audited volatile-clear
boundary. This lets provider-neutral companion crates clear caller-owned
stream storage through `cloud-sdk` without adding another direct dependency.

## Compatibility

All APIs are additions in a new opt-in module path. Buffered transport,
authentication, prepared request, response, pagination, quota, retry, and
provider APIs retain their signatures and behavior. No default or optional
Cargo feature changes.

Cross-thread sources and sinks automatically satisfy their local traits. The
reverse conversion is intentionally unavailable. The reqwest adapter does not
claim streaming implementation in this release; it receives only the core
dependency patch.

## Security Review Focus

- no unbounded or zero hard limits;
- actual bytes rather than offered chunk lengths;
- no next source chunk while sink bytes remain pending;
- terminal failure after any invalid accounting transition;
- distinct declared and executor-framed unknown length;
- bounded empty, waiting, and zero-write progress;
- exact replay identity without non-cryptographic hashing;
- no automatic retry;
- complete scratch cleanup on success, error, and cancellation;
- explicit transactional rollback versus dirty direct state; and
- cancellation during source read, sink write, and sink commit.

No API claims that direct remote effects can be rolled back, that cancellation
means `NotSent`, or that event futures internally pending without yielding a
`Wait` observation are bounded without caller cancellation.
