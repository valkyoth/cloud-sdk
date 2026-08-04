# Dynamic Testkit Scenarios

`cloud-sdk-testkit` provides allocation-free dynamic scenarios for workflows
whose response depends on the current request or successful request sequence.
The default crate remains `no_std`, network-free, filesystem-free, and
runtime-free.

## Choose The Smallest Fixture

Use `MockTransport` for a fixed ordered list of exact requests and responses.
Use `DynamicMockTransport` only when request-dependent selection, bounded
recording, pagination, polling, cancellation, or fault injection matters.

`DynamicResponder` adapts a closure. A provider crate can instead implement
`ProviderFixtureBuilder` to expose named provider-specific fixture builders
without adding a provider-specific testkit crate.

## Transaction Boundary

Each dynamic operation follows this order:

1. Acquire the scenario's single in-flight selection guard.
2. Read the current successful sequence number.
3. Ask the fixture builder for a borrowed `ResponseFixture`.
4. Stage the complete fixture through the same sealed writer as `MockTransport`.
5. Commit a payload-free record and advance the successful sequence.

A builder rejection, response-capacity failure, invalid fixture, cancellation
before polling, or capacity exhaustion does not advance the sequence or create
a record. Builder-owned side effects cannot be rolled back by the SDK; dynamic
responders should therefore derive output only from `DynamicRequest` and
immutable fixture state.

Overlapping requests fail with `DynamicMockError::ConcurrentRequest`. This
keeps sequence-dependent responders deterministic. It does not impose a mutex,
allocator, runtime, or waiting policy.

## Bounded Recording

The caller supplies between one and `MAX_DYNAMIC_RECORDS` clean
`RequestRecordSlot` values. A committed `RecordedRequest` contains only:

- successful sequence number;
- finite method classification;
- encoded target length;
- request body length;
- request-header count; and
- response status.

No target bytes, query values, header names or values, request bodies, response
bodies, extension-method tokens, builder errors, or credentials are retained.
Slots deliberately provide no reset operation. Allocate a fresh slot array for
each independent scenario so stale observations cannot be mistaken for current
evidence.

## Validated Scripts

`PaginationScript` validates a finite page-one-through-last-page sequence. Page
numbers must increase by one, while page size, total entries, and final page
remain stable. The final fixture must represent the declared last page.

`ActionScript` validates a finite poll sequence. Progress cannot decrease,
intermediate states must be `Running`, and the final state must be `Success` or
`Error`. A custom `ProviderFixtureBuilder` remains available for deliberately
adversarial state regressions.

Both scripts borrow response fixtures and are bounded by
`MAX_DYNAMIC_RECORDS`. They select by the transport's successful sequence, so
failed staging can be retried against the same fixture.

## Streaming And Faults

`StreamFixtureSource::with_fault_at_observation` and
`StreamFixtureSink::with_fault_at_write` inject failures at exact one-based I/O
attempts. `StreamFixtureSink`'s maximum accepted write size models partial I/O.

`StreamPatternSource` provides two non-terminating inputs:

- `EndlessEmpty` emits only explicit zero-length chunks.
- `AlternatingEmptyData` requires a nonempty borrowed chunk and alternates it
  with empty chunks.

Pattern sources never report end-of-stream. Always drive them through a
validated `StreamPolicy`; byte, chunk, observation, and consecutive
zero-progress limits are the mechanism under test.

## Cancellation

The async transport future performs no responder call until first poll.
Dropping an unpolled future therefore leaves records and sequence state empty.
After polling starts, core response-attempt guards retain their ordinary
cleanup semantics if cancellation or unwinding occurs.

## Security Boundary

Dynamic scenarios are test infrastructure, not production transports.
Responders execute synchronously during a transport poll and must not perform
network, filesystem, clock, blocking, or unbounded work. Keep mismatch errors
finite and payload-free. Secret-bearing caller buffers remain caller-owned and
must be sanitized according to the application's threat model.
