# crates.io Catalog Contract

Status: unreleased `1.1.0`, logical Commit 9 passed incremental pentest and
remediation retest at `41fd2611ebe4468fca56e0ab12891d06eb06b622` against
accepted checkpoint `71e3f972ee68995be7b0be048dc7a856c5f1a611`. The
[permanent report](../security/pentest/cratesio-commit-9.md) records the evidence.
Wait for GitHub on the evidence checkpoint; do not tag, publish or start Commit 10.

## Executable Scope

| Source operation | GET target below `/api/v1` | Profile |
| --- | --- | --- |
| `list_crates` | `/crates` | Complete crates.io web list or explicit minimal Cargo search |
| `find_crate` | `/crates/{name}` | Named metadata with includes |
| `find_new_crate` | `/crates/new` | Metadata for the literal crate name `new` |

Request construction is allocation-free; response models require `alloc`.
`blocking` and `async` enable checked official-origin execution without adding
a network/TLS dependency. Metadata is read-only and never automatically retried.
No request publishes, changes an account, traverses a list or follows a returned
crate link implicitly. Prefer Cargo's sparse index for dependency resolution,
static downloads for artifacts and the database dump for bulk analysis.

## Source And Schema

The [source lock](CRATESIO_SOURCE_LOCK.md) covers all OpenAPI rows. Additional
implementation evidence at `9ae7f769cea32f38ebc2ea9ec2ce455b47641511` is pinned by
`check_cratesio_request_policy.py`:

- [search controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/krate/search.rs)
  binds filters, numbered/seek pagination, sort defaults and the relevance limit;
- [metadata controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/krate/metadata.rs)
  binds include selection and the literal-new alias, including its query support.

The three committed fixtures are derived from pinned OpenAPI fields/examples.
Independent example fragments are normalized into coherent list totals and
full include expansion; they are not recorded live account responses.
The generated included-version table covers the complete reachable Version
schema. Unsupported new constraints, external references and excessive schema
depth fail generation instead of silently extending the support claim.

`CatalogRequest::cargo_search` admits only Cargo's `q` and `per_page` parameters.
Its decoder accepts the stable `name`, `max_version`, nullable/omitted
`description`, and `meta.total` contract. `CatalogRequest::list` instead requires
the full web Crate schema and all three meta fields, including nullable
`next_page`/`prev_page`. Omit `q` for an empty search; an explicitly empty
`SearchQuery` rejects. Other query limits remain in the
[request policy](CRATESIO_REQUEST_POLICY.md).

Metadata requires `crate`, `versions`, `keywords`, and `categories`, including
explicit nullable fields. Omitted includes select upstream `full`; explicit
include selectors determine which expansion arrays must be present or null.
Default-version-only expansion must contain exactly the selected version when
one exists. Returned crate identity is bound using upstream ASCII case and
hyphen/underscore equivalence. Included versions additionally require matching
crate names, valid SemVer, positive IDs, 64 hexadecimal checksum characters,
checked timestamps and the full source-owned nested schema.

Public crate fields use the existing complete `SummaryCrate` model, re-exported
as `CrateRecord`. `IncludedVersion::fields()` exposes a schema-checked protected
JSON view, preserving unknown version fields and numeric spelling. Dedicated
version APIs and ergonomic version-domain models remain Commit 10. Other
unknown fields are fully parsed and bounded before being discarded. Crate
links are bounded inert strings, not validated routing capabilities; applications
must not use them as credential destinations or render descriptions as HTML.

## Bounds And Pagination

