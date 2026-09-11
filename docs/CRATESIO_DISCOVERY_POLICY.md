# crates.io Discovery Contract

Status: unreleased `1.1.0`, logical Commit 8 passed incremental pentest and
remediation retest for `89910ad7cab7a1549a67f16e7bea7556557c4f3e` through
`2c0746982c927929f63a08ff034bd3d132f0a4d3`. The user confirmed GitHub green
on evidence checkpoint `71e3f972ee68995be7b0be048dc7a856c5f1a611`.
This checkpoint also includes the [Hetzner drift review](SPEC_LOCK.md#reviewed-live-drift-2026-09-10).
Commit 9 reuses the private checked GET runner for the
[catalog surface](CRATESIO_CATALOG_POLICY.md); its pentest and remediation
retest passed, with GitHub approval pending on the evidence checkpoint.

## Executable Scope

All seven operations use anonymous GET, no body, and no credentials:

| Operation | Target below `/api/v1` | Success model |
| --- | --- | --- |
| `list_categories` | `/categories` | `DiscoveryPage<Category>` |
| `find_category` | `/categories/{category}` | `Category` |
| `list_category_slugs` | `/category_slugs` | `Vec<CategorySlug>` |
| `list_keywords` | `/keywords` | `DiscoveryPage<Keyword>` |
| `find_keyword` | `/keywords/{keyword}` | `Keyword` |
| `get_site_metadata` | `/site_metadata` | `SiteMetadata` |
| `get_summary` | `/summary` | `Summary` |

`DiscoveryRequest` owns an immutable typed operation/path/query snapshot.
Its target encoder reuses the reviewed atomic encoder. Metadata is read-only,
safe, no known cost, no request ID and **never retry**. Request construction
remains allocation-free. Models and checked decode require `alloc`; client
execution requires `blocking` or `async`. No transport dependency is added.

## Source And Models

The [source lock](CRATESIO_SOURCE_LOCK.md) binds the OpenAPI representation.
The seven JSON fixtures are generated from its required fields, examples and
nullable types, not captured account data. The source-derived summary fixture
includes every field of its embedded Crate and CrateLinks schemas.

All required fields must be present; a nullable field is not an optional field.
Category parent/child collections and the site banner may be absent, but
explicit null is rejected. Unknown fields are fully parsed and bounded, then
discarded. Open-ended badge objects retain their bounded JSON structure and
closure-scoped protected text/number access. No float conversion loses numeric
spelling. Counts reject negatives, fractions, exponents and out-of-range values.

Timestamps reuse the core calendar validator and retain source timezone
spelling. The local profile accepts RFC 3339 UTC or numeric offsets, lowercase
`t/z`, and up to nine fractional digits; leap seconds and year zero reject.
No implicit UTC conversion occurs. Response version strings remain bounded
metadata rather than being coerced into newly accepted publish identifiers.

`DiscoveryRequest::decode` is a lower-level decoder over an already admitted
response. Its endpoint argument supplies pagination context; it is not evidence
of where a caller obtained that response. Use the client for inseparable
transport binding and execution, or independently enforce those boundaries
when integrating a custom transport without `std`.

All models and errors redact payloads in Debug. Temporary JSON keys, numbers,
strings and retained badge text use the admitted neutral sanitization crate.
Known public response fields are returned as ordinary owned strings, not as
secret storage. Applications must escape untrusted descriptions/banner text
when rendering, must not log sensitive copied values, and must not treat
returned CDN, repository, homepage or link text as trusted destinations.

## Resource Limits

These are conservative SDK limits unless explicitly attributed to the source;
an oversized response fails, never truncates into a successful partial model.

| Bound | Limit |
| --- | --- |
| Complete response | Caller-selected nonzero maximum, at most 8 MiB |
| JSON value containers / total values | 32 levels / 16,384 values |
| Object members / key bytes | 64 / 256 |
| Array elements | 1,024 |
| Decoded string bytes | 65,536, with smaller field-specific bounds |
| Number token | Shared parser limit of 128 bytes |
| Category relationship nesting | Eight edges below the root category |
| List response items | Requested page size, at most 100 |
| Numbered page depth | 10 |
| Each summary collection | 10, matching the documented summary contract |

Wire-level token and total-field limits still apply independently. Storage
growth uses fallible reservation; allocator abort, deliberately leaked objects,
caller copies and transport/OS buffers remain outside cleanup guarantees.

## Pagination

Category and keyword controllers use numbered offset pagination. The pinned
implementation files `src/controllers/category.rs` and
`src/controllers/keyword.rs` are verified by the request-policy checker at
`9ae7f769cea32f38ebc2ea9ec2ce455b47641511`. Although the shared OpenAPI query
parameters include `seek`, these operation constructors reject it.

Current controllers return `meta.total`. Continuation is derived from the
original page and page size; the page-depth ceiling returns `LimitReached`
instead of claiming end-of-data. Optional returned next/previous links are
checked against the same origin, resource, filters, page size and direction.
Supplied links, including nulls, must agree with the count-derived continuation
and request-derived previous page; contradictory metadata fails with `Binding`.
In particular, null cannot turn continuation or `LimitReached` into `End`, and
a next link cannot extend count-derived completion. Omitted links retain the
derived state. They cannot introduce a seek cursor. Changing totals do not
provide snapshot isolation. Callers explicitly create each next request and
must obey the same rate gate and their own total-page/item/time budget; there
is no bulk crawler.

## Execution And Cleanup

`DiscoveryClient::production` and `::staging` accept a trusted raw executor
bound to the exact official origin. The new neutral `BoundUserAgent` contract
exposes the stable user-agent actually sent by that executor. Both origin and
identifying user-agent are verified at construction and before dispatch.
The reqwest raw adapters implement this inspection contract. Generic request
headers still cannot override transport-owned user-agent configuration.

Blocking, local-async and Send-async entry points share these policies:

- Exactly one bodyless GET, JSON accept, identity content encoding.
- No Authorization, cookies, custom origin, implicit redirects, retries or sleeps.
- Raw response caps, JSON media policy, bounded informational heads and no trailers.
- Exact HTTP 200 and the checked Cargo error-envelope policy before model decode.
- Full caller response/header storage cleanup on success, rejection and cancellation.
- Async storage ownership begins at future construction, so unpolled drop clears it.
- Gate admission begins on execution/first poll; concurrent attempts fail closed.

The process-wide gate serializes the entire attempt across clients and origins.
An owned admission guard holds no mutex across await. Completion, failure and
cancelled in-flight attempts impose a fresh one-second quiet interval.
Valid Retry-After on a checked success or provider error can only extend it.
Commit 9 remediation bounds provider deferral to 24 hours (`MAX_PROVIDER_DELAY`).
Larger values return `ScheduleError::Overflow` with a capped wait, never a
permanently poisoned gate. Intrinsic clock overflow or unwinding closes the gate; no public reset
exists. Wall time is used only for HTTP-date delay interpretation.
Other processes and other applications sharing egress require operator coordination.
A custom raw executor remains trusted to report truthful identity, enforce the
response policy and avoid injecting credentials or extra exchanges.

## Verification

`cargo test -p cloud-sdk-cratesio --all-features` covers all seven source fixtures,
blocking/local/Send parity, required/null/unknown fields, complete summary
records, timestamps, atomic targets, malformed pagination and limits, response
cleanup, unpolled and pending cancellation, endpoint/user-agent mismatches,
status errors and Retry-After. Reqwest loopback tests verify inspected user-agent
bytes against the actual request. Default and alloc-only builds remain separate
verification targets. No tests contact the live API or use credentials.

```sh
python3 scripts/test-cratesio-discovery-fixtures.py
python3 scripts/generate_cratesio_discovery_fixtures.py
python3 scripts/check_cratesio_request_policy.py --fetch
scripts/check_cratesio_drift.py --fetch
scripts/check_hetzner_api_surface.sh --fetch
```

The fixture generator verifies exact pinned upstream bytes before generation.
Use `--write` only after source review. Offline tests reject changed operation
identity, absent operations, unsupported/ref-cyclic schemas, stale, missing and
extra fixture files. The complete checkpoint is subject to the repository,
compatibility, platform, package, dependency and incremental pentest gates.

Local implementation qualification on 2026-09-10 passed `scripts/checks.sh`,
the complete Rust 1.92.0 through 1.98.1 matrix, configured portable/native
platform checks, all six packaged feature graphs, workspace Clippy/tests/docs,
default/alloc-only tests and separate blocking/async feature checks.
The provider suite contains 82 unit tests, one integration test and 14 doctests.
Four fresh RustSec lockfile scans, cargo-deny policies, direct dependency/tool
freshness, all four SBOM graphs and live crates.io/Hetzner source gates passed.
All six candidate publication flags remain false. These are local checks,
not an independent pentest result or GitHub approval for this new checkpoint.

The subsequent pagination pentest finding was reproduced by two regression
matrices that failed on the original decoder. Both now pass for category and
keyword lists, including on Rust 1.92.0 with only `alloc` enabled. The updated
provider suite (84 unit tests, one integration test and 14 doctests), Clippy,
and `scripts/checks.sh` passed. The user confirmed the independent retest is
green. The [permanent report](../security/pentest/cratesio-commit-8.md) records
the finding and final qualification; GitHub approval was subsequently confirmed.
