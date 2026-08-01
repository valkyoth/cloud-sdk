# Migrating To v0.45

v0.45 moves quota interpretation out of transports and into provider-owned
decoders. It also adds provider-neutral bounded quota and pure delay policy.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.45.0"
cloud-sdk-hetzner = "0.35.0"
cloud-sdk-reqwest = { version = "0.31.0", features = ["blocking-rustls"] }
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.25.2"
```

`cloud-sdk-sanitization` is unchanged and is not published. The testkit change
is dependency-only.

## Provider-Owned Decoding

Do not interpret `RateLimit-*` in a transport adapter. Retain only headers
admitted by the prepared operation and call `HetznerQuota::decode` with a
caller-observed `WallClockTimestamp`. The reqwest `InvalidRateLimitHeaders`
variant is removed because provider-specific metadata is outside transport
ownership.

Checked Hetzner responses now expose `quota()`. Typed provider errors expose
the same metadata through `HetznerApiError::quota`. Existing `rate_limit()`
accessors remain as single-bucket compatibility views.

`QuotaBucket`, `QuotaBuckets`, `QuotaExtension`, and `HetznerQuota` are not
`Copy`. Read-only accessors borrow these values. Use an explicit `Clone` only
when a second owned snapshot is required; cloned extension storage is cleared
independently when each owner is dropped.

Use `decode_response_at` when an HTTP-date `Retry-After` can occur. The old
`decode_response` remains valid for decimal delay-seconds and fixed-year HTTP
dates, but rejects obsolete RFC 850 dates that require a current-year input.

## Delay Policy

Replace direct timestamp subtraction or unbounded sleeps with `decide_delay`.
Supply current wall time, an optional previous observation, an explicit past
timestamp policy, a conflict policy, and a hard maximum delay. The function
returns data only. Continue to keep retry eligibility, attempt count, deadline,
sleeping, and cancellation in caller policy.

See [`QUOTA_AND_RETRY.md`](QUOTA_AND_RETRY.md) for complete examples and
security boundaries.
