# Migrating To v0.49

v0.49 adds opt-in incremental provider JSON decoding. Existing buffered
checked decoding, requests, transports, and default feature graphs are
unchanged.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.49.0"
cloud-sdk-hetzner = "0.37.0"
cloud-sdk-reqwest = "0.32.2"
cloud-sdk-sanitization = "0.16.0"
cloud-sdk-testkit = "0.28.1"
```

`cloud-sdk-hetzner` is the code release. `cloud-sdk` publishes aligned facade
documentation. Reqwest and testkit receive dependency-only patches.
Sanitization is unchanged and is not published.

## New Boundary

Enable `cloud-sdk-hetzner/serde`, implement `IncrementalJsonVisitor`, feed
bounded chunks through `IncrementalJsonDecoder::push`, and always call
`finish`. Treat only `IncrementalJsonProgress::Complete` as full-document
validation. `Stopped` means the visitor deliberately left the remainder
unvalidated.

No existing buffered code needs migration. Continue using `decode_response`
when operation binding and a complete typed success or provider-error model
are required. See [`INCREMENTAL_DECODING.md`](INCREMENTAL_DECODING.md) for
limits, cleanup ownership, and a complete visitor example.
