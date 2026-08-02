# Quota And Retry Policy

`cloud-sdk` separates provider quota decoding from transport execution.
Transports retain only response headers admitted by the prepared operation.
Provider crates interpret their own header names and semantics. Neither layer
reads a clock, sleeps, retries, or replays a request.

## Core Domains

`QuotaBuckets` holds at most eight distinct `QuotaBucket` values. Each bucket
has a bounded provider identity, nonzero limit, coherent remaining count, and
one explicit reset form:

- `QuotaReset::After(DelaySeconds)` for a relative duration;
- `QuotaReset::At(WallClockTimestamp)` for an absolute Unix timestamp;
- `QuotaReset::Unknown` when metadata is informational but not actionable.

Each bucket can preserve up to four bounded informational extensions. Values
remain available as exact bytes but are redacted from `Debug` because provider
partition keys can disclose account structure. Extensions are informational
metadata, not credential or secret storage. They are non-`Copy`, and the final
live owner volatile-clears its complete fixed-capacity name, value, and length
storage on drop through `cloud-sdk-sanitization`. This is best-effort defense
in depth: ordinary Rust moves can leave stale inline bytes at prior locations,
which are not dropped. Secret values require stable caller-owned or allocated
cleanup storage rather than `QuotaExtension`.

The inline aggregate is deliberately not `Copy`: it can occupy several
kilobytes at maximum capacity. Read-only accessors borrow buckets and quota
sets, while callers can use an explicit `Clone` when a second owned snapshot
is required. The optional owned Hetzner response decoder boxes quota once at
its allocation boundary and moves that box through success or error decoding.

`RetryAfter::parse` accepts decimal delay-seconds and all HTTP-date forms that
RFC 9110 requires recipients to accept: IMF-fixdate, obsolete RFC 850, and
obsolete asctime. The caller supplies wall time so the RFC 850 two-digit year
rule can be resolved without a hidden clock. Resolution compares the complete
candidate date and time with the exact 50-year civil-time boundary, not only
the year.

## Hetzner Decoding

Hetzner quota metadata is one complete set of `RateLimit-Limit`,
`RateLimit-Remaining`, and `RateLimit-Reset`. Any partial set, empty or
non-decimal value, numeric overflow, zero limit, or remaining count above the
limit fails closed. Header duplication is rejected earlier by the bounded
transport header collection.

```rust
use cloud_sdk::rate_limit::WallClockTimestamp;
use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};
use cloud_sdk_hetzner::rate_limit::HetznerQuota;

let mut storage = [0_u8; 8_192];
let mut headers = ResponseHeaders::new(&mut storage);
headers.try_push("ratelimit-limit", b"3600", HeaderSensitivity::Public)?;
headers.try_push("ratelimit-remaining", b"0", HeaderSensitivity::Public)?;
headers.try_push("ratelimit-reset", b"2", HeaderSensitivity::Public)?;
headers.try_push("retry-after", b"3", HeaderSensitivity::Public)?;

let quota = HetznerQuota::decode(&headers, WallClockTimestamp::new(1))?;
assert_eq!(quota.buckets().len(), 1);
# Ok::<(), Box<dyn core::error::Error>>(())
```

With the optional `serde` feature, `decode_response_at` carries decoded quota
on both `CheckedHetznerResponse` and `HetznerApiError`. `decode_response`
remains available when no caller clock is needed; an obsolete RFC 850
`Retry-After` value then fails with `WallClockRequired` rather than guessing a
century.

## Pure Delay Decisions

`decide_delay` considers only exhausted buckets. When multiple exhausted
buckets exist it uses the longest required bucket delay. An exhausted bucket
with `QuotaReset::Unknown` fails closed; unknown reset metadata on a bucket
that still has capacity remains informational.

```rust
use cloud_sdk::rate_limit::{
    DelayConflictPolicy, DelaySeconds, ExcessDelayPolicy, PastTimestampPolicy,
    QuotaDelayPolicy,
};

let policy = QuotaDelayPolicy::new(
    DelaySeconds::new(300),
    PastTimestampPolicy::Reject,
    ExcessDelayPolicy::Reject,
    DelayConflictPolicy::RetryAfterPrecedence,
);
assert_eq!(policy.maximum().get(), 300);
```

Conflict policy is explicit: standard `Retry-After` precedence, longest delay,
or rejection unless both sources agree. Past timestamps can become an
immediate zero delay or an error. Delays above the caller maximum are clamped
or rejected. A previous wall-clock observation can be supplied so rollback
fails before a decision is returned.

The decision does not imply that replay is safe. Combine it with the operation
metadata, exact request identity, delivery phase, attempts, and monotonic
budgets in the v0.46
[`RetryController`](RETRY_AND_IDEMPOTENCY.md). Cancellation and sleeping remain
caller policy.

## Compatibility

`RateLimit` remains as a legacy single-bucket view for pagination, polling,
and existing integrations. `HetznerQuota::rate_limit` and
`CheckedHetznerResponse::rate_limit` expose that view only when the provider
metadata is one absolute-reset bucket. New integrations should retain the
full `HetznerQuota` value.
