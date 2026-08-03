# v0.49.0 Public API Review

Date: 2026-08-03

Scope: provider-owned bounded incremental JSON events, visitors, limits,
progress, errors, and decoder lifecycle.

## Added API

`cloud_sdk_hetzner::serde` exports `IncrementalJsonDecoder`,
`IncrementalJsonLimits`, `IncrementalJsonVisitor`, `IncrementalJsonEvent`,
`VisitControl`, `IncrementalJsonProgress`, `IncrementalJsonError`, and
`IncrementalJsonLimitsError` behind the existing optional `serde` feature.
`cloud-sdk-sanitization` adds `try_append_secret_string` and the payload-free
`SecretStringAppendError` behind its existing `alloc` feature.

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
Visitor callbacks are pre-poisoned: unwinding leaves the decoder failed, while
normal continuation alone restores the active state. Stop clears all owned
lexical and structural staging before returning.

## Compatibility

Decoder additions are provider-owned and gated by the existing `serde`
feature. The reusable protected-string helper belongs to the existing neutral
sanitization boundary and is gated by its existing `alloc` feature.
The default no_std provider graph, buffered checked decoder, transport traits,
request preparation, and response models retain their signatures and
behavior. No third-party dependency or feature changes.

## Security Review Focus

- duplicate decoded keys across arbitrary chunk boundaries;
- grammar and trailing-document rejection;
- partial UTF-8, escape, and surrogate state;
- finite numeric admission matching the buffered decoder;
- exact and one-over resource ceilings;
- fallible protected key and number growth plus fallible structural storage;
- panic-safe scratch guards and callback poisoning;
- cleanup after success, failure, visitor error, and drop;
- redacted payload-bearing diagnostics;
- unambiguous pending, stopped, complete, and failed states; and
- explicit separation from HTTP and operation-envelope admission.
