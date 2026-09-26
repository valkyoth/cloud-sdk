# crates.io Publish Contract

Commit 18 implements the stable Cargo publish framing over an explicitly
trusted blocking streaming adapter. It does not package, unpack, inspect or
build a crate. Stop before Commit 19 and pentest `04f24c38..HEAD`. No tag,
registry publication or live mutation is authorized by this checkpoint.

## Source Contract

The [Cargo Registry Web API](https://doc.rust-lang.org/cargo/reference/registry-web-api.html#publish)
defines `PUT /api/v1/crates/new`: four little-endian bytes for the JSON length,
the original JSON bytes, four little-endian bytes for the compressed archive
length, then the exact caller-provided `.crate` bytes. The media type is
`application/octet-stream`; JSON responses are requested explicitly.

The [crates.io source lock](CRATESIO_SOURCE_LOCK.md), public OpenAPI and
`src/controllers/krate/publish.rs` plus `crates/crates_io_validation/src/lib.rs`
at upstream `9ae7f769cea32f38ebc2ea9ec2ce455b47641511` are digest-verified by
`scripts/generate_cratesio_publish.py`. CI also checks the generated success
schema and fixture. Authority, status, inherited parameters and media changes
require review rather than silently expanding the contract.

The pinned crates.io success response requires `crate` and `warnings`, unlike
the generic Cargo registry protocol's looser optional-warning description.
Only status 200 with complete admitted JSON is accepted. The returned crate
name and ID must match the requested name. `invalid_categories`, the deprecated
`invalid_badges`, and `other` warnings remain available through protected
`DiscoveryValue` fields; each array is limited to 64 strings of 4096 bytes.
An acknowledgement does not prove archive availability or index propagation.
The provider may commit a mutation before a later error; do not retry it.

## Metadata Profile

`PublishMetadata::from_json` borrows the original immutable JSON while retaining
a protected parsed tree for validation. Duplicate JSON keys and unknown request
fields are rejected. There is no second archive copy and no JSON reserialization.
Accepted fields cover name, version, dependencies, features, authors,
description, documentation/homepage/repository URLs, readme content/path,
keywords, categories, license/external license path, badges, links and Rust
version. Dependencies cover name, requirement, features, optional/default-feature
flags, target, kind, registry and manifest rename.

The local metadata ceiling is 128 KiB, below the source's one-MiB ceiling.
Existing JSON limits also apply: depth 32, 16,384 nodes, 64 object members and
65,536-byte strings. This intentionally rejects some Cargo-valid large manifests.
There are at most 256 dependencies, 64 authors, five keywords and five categories.
Feature arrays have at most 300 entries. SPDX expressions and requirements are
at most 1024 bytes. SPDX uses the admitted parser; requirements use SemVer.

Dependency duplicates compare effective manifest alias, normalized kind and
exact optional target. Renaming, distinct kinds and distinct targets are allowed.
The conservative ASCII feature profile accepts an alphanumeric or underscore
start, then alphanumeric/underscore/hyphen/plus/dot characters, plus `dep:name`,
`dep/feature` and `dep?/feature` references. Dependency aliases permit an
underscore start, unlike published crate names. Unicode XID feature names
accepted by the pinned registry are outside this conservative local profile.
It does not resolve references or evaluate features. Target expressions admit
bounded `cfg` syntax (depth 8, 64 predicates), `all`, `any`, `not`, identifiers
and unescaped ASCII string values, or bounded triple-like literals. No target
catalog or cfg evaluation is implied. Rust versions accept numeric major/minor
with optional patch, not ranges, build metadata or prereleases.

URLs are inert HTTP(S) metadata with nonempty authority and no user-info,
whitespace or backslashes; validation does not fetch them or certify DNS/TLS.
Paths are nonempty relative slash-separated names without dot segments,
backslashes or colons; no filesystem operation or existence check occurs.

Most importantly, current crates.io derives much publication metadata from
`Cargo.toml` inside the archive and checks the JSON name/version against it.
The caller must supply a correctly packaged artifact and matching metadata.
The SDK does not validate that equivalence, license-file existence, actual
dependency availability, server quotas or permission to create a new crate.

## Authority And Streaming

`PublishRequest` validates nonzero archive length, at most 512 MiB locally,
representable framing lengths and checked total arithmetic. Provider quotas
can be lower. Caller-selected `StreamLimits` bound bytes, chunks, observations
and waits. `SlicePackage` borrows a preexisting byte buffer; any caller-owned
`BlockingStreamSource` may instead supply incremental reads.

`confirm_api` or `confirm_trusted` consumes the request and creates a non-cloneable
permit tied to credential origin. Temporary-token expiry, issuer, scope and
configuration must be qualified separately; Commit 19 owns their management.
`PublishClient` only constructs fixed production/staging endpoints, checks the
bound transport and identifying user agent, and uses the shared rate gate.

The trusted callback receives scoped authorization and a one-shot
`PublishUpload`, not an eagerly buffered request body. It must configure sensitive
Authorization, the exact declared content length, HTTP PUT, TLS verification,
deadlines and complete response framing. It must disable redirects, cookies and
retries, stream into that same exchange's direct sink, and commit the response
only after that exchange. The SDK checks upload completion before admitting
the reply, but a malicious adapter can lie about its actual network behavior.
This is the same explicit transport trust boundary as other mutation clients.
Commit 20 adds `PublishClient::execute_bundled`, `execute_bundled_local` and
`execute_bundled_async` plus the corresponding `RegistryClient::publish`,
`publish_local` and `publish_async` conveniences. These opt-in methods retain
the permit, origin, user-agent, shared admission and checked acknowledgement
rules without requiring a caller HTTP callback. Local sources may be non-Send;
Send execution has a Send future. Guards are installed before future creation.
These methods now use provider-neutral `BlockingRawUploadExecutor`,
`LocalRawUploadExecutor` and `AsyncRawUploadExecutor` contracts. Custom adapters
need only the `blocking`/`async` features; bundled networking remains opt-in.
The reqwest implementations delegate to the existing live HTTP uploader.

The neutral bundled adapter uses a one-chunk queue and sanitization-owned frame
storage; it never concatenates the archive. It sets the exact Content-Length,
checks source framing/progress, and requires completed upload plus a checked
response before committing response storage. It rejects an early final response
while the source is incomplete, uses the original total timeout across upload
and response, and disables retries, redirects, cookies and decompression.
Underlying transport/TLS buffers remain within their documented trust boundary.
Deadline enforcement cannot preempt non-cooperative caller source code.
Integrated qualification is recorded in the [Commit 20 ledger](CRATESIO_UNIFIED_CLIENT.md).
The strict generated facade
coverage check now passes all 51 rows, including actual successful publication
fixtures in three modes with both credential schemes. This is not live registry
publication or proof that arbitrary custom adapters satisfy the contract.

Short/overlong archives, source failure, sink failure, exhausted waits and
incomplete responses fail closed. Partial remote writes cannot be rolled back.
No status, timeout or ambiguous acknowledgement authorizes automatic replay.
Constructing another permit is fresh explicit caller consent, not an SDK retry.

## Memory And Verification

Stream scratch, credential scratch and response/header storage are cleared by
the existing sanitization guards on normal exits and unwinding. Source JSON,
archive bytes and any copies retained by the caller/adapter remain their
responsibility. Process abort, allocator exhaustion and hostile custom drop or
abort implementations do not have guaranteed cleanup. Parser allocation is
bounded by input limits but third-party allocation failure may still abort.

Regressions cover exact framing across small chunk boundaries, partial writes,
zero/max/oversized lengths, truncation, excess bytes, overclaimed reads, stalled
sources, empty scratch, sink failure/commit failure, skipped uploads, late
timeouts, both credential schemes, origin mismatch, secret erasure, Cargo
errors, wrong identities/status, oversized warnings and incomplete replies.
Metadata tests cover duplicate keys/dependencies, renames, invalid semantic
fields and bounded cfg recursion. Generator regressions exercise source and
schema drift plus verification-gate wiring. No live publication test runs in CI.

## Local Checkpoint Evidence

On 2026-09-25, `scripts/checks.sh` passed, including package verification,
workspace default/all-feature tests, warning-denied Clippy, feature-isolated
checks, documentation links, source contracts, fuzz harness metadata and the
500-line policy. The provider's all-feature run passed 191 unit tests, two
integration tests and 44 doctests. The alloc-only test run, Rust 1.92.0
all-feature compilation and `thumbv7em-none-eabi` alloc-only compilation passed.
Fresh RustSec scans, cargo-deny and all four SBOM freshness checks passed.

The separate live direct-pin query found the transport dependency follow-up
recorded in [Commit 20](cratesio-commit-plan.md#commit-20---unified-client-and-cargo-compatibility);
it is not claimed to pass. No live archive publication or independent pentest
of Commit 18 was performed by this implementation checkpoint.
