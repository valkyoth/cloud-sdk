# Raw Bounded HTTP Executor

Status: implemented; the v0.40.0 pentest and final retest passed.

The raw executor is the provider-neutral wire boundary beneath later
authentication and typed client policy. It accepts one validated
`TransportRequest`, one `RawResponsePolicy`, and caller-owned response storage.
It does not add authorization, `Accept`, redirects, proxies, response
decompression, retries, or cross-origin forwarding.

## Guarantees

- `TransportFailure` reports `NotSent`, `PossiblySent`, or `ResponseStarted`.
  `ResponseStarted` means any informational or final head was observed.
  Unknown send state always maps to `PossiblySent`.
- HTTP/1 response heads are bounded to 100 fields, 64 KiB of encoded
  name/value field bytes, and a 64 KiB pinned Hyper parser buffer.
- At most eight informational responses may be admitted by core policy.
  `101 Switching Protocols` is always rejected. The adapter cancels the
  in-flight request as soon as either condition is observed.
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
  storage backed by `cloud-sdk-sanitization`. The first-party raw reqwest
  adapters reject request bodies before allocation above the 8 MiB large
  preparation profile. The provider-neutral raw traits do not impose this
  adapter-local limit on external executors.
- Every execution uses core's `ResponseAttempt`; failed, timed-out, unwound, or
  cancelled attempts clear complete caller body and header storage before it
  can be reused. `ResponseWriter` exposes no direct mutation or commit bypass.
- The isolated `raw_response_parser` target fuzzes the post-parse response-head
  validator and streamed-body budget. The separate `raw_http1_wire` target
  feeds arbitrary bytes through Hyper's HTTP/1 parser, informational callback,
  frame stream, and the same production validator/body consumer. Its 66,560
  byte input budget reaches the encoded-head limit and limit-plus-one case;
  canonical seeds cover 101, informational overflow, framing conflicts,
  trailers, truncation, and excessive header counts.

## Allocation Boundary

`ResponseWriter` proves that retained body and header bytes fit caller-owned
storage. It does not prove that the complete HTTP/TLS stack performs no heap
allocation.

The opt-in reqwest adapter uses pinned Hyper HTTP/1 parsing with:

| Resource | Bound |
| --- | ---: |
| Response fields | 100 |
| Encoded response-header fields | 65,536 bytes |
| Pinned Hyper HTTP/1 parser buffer | 65,536 bytes |
| Informational responses | 8 |
| Response data frames | 4,096 |
| Adapter-owned request-body copy | 8,388,608 bytes |
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

The raw executor intentionally has no credential input. The 8 MiB request-copy
limit is a first-party `cloud-sdk-reqwest` guarantee, not a core raw-trait
guarantee. Bearer authentication policy is implemented separately in v0.41.0;
see [`AUTHENTICATION_POLICY.md`](AUTHENTICATION_POLICY.md). Basic
authentication and canonical signing inputs remain the v0.42.0 milestone.
