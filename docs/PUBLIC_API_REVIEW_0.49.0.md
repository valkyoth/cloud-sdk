# v0.49.0 Public API Review

Date: 2026-08-03

Scope: provider-owned bounded incremental JSON events, visitors, limits,
progress, errors, and decoder lifecycle.

## Added API

`cloud_sdk_hetzner::serde` exports `IncrementalJsonDecoder`,
`IncrementalJsonLimits`, `IncrementalJsonVisitor`, `IncrementalJsonEvent`,
`VisitControl`, `IncrementalJsonProgress`, `IncrementalJsonError`, and
`IncrementalJsonLimitsError` behind the existing optional `serde` feature.

Events borrow key, string-fragment, and number text only for one callback.
Payload-bearing event and visitor-error diagnostics are redacted. Visitor
errors remain recoverable through `into_visitor_error` without being formatted
by the decoder.

Limits are private-field values with reviewed defaults, read accessors, and
builders that can only lower hard ceilings. Value and key tokens, aggregate
and per-object fields, decoded strings, number tokens, exponent digits,
nesting, and complete input bytes are independently bounded.

## Lifecycle

`push` validates arbitrary chunks but remains pending. `finish` is mandatory
and uniquely reports complete-document validation. Visitor stop, decoder
failure, and completion are terminal. `Stopped` cannot be confused with
`Complete`, and later bytes are not silently accepted as validated.

## Compatibility

All additions are provider-owned and gated by the existing `serde` feature.
The default no_std provider graph, buffered checked decoder, transport traits,
request preparation, and response models retain their signatures and
behavior. No third-party dependency or feature changes.

## Security Review Focus

- duplicate decoded keys across arbitrary chunk boundaries;
- grammar and trailing-document rejection;
- partial UTF-8, escape, and surrogate state;
- exact and one-over resource ceilings;
- protected growth-aware key and number staging;
- cleanup after success, failure, visitor error, and drop;
- redacted payload-bearing diagnostics;
- unambiguous pending, stopped, complete, and failed states; and
- explicit separation from HTTP and operation-envelope admission.
