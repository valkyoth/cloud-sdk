# Migrating To v0.36

v0.36 replaces the separate request content-type field with a complete bounded
header block and adds owned bounded response-header metadata.

## Dependency Versions

```toml
[dependencies]
cloud-sdk = "0.36.0"
cloud-sdk-hetzner = "0.29.0"
cloud-sdk-reqwest = { version = "0.24.0", features = ["blocking-rustls"] }
```

Related boundary releases are:

- `cloud-sdk-sanitization 0.15.4` as a dependency-only patch;
- `cloud-sdk-testkit 0.21.0` with exact header matching and response metadata.

## Request Headers

Replace `TransportRequest::with_content_type` with a typed header block:

```rust
use cloud_sdk::Method;
use cloud_sdk::transport::{
    ContentType, MediaType, RequestHeader, RequestHeaders, RequestTarget,
    TransportRequest,
};

let target = RequestTarget::new("/servers")?;
let entries = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::JSON),
];
let headers = RequestHeaders::new(&entries)?;
let request = TransportRequest::new(Method::Post, target)
    .with_headers(headers)
    .with_body(br#"{"name":"example"}"#);

assert!(request.headers().get("content-type").is_some());
# Ok::<(), Box<dyn core::error::Error>>(())
```

`RequestHeader::new` creates public metadata.
`RequestHeader::sensitive` marks a value that adapters must protect while all
header-value `Debug` output remains redacted regardless of classification.
`RequestHeaders::encode_http1` writes exact field lines only after complete
capacity admission and leaves undersized output unchanged.

Names are compared case-insensitively. Identical and conflicting duplicates
are both rejected. Names are limited to 64 bytes, values to 1,024 bytes,
request blocks to 32 entries and 8,192 encoded bytes.

## Reserved Ownership

Callers cannot set `Host`, `Content-Length`, `Transfer-Encoding`,
`Authorization`, proxy authorization, or hop-by-hop fields. The reqwest
adapter owns bearer authorization and HTTP framing. It derives Host and TLS SNI
from the same URL whose scheme, host, effective port, and base path are bound
to the verified `EndpointIdentity`.

Custom endpoints remain explicit credential destinations and must come only
from trusted operator configuration.

## Response Headers

`TransportResponse::headers` returns an owned `ResponseHeaders` block. Each
value is available as exact bounded bytes and carries a sensitivity marker.
Reqwest captures metadata before reading the body and rejects controls,
identical or conflicting duplicates, more than 32 entries, values above 1,024
bytes, or more than 8,192 aggregate encoded bytes.

`TransportResponse::content_type` and `rate_limit` remain typed conveniences
derived from the same captured metadata. A duplicate content type or
rate-limit field now returns `TransportError::InvalidResponseHeaders`; malformed
single values retain their specific typed error.

## Provider And Testkit Changes

Every Hetzner prepared request now contains an explicit `Accept:
application/json`; requests with JSON bodies also contain an explicit
`Content-Type: application/json`. The reqwest adapter no longer injects an
Accept policy.

Replace `ExpectedRequest::with_content_type` with
`ExpectedRequest::with_headers`. `MockError::ContentTypeMismatch` is replaced
by `MockError::HeadersMismatch`. `ResponseFixture::with_headers` supplies
complete raw response metadata.

## Retained Metadata Boundary

v0.36 marks and redacts sensitive response headers but retains them in a
fixed-capacity owned value. The cleanup-owning response workspace and retained
metadata lifecycle are assigned to v0.37-v0.38. Until then, callers must avoid
unnecessary copies and must not log header values.
