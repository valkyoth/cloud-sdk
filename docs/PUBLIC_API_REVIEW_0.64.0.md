# v0.64.0 Public API Review

Status: implementation stop reached; pentest required.

Scope: changes from signed v0.63.0 through v0.64.0.

## Added Provider API

- `UtcTimestamp` exposes a calendar-valid canonical UTC RFC 3339 value through
  `as_str` and checked-allocation `try_clone`.
- `ExactDecimal` exposes the exact bounded provider JSON number token through
  `as_str` and checked-allocation `try_clone`.
- `MetricPoint`, `MetricSeries`, and `Metrics` now retain exact numeric text,
  expose checked copies, and redact values from diagnostics.
- `ActionResultError::code_text`, `ApiErrorResponse::code_text`, and
  `HetznerApiError::code_text` preserve validated future provider codes while
  retaining the existing classified `ApiErrorCode` accessor.
- `CompositeResult::action`, `actions`, and `next_actions` preserve each source
  field independently. `CompositeResult::secret` distinguishes absent, null,
  and protected text results.

## Changed Provider API

- `MetricPoint::timestamp` returns `&ExactDecimal` instead of `f64`, and
  `Metrics::step` returns `&ExactDecimal` instead of `f64`.
- Owned action timestamps use `UtcTimestamp` internally and continue to expose
  `&str` through the existing `started` and `finished` accessors.
- `CompositeResult::actions` now means only the source `actions` field. It no
  longer flattens singular and follow-up actions into one collection.
- Action, resource, metric, timestamp, decimal, and error diagnostics redact
  provider-controlled text. Complete owned metric models no longer implement
  infallible `Clone`; use `try_clone`.

## Compatibility

These are intentional pre-1.0 provider API changes. No provider-neutral API,
default feature, runtime, transport, executor, filesystem, clock, TLS, or
secret-store boundary changes. The work remains behind the existing optional
`cloud-sdk-hetzner/serde` feature and uses only existing `alloc` and protected
sanitization boundaries.
