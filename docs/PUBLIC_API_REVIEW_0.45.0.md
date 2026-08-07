# v0.45.0 Public API Review

Date: 2026-08-01

Scope: provider-owned quota decoding and provider-neutral delay policy.

## Added API

`DelaySeconds`, `WallClockTimestamp`, `HttpDate`, and `RetryAfter` keep
relative durations, Unix timestamps, and HTTP dates type-separated.
`RetryAfter::parse` accepts all three RFC 9110 HTTP-date forms and requires a
caller time for two-digit-year resolution.

`QuotaBucketId`, `QuotaReset`, `QuotaBucket`, and `QuotaBuckets` provide fixed
capacities, distinct bucket identities, coherent counts, multiple reset
semantics, duplicate rejection, and bounded informational extensions.
`QuotaExtension` retains exact visible-ASCII values while redacting values in
diagnostics. It is deliberately non-`Copy` and volatile-clears its complete
name, value, and length storage when the final live owner is dropped. This is
best-effort cleanup for informational metadata, not a strong secret-storage
guarantee: ordinary moves can leave stale inline bytes at prior locations.

`QuotaDelayPolicy` and `decide_delay` add pure conflict, stale-time,
clock-rollback, unknown-reset, and caller-maximum decisions. The API performs
no sleep, retry, clock access, allocation, or I/O.

`HetznerQuota` owns decoding of the complete Hetzner three-header set plus
standard `Retry-After`. `CheckedHetznerResponse::quota` and
`HetznerApiError::quota` retain provider metadata after response storage is
cleared. `decode_response_at` accepts caller wall time.

## Removed And Changed API

`cloud_sdk_reqwest::TransportError::InvalidRateLimitHeaders` is removed with
the obsolete private transport parser. Reqwest retains bounded admitted
headers and does not interpret provider quota semantics.

`CheckedHetznerResponse::rate_limit` is no longer `const`; it derives the
legacy compatibility view from provider-owned quota. Its return type and
meaning for the Hetzner single bucket are unchanged.

`QuotaBucket`, `QuotaBuckets`, and `HetznerQuota` are intentionally not
`Copy`. Their read-only accessors take `&self`; callers must request an
explicit `Clone` when they need a separate owned snapshot.

`QuotaExtension` is also no longer `Copy`; callers must explicitly clone an
extension when retaining more than one owned instance.

## Security Review

All capacities and decimal arithmetic fail closed. Partial Hetzner header sets
are errors; duplicates remain rejected by `ResponseHeaders`. Date parsing
validates calendar bounds, leap years, leap seconds, weekday agreement,
full-timestamp obsolete-year resolution, and numeric overflow. Past and
rollback behavior is never implicit. Extensions are bounded, redacted, and
best-effort cleared when the final owner is dropped; they are explicitly not a
credential-storage boundary. Delay output is bounded by caller policy and
cannot trigger I/O or replay by itself. Large
fixed-capacity aggregates are borrowed through decision and decode paths, and
the optional owned response boundary boxes quota before branching into
success or provider-error decoding.
