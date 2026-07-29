# Raw Bounded HTTP Executor

Status: implemented for the v0.40.0 pentest stop.

The raw executor is the provider-neutral wire boundary beneath later
authentication and typed client policy. It accepts one validated
`TransportRequest`, one `RawResponsePolicy`, and caller-owned response storage.
It does not add authorization, `Accept`, redirects, proxies, response
decompression, retries, or cross-origin forwarding.

## Guarantees

- `TransportFailure` reports `NotSent`, `PossiblySent`, or `ResponseStarted`.
  Unknown send state always maps to `PossiblySent`.
- HTTP/1 response heads are bounded to 100 fields and a 64 KiB parser buffer.
- At most eight informational responses may be admitted by core policy.
  `101 Switching Protocols` is always rejected.
- Duplicate response fields, declared trailers, and observed trailer frames are
  rejected.
- Success and error responses have independent media and body limits.
- Actual data-frame bytes and frame observations are checked even without
  `Content-Length`. A truncated declared body fails in Hyper. Bytes beyond a
  smaller declared HTTP message body are not part of that response; connection
  reuse is disabled so they cannot be interpreted as another response.
- `HEAD`, `204`, and `304` never expose body bytes. A `204` response carrying
  `Content-Length` is rejected.
- Only operation-admitted response headers enter caller storage. Authentication,
  cookie, framing, proxy, and upgrade headers cannot be admitted.
- Request body and header-value staging allocations use cleanup-owning byte
  storage backed by `cloud-sdk-sanitization`.

## Allocation Boundary

`ResponseWriter` proves that retained body and header bytes fit caller-owned
storage. It does not prove that the complete HTTP/TLS stack performs no heap
allocation.

The opt-in reqwest adapter uses pinned Hyper HTTP/1 parsing with:

| Resource | Bound |
| --- | ---: |
| Response fields | 100 |
| HTTP/1 response-head parser buffer | 65,536 bytes |
| Informational responses | 8 |
| Response data frames | 4,096 |
| Buffered response body policy | 67,108,864 bytes |

TLS records, DNS resolution, socket buffers, task/runtime state, and upstream
implementation bookkeeping remain process allocations outside caller response
storage. The blocking raw client creates a current-thread Tokio runtime per
call and disables idle connection pooling. The async raw client uses the
caller's Tokio runtime. Both use the same raw Hyper engine.

## Security Boundaries

The configured endpoint and user agent are trusted operator configuration.
Request targets and provider policy must already be validated. Callers must not
retry mutations from `PossiblySent` or `ResponseStarted` without a later
operation-specific idempotency or reconciliation policy.

The raw executor intentionally has no credential input. Bearer and Basic
authentication policy are separate v0.41.0 and v0.42.0 milestones.
