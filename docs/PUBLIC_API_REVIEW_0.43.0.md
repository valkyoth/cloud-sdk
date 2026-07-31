# v0.43.0 Public API Review

Date: 2026-07-31

Scope: complete Hetzner migration from prepared operations through
authenticated bounded raw execution.

## Core API

`PreparedRequest` now owns mandatory `AuthenticationScopePolicy` and
`RawResponsePolicy` values. Its constructor accepts both policies, exposes
them through accessors, and can produce the exact `AuthenticatedRequest`.
Prepared blocking and async execution require authenticated transport traits.
There is no compatibility path through `BlockingTransport` or
`AsyncTransport`.

`AuthenticatedRequest::new` now requires the raw response policy. This removes
adapter inference of response size, media, admitted headers, informational
responses, and trailer behavior.

`RawResponsePolicy` stores admitted header names in bounded inline storage.
This allows a provider to create a complete policy without lending temporary
header-array storage. `TransportFailure::map` preserves its delivery phase
while mapping a payload-free inner error.

## Provider API

Every active Hetzner prepared operation now binds the exact Cloud, DNS,
Security, or Storage service identity and complete official endpoint,
authentication, and raw response policy. The public prepared-operation types
are retained; their `PrepareOperation` output is stricter.

`HetznerPreparationError` adds `InvalidRawResponsePolicy`. Its display message
is static and payload-free.

## Adapter API

Bearer and Basic clients retain their public builders and authenticated
transport traits, but their error type is now
`AuthenticatedTransportFailure`, preserving `NotSent`, `PossiblySent`, or
`ResponseStarted`. Internally they use the shared bounded raw Hyper engine.
Authenticated clients remain type-separated from credential-free raw clients.

The removed async body staging module and legacy high-level reqwest send path
were private.

## Testkit API

`MockTransport` adds authenticated trait implementations.
`PreparedRequestRecord` adds authentication and raw response policy accessors.
The record remains redacted and does not copy request values.

## Compatibility Decision

The constructor and prepared-execution source breaks are accepted under the
pre-1.0 policy. Keeping optional policy fields or a legacy fallback would
allow authenticated sends without complete wire limits and undermine the
v0.43 migration goal.
