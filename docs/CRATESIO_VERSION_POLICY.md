# crates.io Version Contract

Status: unreleased `1.1.0`, logical Commit 10 accepted after pentest and GitHub.
Compare against accepted checkpoint `38d493a17c6741691e676be16fa6ac341ed28642`.
Stop before Commit 11. No tag or publication is authorized.

## Executable Scope

| Source operation | Request | Success |
| --- | --- | --- |
| `list_versions` | `VersionRequest::list` | checked records, complete meta, explicit seek continuation |
| `find_version` | `VersionRequest::detail` | exact crate/version-bound record |
| `get_version_dependencies` | `VersionRequest::dependencies` | bounded dependency records |
| `get_version_authors` | `VersionRequest::authors` | deprecated empty compatibility result |
| `get_version_readme` | `VersionRequest::readme` | JSON URL object, not static HTML |

Each operation executes anonymously through `VersionClient` with blocking,
local async and Send async parity. Production and staging constructors verify
the fixed official authority and identifying user agent. The shared checked
GET runner enforces one attempt, exact status 200, JSON content type, complete
bounded parsing, response/header erasure and the process-wide scheduling gate.
No new transport, authentication, retry or dependency boundary is introduced.
Separately, this checkpoint admits the rustls `0.23.45` security patch for the
existing optional neutral transport, with unchanged features and graph edges.
Its lockfiles, SBOMs and boundary gates are part of the same pentest range.
The source is the [locked API](CRATESIO_SOURCE_LOCK.md); implementation details
are additionally pinned by `scripts/check_cratesio_request_policy.py --fetch`.

## Pagination

The pinned [version-list controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/krate/versions.rs)
only paginates when `per_page` is present and rejects numeric pages. Therefore
`VersionRequest::list` requires an explicit `Parameter::PerPage` (1 through
100) and rejects `Parameter::Page`, even though OpenAPI lists it as a shared
parameter. `PerPage::DEFAULT` supplies ten entries. No unpaginated high-level
mode is exposed. Include, sort and exact `nums[]` selectors remain explicit.

Every full page must carry a seek link, including a full final page: the source
does not look ahead. A subsequent empty page with total zero ends traversal.
Totals are checked for internal consistency but do not provide snapshot
isolation. Links preserve authority, route, all filters and page size; page
cursors, repeated seek tokens and changed selectors are rejected. Returned
links are inert and need explicit use with the existing pagination limits and
cycle tracking. The SDK never bulk-enumerates the registry automatically.

## Models And Bounds

The source-generated validator checks complete version/dependency schemas,
including nested publication/audit data, feature mappings, nullable historical
fields, licenses, links, checksum, edition and unstable trust-publishing or
line-count fields. Missing required values differ from explicit null. Unknown
fields remain bounded protected values; known union rules fail closed.
Version records retain all fields, and `DiscoveryValue` exposes text only in
closures with redacted Debug. Crate names use canonical case/hyphen comparison;
exact SemVer identity includes build metadata. Checksums require 64 hex digits;
they are metadata, not evidence that an artifact was downloaded and verified.

Dependency IDs must be positive and unique; all records in a response must
agree on their version ID. There is no numeric version ID in the route to
independently prove that ID against without a second request, which is not
performed implicitly. Repeated crate names across distinct records are legal.
Unknown dependency kinds are exposed as `Unknown`, with the spelling retained.
Requirements are bounded, nonempty inert strings rather than parsed Cargo
requirements: future syntax is not silently normalized or executed. The
existing dev-only `semver` oracle tests representative requirements and exact
version grammar; no parser dependency enters the runtime graph. Sparse index
data remains the preferred source for dependency resolution.

The existing parser bounds remain: 8 MiB maximum response (lowerable), 16,384
model nodes, 32 builder frames, 1,024 entries per array, 64 object fields,
256-byte keys and 65,536-byte strings. Schema traversal is capped at 24 levels.
Version pages additionally enforce `per_page`; feature names/values are capped
at 256 bytes, requirements and targets at 4,096 bytes, kinds at 64 bytes, and
README URLs at 8,192 bytes. Protected staging uses admitted sanitization with
fallible growth. Process-abort cleanup remains outside the threat model.

## Authors And README

