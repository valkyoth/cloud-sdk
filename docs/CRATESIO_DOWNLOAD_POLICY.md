# crates.io Downloads And Statistics

Status: unreleased `1.1.0`, logical Commit 11 implemented; pentest/GitHub pending.
Baseline: `51a7d946` (accepted Commit 10 evidence).

## Operations

| Operation | Preferred source and behavior |
| --- | --- |
| `download_version` | JSON URL metadata from the API; prefer direct static CDN for actual archives |
| `get_crate_downloads` | API for focused recent counts; database dumps for bulk analytics |
| `get_version_downloads` | API for a selected inclusive ninety-day window, optionally `before_date` |
| `list_reverse_dependencies` | bounded numbered API pages; database dumps for complete bulk graphs |

`DownloadRequest` binds method, path and query. `DownloadClient` executes the
JSON profile through the existing blocking/local/Send checked GET runner:
identifying user-agent, shared one-request-per-second API admission, exact
200/JSON response admission, bounded body/header storage, preserved provider
errors and cleanup. No credentials, redirects, automatic retries or crawl.
The JSON location is inert text, including when staging returns another origin.
It is never a transport capability. The older explicit 302 redirect proof
remains available; it only admits the exact source-correlated static authority.

## Statistics And Reverse Dependencies

Generated schema projections validate the complete source response, including
optional version metadata. Protected `DiscoveryValue` exposes closure-scoped
text and object enumeration without default payload logging.

Version buckets have positive int32 IDs, real Gregorian dates, nonnegative
int32 counts and unique `(version, date)` pairs. Version counts use one ID and
at most 90 rows. Crate counts admit at most five IDs and 450 rows plus at most
90 aggregate buckets with nonnegative int64 counts and unique dates. Observed
dates fit a ninety-day window. Explicit `before_date` includes that date and
the preceding 89 days, including leap-day arithmetic. No wall clock is inferred
from an arbitrary machine: absent an explicit date selector, freshness remains
provider metadata, not a freshness attestation. Route names/versions are not
numeric IDs; without included records the SDK cannot prove a numeric ID's
identity through a second API lookup and never silently performs that lookup.

`include=versions` must match response presence. Included versions must belong
to the requested crate and correlate with all referenced count IDs. Records
without counts are allowed. Schema validation retains nullable archived data,
while checking identifiers, versions, checksums and feature values.

Reverse dependencies explicitly reject seek, despite the shared OpenAPI query
parameter listing: the pinned controller uses offsets. Requests require an
explicit page size (1-100), with pages bounded to 1-10. Dependencies must
reference included version IDs, have unique IDs and identify the requested
dependency crate. Included versions identify the dependent crates, not the
requested crate. Requirement text and unknown kind spellings remain inert
metadata, not resolver input or a coerced normal dependency. Continuation
distinguishes `End`, `Next(Page)` and `LimitReached`; totals are not snapshot
isolation. No recursive resolution or bulk traversal occurs.

## Artifact Streaming

`ArtifactDownload` fixes crate/version, trusted expected length, trusted SHA-256
and complete `StreamLimits` before execution. It constructs only the official
static path and verifies transport origin and user-agent. Empty request headers
and an empty GET body prevent API token/cookie forwarding. There is no custom
URL argument, URL follow, automatic retry or staging-to-static credential path.

Execution accepts a caller-supplied `BlockingArtifactTransport`,
`LocalArtifactTransport` or `AsyncArtifactTransport`. The existing bundled
raw reqwest adapter buffers responses and does not implement these streaming
traits. Commit 20 explicitly requires bundled adapter/hash integration before
the complete provider can be qualified. Do not substitute a buffered adapter
and claim bounded streaming. A live adapter must
open the exact request on its immutable bound origin, without authentication,
redirects, retries, decompression or whole-body buffering, and enforce TLS,
framing/header correctness and connect/read deadlines. This trusted adapter
boundary is the same class of obligation as the raw HTTP executor, not proof
that arbitrary third-party implementations are correct.

`OpenedArtifact` admits status 200 only and identity HTTP content coding.
If Content-Length is supplied it must equal the trusted expected length; actual
byte accounting independently rejects short/long bodies. The `.crate` bytes
remain compressed archive bytes, not extracted files. Caller-owned scratch,
chunk, total byte, observation and no-progress limits bound the transfer.
No archive extraction, filesystem access or resume/range requests are supplied.

The transactional sink must hide partial bytes and roll them back on abort.
A fresh reviewed `ArtifactChecksum` implementation receives exactly accepted
bytes and must return SHA-256 over them. Mismatch/error prevents commit. This
is a required hook, not an in-house hash implementation or optional bypass.
Expected checksums must come from trusted registry/index metadata; deriving an
expected checksum from the downloaded body does not authenticate it. Tests
use an exact-input hook, not a cryptographic implementation claim.

Blocking errors and local/Send cancellation, including dropping an unpolled
future, clear scratch and abort the sink. Process abort, allocator exhaustion,
malicious adapters, and caller-retained copies remain outside cleanup guarantees.
No transport or hashing dependency was added to the default provider graph.

## Source And Drift Review

The source lock remains at upstream commit
`9ae7f769cea32f38ebc2ea9ec2ce455b47641511`. Added byte-locked controllers:

- [crate counts](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/krate/downloads.rs)
- [version location/counts](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/version/downloads.rs)
- [reverse dependencies](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/krate/rev_deps.rs)

The 2026-09-15 OpenAPI refresh only adds `add_owners` prose documenting owner
name prefixes. All 51 normalized operations, schemas, parameters, auth, Cargo
compatibility and typed data-access policy remain unchanged. Policy-page bytes
changed without typed policy changes. The stage/verify workflow reviewed and
accepted only these source digests; ownership implementation remains later work.

Hetzner Robot's live lock check passed. Hetzner Cloud and changelog checks
detected the new [network-members endpoint](https://docs.hetzner.cloud/changelog#2026-09-15-network-members-endpoint),
raising active Cloud operations from 208 to 209. That addition is NOT accepted
into the coverage lock by this checkpoint. The live drift gate intentionally
continues to fail until endpoint models/client/schema coverage are implemented
and reviewed in a separate maintenance change. Do not report current live
Hetzner coverage as complete on the strength of the older lock.

## Verification And Stop

Local qualification on 2026-09-15 passed `scripts/checks.sh` (including full
workspace tests, doctests, Clippy, package checks and fail-closed test policies),
the complete twelve-toolchain Rust matrix, the portable/native platform matrix,
allocation-only and default provider tests, all four SBOM freshness comparisons,
and live crates.io/controller checks. Direct dependency pins and Cargo tools
are current. No live credentialed requests, mutations, tag or publication ran.
The Hetzner Cloud/changelog checks are the explicit outstanding drift described
above; they are not included in a claim that every live API check is green.

Regression coverage exercises exact requests, response schemas, bad statistics,
duplicate IDs/dates, correlations, paging ceilings, all three JSON execution
modes, redacted output and buffer cleanup. Stream tests exercise partial reads
and writes, truncation, oversized archives, header mismatch, wrong origins,
checksum failure, source/sink errors, empty buffers and cancellation.
The new generator has offline malformed-schema and allocation/client guard tests.

Run the incremental pentest over `51a7d946..HEAD`, including the README convention
and source-lock changes. Do not start Commit 12, tag or publish until the user
confirms green pentest/retest and GitHub. The full `1.1.0` train remains pending.