The [discovery limits](CRATESIO_DISCOVERY_POLICY.md#resource-limits) apply:
caller response cap up to 8 MiB, 16,384 values, 32 levels, 64 members/object,
1,024 elements/array, 65,536 decoded bytes/string and fallible reservations.
Catalog list items are additionally capped at the requested size, at most 100.
Oversized responses reject instead of returning a truncated success.

Every returned continuation preserves the exact official origin, path, filter,
sort and explicit page-size parameters. The existing `PageLink` checks cursor
advancement and disallows reusing the current seek token. Null links contradicting
known numbered continuation fail closed. Missing previous-page history rejects.
The SDK's page-10 ceiling and upstream's first-1,000 relevance matches return
`LimitReached` when further results exist. A returned page-11 link undergoes all
binding checks but is never exposed as an executable over-limit link.

Seek totals are not an offset or a snapshot guarantee. When a relevance seek
response ends with more than 1,000 total matches, it conservatively reports
`LimitReached`. A null non-relevance seek continuation is provider-reported end.
The relevance cap requires a non-empty search query; `sort=relevance` alone
uses the upstream alphabetical fallback. Seek responses cannot add previous links.
Callers validate a `CatalogLink`, inspect its `Cursor`, explicitly construct
the next typed request preserving filters, and enforce total time/page/item
budgets. There is no high-level collection crawler or implicit backtracking.

## Execution And Credentials

`CatalogClient::production` and `::staging` reuse the private discovery GET
runner. It verifies transport origin and actual configured identifying
user-agent at construction and before dispatch. Exact HTTP 200, JSON media,
response bounds, Cargo error envelopes and schema validation are inseparable
from client execution. Lower-level `decode` accepts already wire-admitted data;
it does not independently prove the origin of caller-supplied responses.

Anonymous blocking, local async and Send async calls share the process gate
with discovery: one in-flight attempt and at least one second of quiet time
after completion/error/cancellation. Retry-After may only extend the delay.
`MAX_PROVIDER_DELAY` bounds provider-requested deferral to 24 hours. A larger
delay rejects the current response with `ScheduleError::Overflow` and imposes
the capped wait; it cannot poison the process gate. This SDK policy applies
to seconds and HTTP dates, successful responses and provider errors, and the
token adapter. No retry is scheduled automatically after the wait.
The pause remains a substantial intentional availability boundary. There is
no runtime shorter-cap configuration, and production/staging share one gate.
Future isolation or shorter-cap policy must preserve aggregate API rate limits.
Storage ownership begins before dispatch/first poll, including unpolled future
cleanup. No retries, sleeps, credentials, cookies or redirects are implicit.
Separate processes sharing egress still require operator coordination.

`execute_with_token` is an explicit blocking trusted-adapter callback for
`list_crates` only. It scopes an `ApiToken` to the exact official origin, GET
target and shared gate, and clears secret scratch and response/header storage
on every exit. The callback must apply raw `Authorization` once as sensitive,
honor the supplied response policy and execute only the supplied request.
This is not a Bearer token. Ordinary anonymous raw reqwest executors do not
inject it. Built-in token transport and async authenticated-client integration
remain later work. The `following` filter fails before anonymous dispatch;
metadata and the minimal Cargo profile cannot opt into token execution.

## Verification

```sh
cargo test --locked -p cloud-sdk-cratesio --all-features
cargo test --locked -p cloud-sdk-cratesio --no-default-features --features alloc
python3 scripts/test-cratesio-catalog.py
python3 scripts/generate_cratesio_catalog.py
python3 scripts/check_cratesio_request_policy.py --fetch
scripts/check_cratesio_drift.py --fetch
scripts/check_hetzner_api_surface.sh --fetch
```

Tests cover every source fixture, include selector, stable Cargo profile,
required fields, invalid types, unknown fields, defaults, atomic targets,
literal-new routing, relevance/page limits, same-query links, response cleanup,
transport parity, cancellation, shared rate admission and token origin/route
rejection. Offline generator tests reject schema/operation mutations. The
release gate verifies generated fixtures and projection against pinned source.
Use generator `--write` only after reviewing upstream source changes.

## Local Qualification

The implementation is committed as `a9bce255`. Local qualification on
2026-09-11 passed:

- `scripts/checks.sh`, including workspace default/all-feature tests, doctests,
  Clippy, fixture fail-closed lint, module boundaries and package verification;
- the complete Rust 1.92.0 through 1.98.1 matrix and configured portable/native
  platform checks, with no live credentialed requests;
- provider default and alloc-only Clippy, independent blocking/async checks,
  and alloc-only tests on both development Rust and the MSRV;
- all six packaged feature graphs and all four SBOM freshness graphs;
- pinned catalog generation, request implementation source verification,
  live crates.io/Hetzner drift, direct dependency and Cargo-tool freshness;
- GitHub checkout freshness (`v7.0.1`), README parity and documentation links.

The all-feature provider suite has 99 unit tests, one integration test and
16 doctests. The alloc-only suite has 88 unit tests plus the same integration
and documentation tests. No dependencies, lockfiles or publication flags changed;
all six crates remain unpublished candidates. This is local implementation
evidence, not an independent pentest result or GitHub approval for Commit 9.

## Pentest Remediation

The first scan of `71e3f972..16269d3f` reported one medium and two low findings.
All three were reproduced with regression tests before correction:

- Unbounded provider delays could indefinitely disable shared API execution.
  Deferral now stores at most 86,400 seconds and returns a scheduling error for
  larger input. Boundary tests cover zero, the exact cap, cap-plus-one, maximum
  seconds/duration, and a far-future HTTP date. Client tests cover success/error
  status, blocking/local/Send execution, the token adapter and buffer cleanup.
  A deterministic test advances the real scheduler to its deadline and proves
  recovery without poisoning or sleeping for a day.
- `oneOf` sibling validation constraints were silently ignored by generation.
  The generator now rejects those combinations and requires one to eight
  branches. Regression fixtures cover type/enum/format siblings, malformed
  branch containers, empty/oversized choices and allowed annotations.
- Allocation failures in `oneOf` were misclassified as limit errors. The shared
  branch evaluator now preserves both fatal errors exactly and stops immediately.
  Deterministic branch-result injection tests allocation/limit errors before
  and after a matching branch, while preserving exactly-one-match semantics.
  This tests classification, not a global allocator failure hook.

The pinned schema projection and source fixtures remain unchanged. The user
confirmed that independent remediation retest passed with no new findings.
The permanent report combines the initial scan and remediation retest ranges;
local qualification does not substitute for the independent assessment.

Remediation verification passed `scripts/checks.sh`, including workspace tests,
Clippy, doctests, package checks and the fail-closed fixture lint. The provider
passes 104 all-feature unit tests on Rust 1.98.1 and the 1.92.0 MSRV, 90
alloc-only unit tests, one integration test and 16 doctests. Separate std-only,
blocking and async builds, generator regression/source verification, document
links, file-length checks and all four SBOM freshness graphs passed. No
dependencies, generated schema/fixtures or publication flags changed.
