# v0.40.0 Public API Review

Date: 2026-07-29

Scope: delivery-phased raw HTTP execution and response-wire admission.

## Decision

Core adds `BlockingRawHttpExecutor`, `AsyncRawHttpExecutor`,
`RawResponsePolicy`, `ResponseMediaPolicy`, `TrailerPolicy`,
`InformationalResponseTracker`, `DeliveryPhase`, and
`TransportFailure<E>`. Pentest remediation also adds `ResponseAttempt`, a
provider-neutral `no_std` guard obtained through
`ResponseWriter::begin_attempt`. It clears prior uncommitted residue on entry
and clears complete body/header storage when dropped without a commit.
All types remain provider-neutral and `no_std`.
The traits receive already validated request and response policy values; they
do not own authentication, retry, scheduling, or provider decoding.

Unknown delivery state fails closed as `PossiblySent`. `TransportFailure`
formats only static phase text and redacts its adapter error from `Debug`.

## Adapter API

`cloud-sdk-reqwest` adds `RawBlockingClientBuilder`,
`RawAsyncClientBuilder`, `RawBlockingClient`, `RawAsyncClient`,
`RawHttpError`, `MAX_RAW_REQUEST_BODY_BYTES`, and pinned wire-limit constants.
Raw builders have no credential argument. Blocking, async, deterministic-root,
and FIPS builds use one internal Hyper HTTP/1 engine.

The engine writes admitted data frames through `ResponseAttempt`, rejects
observed trailers and upgrades, disables idle pooling and hidden request
retries, caps request-body copies at 8 MiB before allocation, and stages request
bodies and header values in cleanup-owning allocations.

The opt-in `fuzzing` feature exposes two doc-hidden, assertion-only entry points
for the isolated fuzz workspace. One drives the production post-parse head
validator and body budget; the other feeds arbitrary bytes through an in-memory
Hyper HTTP/1 connection and the same response processing. Neither returns
parsed or sensitive data. Applications do not need this feature.

## Testkit API

`cloud-sdk-testkit` adds `RawFault`, `RawFaultError`, and
`RawFaultExecutor`. The no-allocation executor implements both raw traits and
allows exact delivery-phase tests without a runtime or network.

## Compatibility

The release is additive except that `user-agent` is now transport-owned and
therefore rejected by `RequestHeader`. Applications should continue supplying
the validated user agent through reqwest client builders.

See [`MIGRATION_0.40.0.md`](MIGRATION_0.40.0.md).
