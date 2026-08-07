# Migrating To v0.62.0

v0.62.0 is an internal source milestone after signed v0.61.0. No crate is
published; applications remain on the versions from the v0.60.0 checkpoint.

The checked Hetzner success enum gains source-complete variants for locations,
certificates, and Storage Boxes. Code exhaustively matching
`HetznerSuccess` must add these variants. The enum was already a development
API and this change intentionally lands at the neutral freeze before 1.0.

`get_zone_zonefile` and certificate operations now retain their exact DNS and
security service identities during checked response decoding. Applications
that constructed synthetic prepared requests with the compute service for
these operations must use `DnsService` or `SecurityService` respectively.

Typed read execution now returns `AssociatedCheckedResponse<O>`. Pass that
value directly to the checked decoder:

```rust,ignore
let checked = prepared.execute_blocking(transport, body, headers)?;
let response = decode_associated_checked_response(checked)?;
```

The old two-argument form is removed because an independently supplied
prepared request could not prove that it produced the checked guard. Advanced
code can call `into_untyped` explicitly, but an erased guard cannot re-enter
the associated decoder.

`HetznerQuota::buckets()` now returns compact `HetznerQuotaBucket` values. Use
the fallible, allocation-free adapter before applying neutral delay policy:

```rust,ignore
let buckets = quota.to_quota_buckets()?;
let decision = decide_delay(
    &buckets,
    quota.retry_after(),
    now,
    previous_now,
    policy,
)?;
```

`SensitiveText` and response/error types containing protected provider text no
longer implement `Clone`. Move or borrow these values and use their scoped
secret accessors; there is deliberately no infallible secret duplication path.

The next cumulative crates.io checkpoint is v0.65.0.