The [authors controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/version/authors.rs)
returns `users: []` and `meta.names: []` for deprecated compatibility. OpenAPI
omits its success schema; the SDK explicitly checks that controller contract
instead of pretending the operation produces current author identities.

The [README controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/version/readme.rs)
offers a negotiated JSON URL and a default 302 redirect. The client selects
JSON with `Accept: application/json`; HTML/plain-text responses and redirects
are rejected, not parsed as JSON or followed. Static README HTML is separate
untrusted content. `ReadmeLocation` does not fetch, render, sanitize HTML, or
authorize its URL; callers must apply their own destination and rendering
policy without forwarding API credentials. This checkpoint implements the
API operation, not an HTML browser. The subsequent artifact checkpoint owns
binary download workflows.

## Verification And Stop

Regression coverage includes golden schema fixtures, required/null fields,
identity and checksum rejection, duplicate IDs, exact filters, feature and
target bounds, unknown kinds, requirement retention, seek-only continuation,
HTML/redirect rejection, all three transport modes, cancellation, rate-delay
overflow and cleanup. Schema-generator and feature-guard regression tests run
in `scripts/checks.sh`. Source fixture regeneration is checked with:

```console
python3 scripts/generate_cratesio_versions.py
python3 scripts/check_cratesio_request_policy.py --fetch
python3 scripts/check_cratesio_source_lock.py --fetch
python3 scripts/check_cratesio_drift.py --fetch
```

The independent pentest must include the shared schema-validator refactoring
and source refresh as well as the new version surface. Stop for the user's
review; local verification does not substitute for pentest or GitHub approval.

## Dynamic Metadata Inspection

`DiscoveryValue::visit_fields` enumerates returned object names and values,
including feature maps and release tracks whose keys callers do not know in
advance. Names remain closure-scoped protected borrows. The visitor allocates
no copies, preserves redacted diagnostics, rejects non-objects, and stops on
the first callback error without changing it. Empty objects invoke no callback.
Copies explicitly created by callers are caller-owned and require their own
cleanup policy. External-consumer regression tests cover these contracts.
The [independent retest](../security/pentest/cratesio-commit-10.md) closed
checkpoint 10 finding F1 at `94c004b2` with no new findings.

## Local Qualification 2026-09-15

Implementation checkpoint: `6db21a77`. Local qualification passed:

- `scripts/checks.sh`: full repository suite, default/all-feature workspace
  tests, doctests, warning-denied Clippy, packaging and security-policy gates.
- Provider all-feature tests: 116 unit tests and one identity integration test;
  alloc-only configuration: 99 unit tests and the integration test. The updated
  README and compile-fail contracts supply 17 doctests.
- All 12 supported Rust versions from 1.92.0 through 1.98.1, with workspace
  all-target/all-feature checks against the updated rustls pin.
- All ten configured portable compile targets and native Linux transport tests.
  Cross-compilation is not native test evidence for other operating systems.
- All six packaged dependency graphs, packaged transport test compilation,
  and all-feature rustdoc with warnings denied.
- All 35 existing fuzz targets built and passed 64-iteration smoke campaigns.
  This is bounded existing-harness evidence, not exhaustive version fuzzing;
  the dedicated provider fuzz checkpoint remains planned.
- Fresh RustSec scans of all four lockfiles (1,246 advisories), configured
  cargo-deny checks on the root and both changed secondary graphs, and complete
  freshness checks for all four SBOMs.
- Live crates.io source/semantic drift checks, catalog/version fixture
  regeneration and all 12 pinned request/controller source digests. Hetzner
  Cloud/Storage, Robot and changelog checks reported no drift.
- Direct dependency pins and Cargo security/fuzz tools are current after the
  rustls patch. Documentation links, review digests, release-plan structure,
  README parity and candidate release metadata passed.

The initial sandboxed full-suite attempt could not create loopback test
servers; it was rerun successfully with loopback access. Final qualification
was repeated after the TLS update. No credentialed API probe or live mutation
was performed. All six candidate publication flags remain false. Pentest must
cover `38d493a17c6741691e676be16fa6ac341ed28642..HEAD`, including this evidence
update. Do not start Commit 11 until pentest/retest and GitHub are green.
