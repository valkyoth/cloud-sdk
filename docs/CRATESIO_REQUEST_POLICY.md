# crates.io Identifiers, Queries And Pagination

Status: unreleased `1.1.0`; Commit 8 is accepted at `71e3f972`.
[Discovery execution](CRATESIO_DISCOVERY_POLICY.md) remains covered by that
checkpoint. Commit 9 [catalog execution](CRATESIO_CATALOG_POLICY.md) passed
incremental pentest and remediation retest; its evidence checkpoint awaits
GitHub. No tag or publication is authorized.

## Source Contract

The [public source lock](CRATESIO_SOURCE_LOCK.md) defines the complete OpenAPI
and Cargo registry contract. Supplementary implementation evidence is pinned
to `rust-lang/crates.io` commit `9ae7f769cea32f38ebc2ea9ec2ce455b47641511`:

- [identifier validation](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/crates/crates_io_validation/src/lib.rs)
  caps ASCII crate names at 64 and version text at 150 characters;
- [publish validation](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/krate/publish.rs)
  parses Cargo SemVer and limits keywords to 20 bytes;
- [keyword validation](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/crates/crates_io_database/src/models/keyword.rs)
  admits ASCII alphanumerics followed by alphanumerics, hyphens, underscores or plus;
- [pagination](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/helpers/pagination.rs)
  defaults to ten entries, caps `per_page` at 100, and keeps seek payloads opaque.

The request-policy checker pins all four complete source files by size and
SHA-256. Live verification also compares every public query parameter name
against the Rust contract fixtures. Existing drift checks additionally cover
schema, requiredness, descriptions, media and status changes. Source refresh
must explicitly review these supplementary pins; updating the primary lock
alone does not silently admit new identifier or pagination policy.

```sh
python3 scripts/check_cratesio_request_policy.py --fetch
scripts/check_cratesio_drift.py --fetch
```

The first command is part of the final `1.1.0` gate. Offline request-policy
checks and their mutation tests run in `scripts/checks.sh`.

## Identifier And Query Profiles

Default builds remain allocation-free and `no_std`. Public borrowed types
validate input before encoding. No lowercasing, Unicode normalization, SemVer
range interpretation or conversion of hyphens to underscores occurs.

| Type or policy | Admitted profile |
| --- | --- |
| Crate name | ASCII letter first, then ASCII alphanumeric, `-`, `_`; at most 64 bytes |
| Exact version | Cargo SemVer, `u64` major/minor/patch, complete pre-release/build grammar; at most 150 bytes |
| Keyword | Upstream ASCII keyword grammar; at most 20 bytes |
| Category | Lowercase ASCII/digit/hyphen components separated by `::`; 256-byte SDK cap |
| User | ASCII alphanumeric/hyphen/underscore login; 100-byte SDK cap |
| Team | `github:organization:team`; bounded ASCII components and 256-byte SDK cap |
| Owner | Explicit user/team distinction; no credential or path-token interpretation |
| Numeric ID | Canonical nonzero decimal through `i32::MAX`, matching OpenAPI ID widths |
| Date | Real Gregorian date, canonical `YYYY-MM-DD`, years 0001 through 9999 |
| Search | Nonempty Unicode text, 1024-byte SDK cap; no controls, backslashes or fragments |
| Seek | Opaque provider-issued unpadded base64url alphabet, 1024-byte SDK cap; never decoded |
| Numbered page | Nonzero, at most 10; deliberately conservative SDK policy, not a universal server maximum |
| Per page | Nonzero, at most 100; source default is 10 |
| Aggregate target | 4096-byte SDK cap; at most 16 path segments, parameter groups or entries per array |

SDK profile limits do not assert that an identifier exists, is available, or is
valid on every historical account. The server remains authoritative. Request
profiles do not impose validation on unrelated legacy response display fields.
The numbered-page policy intentionally avoids deep offsets; use returned seek
links where supported or the data-access policy's bulk alternatives.

