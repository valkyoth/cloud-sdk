# Migrating To v0.35

v0.35 moves canonical request-target validation into `cloud-sdk` and exposes
path and query components separately.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.35.0"
cloud-sdk-hetzner = "0.28.0"
cloud-sdk-reqwest = { version = "0.23.0", features = ["blocking-rustls"] }
```

Related boundary releases are:

- `cloud-sdk-sanitization 0.15.3` as a dependency-only patch;
- `cloud-sdk-testkit 0.20.0` with exact query-state matching.

## Structured Targets

`RequestTarget::new("/path?query")` remains available and now admits only the
canonical query dialect. New code that constructs components separately can
use transactional assembly:

```rust
use cloud_sdk::transport::{
    CanonicalQuery, RequestPath, RequestQuery, RequestTarget,
};

let path = RequestPath::new("/servers")?;
let query = CanonicalQuery::new("name=test%20server&page=1")?;
let mut storage = [0_u8; 128];
let target = RequestTarget::assemble(
    path,
    RequestQuery::Canonical(query),
    &mut storage,
)?;

assert_eq!(target.path(), path);
assert_eq!(target.query_bytes(), Some(query.as_str().as_bytes()));
# Ok::<(), Box<dyn core::error::Error>>(())
```

`RequestQuery::Absent` differs from
`RequestQuery::Canonical(CanonicalQuery::new("")?)`: the latter preserves the
trailing `?`. Query pairs preserve insertion order and duplicate keys.
`QueryPair::value()` returns `None` for `flag` and `Some("")` for `flag=`.

`RequestTarget::assemble` writes only the prefix exposed by
`target.as_str()` and `target.len()`. The unused output tail remains untouched
and may contain bytes from an earlier use. Never log, hash, sign, or transmit
the complete scratch buffer; apply the caller's cleanup policy to the complete
buffer before reusing it across a sensitive boundary.

Spaces use `%20` in `CanonicalQuery`. Providers that explicitly require
form-style `+` encoding must construct `FormQuery` and assemble it through
`RequestQuery::Form`; `RequestTarget::new` never infers form semantics.

## Rejected Input

Paths reject repeated slashes, `.` and `..` segments, fragments, backslashes,
raw non-ASCII, malformed or lowercase percent triplets, encoded path
separators, encoded controls, and percent-encoded unreserved bytes. Structured
queries reject empty keys or pairs, multiple raw `=`, raw fragments,
backslashes, spaces, non-ASCII, malformed or lowercase percent triplets,
encoded controls/fragments, and percent-encoded unreserved bytes.

Percent-encoded non-ASCII octets remain exact ASCII wire text and are not
decoded by core. Encoded query component delimiters such as `%26` and `%3D`
remain data; raw `&` and `=` retain pair syntax.

## Error Changes

`RequestTargetError` now wraps precise `RequestPathError` and
`StructuredQueryError` values. Match `RequestTargetError::Path(error)` or
`RequestTargetError::Query(error)` instead of the removed coarse `Empty`,
`NotOriginForm`, and `InvalidByte` target variants.

`cloud-sdk-reqwest::EndpointError::InvalidTargetEncoding` is removed.
Malformed targets can no longer reach the adapter; construct a
`RequestTarget` first and handle its core validation error.

Hetzner `EndpointPathError::NonCanonical` identifies provider paths rejected
by the provider-neutral canonical grammar.
