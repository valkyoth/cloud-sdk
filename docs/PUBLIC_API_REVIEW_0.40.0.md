# v0.40.0 Public API Review

Date: 2026-07-29

Scope: delivery-phased raw HTTP execution and response-wire admission.

## Decision

Core adds `BlockingRawHttpExecutor`, `AsyncRawHttpExecutor`,
`RawResponsePolicy`, `ResponseMediaPolicy`, `TrailerPolicy`,
`InformationalResponseTracker`, `DeliveryPhase`, and
`TransportFailure<E>`. All types are provider-neutral and remain `no_std`.
The traits receive already validated request and response policy values; they
do not own authentication, retry, scheduling, or provider decoding.

Unknown delivery state fails closed as `PossiblySent`. `TransportFailure`
formats only static phase text and redacts its adapter error from `Debug`.

## Adapter API

`cloud-sdk-reqwest` adds `RawBlockingClientBuilder`,
`RawAsyncClientBuilder`, `RawBlockingClient`, `RawAsyncClient`,
`RawHttpError`, and pinned wire-limit constants. Raw builders have no
credential argument. Blocking, async, deterministic-root, and FIPS builds use
one internal Hyper HTTP/1 engine.

The engine directly writes admitted data frames into `ResponseWriter`, rejects
observed trailers and upgrades, disables idle pooling and hidden request
retries, and stages request bodies and header values in cleanup-owning
allocations.

The opt-in `fuzzing` feature exposes one doc-hidden, assertion-only entry point
for the isolated fuzz workspace. It accepts arbitrary bytes, returns no parsed
or sensitive data, and reuses the exact production head validator and body
budget. Applications do not need this feature.

## Testkit API

`cloud-sdk-testkit` adds `RawFault`, `RawFaultError`, and
`RawFaultExecutor`. The no-allocation executor implements both raw traits and
allows exact delivery-phase tests without a runtime or network.

## Compatibility

The release is additive except that `user-agent` is now transport-owned and
therefore rejected by `RequestHeader`. Applications should continue supplying
the validated user agent through reqwest client builders.

See [`MIGRATION_0.40.0.md`](MIGRATION_0.40.0.md).