`ApiPath` only accepts fixed route segments and checked public identifiers.
Secret path tokens remain in the separately reviewed credential API. A path
builder is not an operation allowlist or authorization token. `Query` binds its
operation family to the matching typed path before complete target encoding.
Reserved component bytes and Unicode are percent encoded by the existing
provider-neutral transactional snapshot encoder. Small-buffer and aggregate
capacity failures leave output unchanged.

Sort and include choices are finite and operation-specific. Unknown values,
duplicate groups, empty/duplicate arrays, `page` with `seek`, and shadowed
crate-list filters are rejected. `full` cannot be combined with other includes;
`versions` cannot silently hide `default_version`. Array query keys are emitted
as repeated encoded `ids[]` or `nums[]` pairs. `all_keywords` is space-separated,
and include lists use commas. Keys have deterministic order; list order and
exact identifier spelling remain caller-selected. Date and boolean spellings
follow their source-owned query contracts.

## Continuations

`MetaLinks` checks both optional next and previous links. It accepts provider
query-only links, origin-form links or exact official HTTPS links for the
selected production/staging API. Foreign hosts, userinfo, explicit ports,
fragments, noncanonical paths and static-download origins are rejected.

All original filters, include/sort values, array entries and page-size
parameters must be preserved. Comparison handles one layer of form encoding,
including `+` spaces and hexadecimal case, without treating decoded values as
query syntax. Missing/extra/duplicate parameters fail; array pairs must each
match one original entry. Original query bytes are retained for transport.
Numbered pages must advance exactly one step in the declared direction; seek
tokens cannot be empty or equal the immediately preceding token. Opaque seek
ordering is not interpreted; non-adjacent cycles are bounded by traversal limits.
The core URI validator requires brackets in query keys to be percent encoded.

`PageLink::transfer_to` uses the existing `ValidatedProviderLink` instead of
opening a raw provider-query constructor. The returned cleanup-owning object
rechecks the same transport endpoint, GET method and source operation at
dispatch. Path storage must remain borrowed while it exists. Transfer errors
clear the admitted destination; path-encoding preflight errors occur before
destination transfer. The caller still controls its borrowed input lifetimes.

`Traversal` wraps the neutral transactional request/item budget with fixed page
size and state limits. It handles validated links and legacy Cargo `more`
responses. Call admission once for every decoded response before continuing.
No full metadata JSON decoder or operation driver is claimed by these helper
types: later operation decoders pass checked meta fields into them. Every
actual attempt also needs the Commit 6 rate gate; response counting is not a
substitute for request scheduling, retry policy, or credential authorization.

## Verification And Dependency Admission

Tests cover exact bounds, malformed identifiers, Unicode and reserved bytes,
overflow, ambiguous versions, every query-name/operation combination, all
sort/include values, parameter conflicts, encoding atomicity, next/previous
links, changed filters and origins, array matching, traversal ceilings and
cleanup. A transport fixture verifies that a transferred continuation cannot
dispatch against another origin, method or operation.

`semver = 1.0.28` is newly direct as a **dev-only** independent Cargo grammar
oracle. It was already present transitively in the lockfile. Defaults remain
disabled; it has no selected dependencies or build script, uses MIT/Apache-2.0,
and declares Rust 1.68. Archive SHA-256:
`8a7852d02fc848982e0c167ef163aaff9cd91dc640ba85e263cb1ce46fae51cd`.
It uses allocation internally, which is why it is not the default runtime
parser. The bounded borrowed validator is checked against it using exact
examples, byte mutations and generated component combinations. This is test
evidence, not a proof of grammar equivalence or a completed pentest.

Implementation verification on 2026-09-09 passed the full repository suite,
Rust 1.92.0 through 1.98.1 matrix, local platform matrix, six packaged feature
graphs, fresh four-lockfile advisory scans, dependency policy and SBOM freshness.
Live crates.io drift and request-policy checks passed. The independent
[Hetzner live drift](SPEC_LOCK.md#reviewed-live-drift-2026-09-10) was resolved
in Commit 8 with schema, fixture, changelog and compatibility tests.
Publication remains disabled for this candidate.
