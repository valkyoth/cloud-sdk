# crates.io Commit Plan

Status: selected unreleased `1.1.0` implementation train. Commits 1 and 2 are
accepted; Commit 3 is implemented and awaiting its incremental pentest.

## Decision Summary

The estimated implementation train is **22 planned commits** followed by one
service release. These are logical reviewed implementation commits, not a
promise that remediation, documentation corrections, or merge mechanics will
produce exactly 22 Git objects. Security fixes may add commits without changing
the numbered scope.

The target is every operation in the source-locked public crates.io OpenAPI
document plus exact compatibility with the stable Cargo Registry Web API
operations represented there. The SDK is a client, not a registry server or a
replacement for Cargo. It will not expose crates.io's private OpenAPI document,
browser-only session handling, undocumented routes, or bulk-data behavior that
contradicts the crates.io data-access policy.

Every numbered commit is an implementation stop. It must be locally green and
receive an incremental pentest against the preceding accepted commit before
the next numbered commit begins. The final commit additionally receives a full
service pentest and final retest before any version is selected, tagged, or
published.

## Commit Checkpoint Workflow

`Commit N` names an accepted implementation checkpoint, not one literal Git
object and not a release. Work remains on `main` throughout the train:

1. The first comparison baseline is the approved release tag when the train
   begins; for the current candidate plans that baseline is `v1.0.0`.
2. Implement the numbered scope and commit it normally.
3. Pentest the complete diff from the preceding accepted baseline to `HEAD`.
4. Commit every remediation normally and retest the same complete range until
   it is green.
5. Record the accepted `HEAD` commit hash as that numbered checkpoint's
   baseline. Do not create a tag or publish crates.
6. Start the next numbered scope and compare its eventual `HEAD` against the
   preceding accepted checkpoint hash.

Only the final qualified service candidate receives a release decision and,
if approved, a signed release tag and crate publication.

## Preliminary Survey

This assessment was refreshed on 2026-08-31 against:

- the official [crates.io OpenAPI document](https://crates.io/api/openapi.json);
- the official
  [Cargo Registry Web API](https://doc.rust-lang.org/cargo/reference/registry-web-api.html);
- the data-access policy linked by the OpenAPI document; and
- official [`rust-lang/crates.io`](https://github.com/rust-lang/crates.io)
  commit `ea3b6ebad504d9701bc41f4d2f1d32ab864cee94` as
  secondary implementation evidence.

The observed OpenAPI 3.1 document exposed 40 paths and 51 public operations.
Seven operations overlap the stable Cargo Registry Web API contract: publish,
yank, unyank, owner list/add/remove, and crate search. The remaining 44
operations are crates.io-specific and explicitly described as experimental and
subject to change. Cargo's `/me` login URL is an instruction target rather than
an API request and is not counted as an operation.

The OpenAPI document exposed three authentication schemes:

- a raw API token in the `Authorization` header;
- a bearer-format temporary trusted-publishing token; and
- a browser session cookie.

The public API offers an anonymous, API-token, trusted-publishing-token, or
one-time path-token route for every admitted operation. The SDK therefore does
not ingest or replay browser session cookies. This avoids turning a server-side
SDK into a browser-session automation surface while retaining all 51 public
operations.

The linked data-access policy requires clients to prefer the sparse index,
static crate downloads, RSS feeds, or database dumps when those sources fit the
task. Direct crates.io API use is limited to one request per second and requires
an identifying `User-Agent`. The deployed policy route is content-negotiated:
requests without an HTML `Accept` header currently return `404`, while
`Accept: text/html` returns the crates.io application. Commit 1 therefore binds
both the deployed route and the policy component in the official source
repository at the pinned commit.

The survey copies had these SHA-256 digests:

- OpenAPI JSON:
  `c9c1e39be547cca34ebde188cf3b52e66906ab23c50357ca51c766853139a17f`;
- rendered Cargo Registry Web API page:
  `2e349014e3bc95e7896f3e33ddbee2fdbb30de0a4ec58dca5dd8b1bd6f98a138`;
- deployed data-access route:
  `47cb6a1d933a908deba30caf6440b5b9cc135c220102dd15ffeccd44cdad25ab`;
- pinned OpenAPI source:
  `05319ccd6c1fa9e0e3ac9acd86fca932ccd1100358c5e395427b6322ce4d7810`;
- pinned data-access policy source:
  `db5078a31e412c395321b95124ba5e4b3db7954e1d2a66a9a5847c3cbbaca802`.

Commit 2 additionally observes the current policy file on the official `main`
branch. At this checkpoint it has the same digest as the pinned policy source;
the separate identity exists so a future policy edit becomes semantic drift.

The maintained manifest, operation matrix, and Cargo compatibility matrix are
documented in [`CRATESIO_SOURCE_LOCK.md`](CRATESIO_SOURCE_LOCK.md).

## Scope Rules

1. The service maps to one provider crate: `cloud-sdk-cratesio`.
2. Provider-neutral transport, sanitization, testkit, polling, pagination, and
   execution controls remain in existing neutral crates.
3. Default provider features remain empty and `no_std` compatible.
4. No generated source file or hand-written code file may exceed 500 lines.
5. All 51 source-locked public operations receive request, response, error,
   authentication, retry, idempotency, and authority classifications.
6. Publish, owner changes, yanking, token revocation, and account changes never
   execute implicitly.
7. Publish is non-idempotent and is never retried automatically.
8. Official crates.io endpoints are the safe default. Custom endpoints are
   explicit and never receive credentials from untrusted configuration.
9. The official high-level client enforces the current API request-rate and
   identifying-user-agent policy.
10. Cookie authentication, private routes, and undocumented frontend behavior
    are excluded even when visible in the crates.io source repository.
11. Coverage is claimed only for rows in the committed operation matrix.
12. A numbered commit cannot widen scope assigned to a later commit.

## Commit 1 - Source Lock And Finite Scope

Checkpoint status: accepted after incremental pentest, remediation, retest, and
green GitHub checks.

Goal: establish the exact crates.io and Cargo protocol support claim before
service code exists.

Deliverables: bounded retrieval of the public OpenAPI document, Cargo Registry
Web API page, deployed data-access policy, and relevant official source
evidence; exact URL, redirect, size, digest, OpenAPI version, path, method,
operation ID, stability, auth, request, response, and policy records; and a
matrix assigning every discovered row to included, deferred, excluded, or
superseded.

Verification: independently reproduce source digests; reject cross-origin
redirects, malformed JSON or HTML, duplicate operation identities, unresolved
references, unknown auth schemes, missing policy text, and unclassified rows;
prove that all 51 public operations are represented; and rebuild observations
without rewriting accepted locks.

Exit criteria: the exact public operation count and seven stable Cargo overlaps
are reviewable, no row is unclassified, the deployed access policy is resolved,
and any change to the 22-commit estimate is made here before implementation.

Pentest stop: run an incremental pentest for the exact Commit 1 source-lock and
scope-classification boundary.

## Commit 2 - Drift, Policy, And Compatibility Detection

Checkpoint status: accepted at
`17650825a067b0748399ddfbf8cb9f066bd6b66d` after incremental pentest,
remediation, green retest, and complete local verification. The next checkpoint
must use this commit as its comparison baseline.

Goal: turn upstream API, Cargo contract, and access-policy changes into
fail-closed maintenance events.

Deliverables: a crates.io adapter for the neutral provider-drift engine;
operation, parameter, schema, auth, content-type, status, and stability
fingerprints; Cargo contract fingerprints; policy fingerprints for rate,
user-agent, and preferred-data-source rules; and an explicit lock-refresh
workflow.

Verification: fixtures for additions, removals, renames, requiredness changes,
schema changes, auth changes, policy changes, stable-to-experimental conflicts,
redirects, timeouts, oversized documents, and unavailable policy pages; prove
that failed or incomplete observations never update accepted evidence.

Exit criteria: one command reports every classified drift category and cannot
silently accept a changed experimental operation or weakened access rule.

Pentest stop: run an incremental pentest for the exact Commit 2 drift and
policy-observation boundary.

## Commit 3 - Crate, Identity, And Module Boundaries

Checkpoint status: implemented on `main`; incremental pentest and GitHub checks
are required before acceptance. The comparison baseline is
`17650825a067b0748399ddfbf8cb9f066bd6b66d`.

Goal: add the service without coupling crates.io behavior to neutral core code.

Deliverables: `cloud-sdk-cratesio`; provider and service identities; empty
default features; feature ownership for models, Serde, and execution adapters;
bounded modules grouped by catalog, accounts, ownership, publishing, and
trusted publishing; README, package metadata, licensing, and docs.rs setup.

Verification: default and all-feature builds; `no_std` compile checks; external
provider-identity tests; forbidden dependency-graph checks; package-content
inspection; file-length policy; and proof that unrelated provider crates do not
depend on crates.io code.

Exit criteria: the new crate is independently consumable, contains no endpoint
implementation yet, and preserves all workspace platform and dependency
boundaries.

Pentest stop: run an incremental pentest for the exact Commit 3 crate and
feature topology.

## Commit 4 - Endpoint And Request-Target Safety

Goal: make official routing safe before credentials or operations exist.

Deliverables: constants and constructors for `https://crates.io`, staging, the
static download authority, and any source-locked trusted-publishing authority;
validated relative request targets; redirect policy; and visibly explicit
custom-endpoint constructors.

Verification: host, port, path, query, fragment, user-info, Unicode, control
byte, encoded separator, traversal, downgrade, redirect, and authority-confusion
tests; prove credentials cannot cross an authority boundary and download
redirects cannot inherit API authorization.

Exit criteria: official constructors cannot be redirected to an attacker host,
custom endpoints require an explicit unsafe-trust decision, and targets are
bounded before transport execution.

Pentest stop: run an incremental pentest for the exact Commit 4 endpoint and
redirect boundary.

## Commit 5 - Credentials And Authentication Contexts

Goal: represent each non-cookie authentication mode without accidental leaks or
header confusion.

Deliverables: protected API-token, trusted-publishing-token, OIDC assertion,
email-confirmation token, and owner-invitation token types; raw API-token versus
Bearer formatting; scoped credential application; rotation and caller-buffer
sanitization paths; and redacted diagnostics.

Verification: whitespace, CRLF, control byte, empty, oversized, duplicate
header, wrong-scheme, wrong-authority, clone, drop, rotation, source-buffer
cleanup, and Debug/Display redaction tests; compile-fail tests prevent applying
trusted-publishing tokens to unrelated operations.

Exit criteria: credentials are context-bound, never stored in ordinary owned
strings by the SDK, never appear in request-target diagnostics, and cannot be
attached to static downloads or foreign hosts.

Pentest stop: run an incremental pentest for the exact Commit 5 credential and
authentication surface.

## Commit 6 - Wire, Error, Rate, And Response Foundations

Goal: define one checked wire contract for all crates.io operations.

Deliverables: JSON success and provider-error envelopes; detection of Cargo's
error envelope even on HTTP 200; bounded response sizes; content-type policy;
`Retry-After` parsing; one-request-per-second scheduling metadata; mandatory
identifying user-agent policy; and payload-free public errors implementing
`Display` and `core::error::Error`.

Verification: status/content-type matrices; malformed, duplicate, unknown, and
oversized JSON; 200-with-errors rejection; missing or invalid `Retry-After`;
429 and 503 classification; user-agent injection; concurrent rate-limit tests;
and redacted error-chain tests.

Exit criteria: callers cannot accidentally treat a crates.io error as success,
the official client cannot exceed the source-locked API rate, and diagnostics
contain no credentials or response payloads.

Pentest stop: run an incremental pentest for the exact Commit 6 wire, error,
rate, and diagnostics boundary.

## Commit 7 - Identifiers, Queries, And Pagination

Goal: make all path and query construction canonical, bounded, and policy aware.

Deliverables: crate name, semantic version, category slug, keyword, user/team
login, numeric ID, owner, include selector, sort, query, page, per-page, and
seek-link types; percent encoding; source-locked limits; and typed pagination
that handles both full meta links and legacy `more` responses.

Verification: boundary, Unicode, reserved byte, encoded separator, traversal,
duplicate parameter, ambiguous version, integer overflow, page-depth, unknown
include, malformed next/previous link, and property tests; cross-check SemVer
behavior against the admitted Cargo grammar.

Exit criteria: no operation performs ad hoc path/query interpolation, returned
pagination links are revalidated against the official authority, and access
policy limits remain enforceable across pages.

Pentest stop: run an incremental pentest for the exact Commit 7 identifier,
query, and pagination surface.

## Commit 8 - Taxonomy And Site Discovery

Goal: implement the anonymous low-risk discovery foundation.

Deliverables: category list/detail and slug operations; keyword list/detail;
site metadata; front-page summary; complete request and response models; and
checked sync, local-async, and async execution paths.

Verification: source-derived wire fixtures, optional/null field handling,
pagination, unknown-field compatibility, malformed timestamps, oversized
collections, operation metadata, and transport-parity tests.

Exit criteria: all source-locked category, keyword, site metadata, and summary
rows are executable and independently documented.

Pentest stop: run an incremental pentest for the exact Commit 8 discovery
surface.

## Commit 9 - Crate Search And Metadata

Goal: implement crate discovery and single-crate metadata without bulk crawling.

Deliverables: crate search/list, literal `new` crate lookup, named crate lookup,
include selectors, sort and pagination models, crate links, and checked response
decoders that preserve forward-compatible fields where required.

Verification: stable Cargo search fixtures plus crates.io extensions; `new`
literal versus crate-name routing; relevance-page limits; empty and maximum
queries; include expansion; malformed links; anonymous versus optional-token
execution; and one-request-per-second workflow tests.

Exit criteria: every source-locked crate search and metadata operation is
executable without encouraging API use where the sparse index is more suitable.

Pentest stop: run an incremental pentest for the exact Commit 9 crate discovery
and metadata surface.

## Commit 10 - Versions, Dependencies, Authors, And Readmes

Goal: cover the complete version-detail graph with bounded decoding.

Deliverables: version list/detail, dependency, author, and README operations;
dependency kind, requirement, feature, target, checksum, license, links, and
publication metadata models; and explicit text-versus-JSON response handling.

Verification: SemVer and requirement edge cases, duplicate dependencies,
feature and target bounds, checksum syntax, null/omitted differences, README
content types and sizes, archived fields, unknown enum values, and transport
parity.

Exit criteria: every source-locked version, dependency, author, and README row
is executable with no unbounded allocation or lossy schema interpretation.

Pentest stop: run an incremental pentest for the exact Commit 10 version-detail
surface.

## Commit 11 - Downloads, Statistics, And Reverse Dependencies

Goal: implement artifact and usage reads without credential forwarding or bulk
API misuse.

Deliverables: version download, crate/version download-count, and reverse
dependency operations; bounded binary response streaming; redirect and checksum
hooks; static-CDN guidance; and models for time buckets and reverse dependency
pagination.

Verification: cross-authority redirect stripping, length mismatch, truncation,
oversized artifact, partial read, cancellation, caller buffer exhaustion,
malformed statistics, deep pagination, and API-versus-static-source workflow
tests.

Exit criteria: crate artifacts can be streamed without buffering the complete
body, credentials never reach the static authority, and all three read areas
are documented with their preferred data source.

Pentest stop: run an incremental pentest for the exact Commit 11 artifact,
statistics, and reverse-dependency surface.

## Commit 12 - Public Users, Teams, And Ownership Reads

Goal: complete anonymous identity and ownership inspection.

Deliverables: user lookup and statistics, team lookup, combined owner list,
user-owner list, and team-owner list operations; typed owner distinctions; and
bounded account metadata.

Verification: user/team namespace collisions, case and encoding behavior,
unknown or deleted identities, duplicate owners, private/null fields, oversized
owner lists, route construction, and response identity tests.

Exit criteria: every source-locked public user, team, and ownership read is
executable without implying mutation authority.

Pentest stop: run an incremental pentest for the exact Commit 12 public identity
and ownership surface.

## Commit 13 - Authenticated Personal Workflows

Goal: implement user-scoped actions with explicit intent and token sensitivity.

Deliverables: follow/unfollow, invitation accept/decline, tokenized invitation
acceptance, email-notification update, email confirmation/resend, and user
settings update operations; action-specific permits; and one-time token
redaction from paths and diagnostics.

Verification: wrong-user and wrong-crate binding, expired or malformed one-time
tokens, repeated acceptance, conflicting updates, email state transitions,
follow idempotency classification, destructive confirmation, and credential
scope tests.

Exit criteria: every source-locked personal workflow is executable only with
the correct credential and explicit action authority, and token-bearing paths
remain redacted.

Pentest stop: run an incremental pentest for the exact Commit 13 authenticated
personal workflow surface.

## Commit 14 - API Token Inspection And Revocation

Goal: support the complete public token-management surface without inventing an
undocumented token-creation API.

Deliverables: token lookup by ID, token revocation by ID, and current-token
revocation; scope and expiry models; explicit destructive permits; and safe
rotation guidance that separates replacement provisioning from revocation.

Verification: self-revocation, stale IDs, scope omission, malformed expiry,
replay, wrong-token authority, retry prohibition, redacted responses, and
compile-fail tests requiring destructive authorization.

Exit criteria: all three source-locked token operations are executable, token
creation remains explicitly out of scope, and revocation cannot be retried or
triggered implicitly.

Pentest stop: run an incremental pentest for the exact Commit 14 API-token
management surface.

## Commit 15 - Crate And Version Settings Mutations

Goal: implement metadata changes without ambiguous partial-update behavior.

Deliverables: crate settings update and version settings update operations;
typed patch fields; validation for empty and conflicting patches; mutation
permits; and checked postcondition models.

Verification: omitted versus null fields, archived state, description and URL
bounds, version identity mismatch, no-op updates, concurrent modification,
retry classification, wrong crate scope, and response validation.

Exit criteria: both source-locked settings operations require explicit mutation
authority and cannot silently clear fields or target another crate/version.

Pentest stop: run an incremental pentest for the exact Commit 15 crate and
version settings surface.

## Commit 16 - Ownership Mutations

Goal: implement owner changes while preventing accidental lockout and identity
confusion.

Deliverables: Cargo-compatible add/remove owner requests; crate-bound owner
identities; mutation and destructive permits; invitation-result models; and
optional preflight hooks for current ownership state.

Verification: empty and duplicate owner lists, user/team ambiguity, self and
last-owner removal policy, pending invitations, partial failures, retries,
wrong-crate scopes, 200-with-errors, and exact Cargo wire fixtures.

Exit criteria: add and remove owner operations match both source locks, removal
is never implicit or automatically retried, and contradictory provider results
fail closed.

Pentest stop: run an incremental pentest for the exact Commit 16 ownership
mutation surface.

## Commit 17 - Yank And Unyank

Goal: implement reversible publication visibility changes with exact Cargo
compatibility.

Deliverables: yank and unyank requests; crate/version binding; mutation permits;
idempotency and retry classifications; success-envelope checks; and workflows
that expose the resulting state rather than assuming it.

Verification: method/path golden tests, malformed versions, repeated actions,
wrong crate scope, stale state, 200-with-errors, contradictory `ok` values,
timeouts after send, and Cargo-compatible fixtures.

Exit criteria: both source-locked operations are executable, require explicit
authority, and never infer success solely from an HTTP status.

Pentest stop: run an incremental pentest for the exact Commit 17 yank and
unyank surface.

## Commit 18 - Publish Metadata And Binary Framing

Goal: implement the stable Cargo publish protocol without reimplementing Cargo
packaging or making publish retryable.

Deliverables: complete source-locked publish metadata and warning models;
dependency, feature, target, license, link, and Rust-version validation; exact
little-endian length framing; caller-buffer and streaming package sources;
overflow and size limits; publish permit; and API-token and trusted-publishing
credential variants.

Verification: byte-for-byte Cargo fixtures; zero, boundary, truncated, and
overflowed lengths; malformed metadata; duplicate and renamed dependencies;
invalid features/targets/licenses; archive read failure; cancellation; timeout
after send; warning decoding; no automatic retry; and secret cleanup.

Exit criteria: a caller-provided `.crate` artifact can be published through the
exact stable protocol, package bytes need not be fully duplicated in memory,
and no API can publish without explicit one-shot authority.

Pentest stop: run an incremental pentest for the exact Commit 18 publish and
binary-framing surface.

## Commit 19 - Trusted Publishing

Goal: support the complete source-locked GitHub and GitLab trusted-publishing
surface without treating OIDC assertions as ordinary text.

Deliverables: list/create/delete GitHub configurations; list/create/delete
GitLab configurations; OIDC assertion exchange; temporary token revocation;
repository, workflow, environment, namespace, project, and crate-bound models;
and temporary-token lifetime and erasure policy.

Verification: issuer/audience/provider mix-ups, repository and namespace
confusion, wildcard rejection, missing claims, oversized assertions, malformed
responses, expired temporary tokens, revoke-after-publish, wrong credential
scheme, duplicate configuration, deletion authority, and redaction tests.

Exit criteria: all eight source-locked trusted-publishing operations are
executable, temporary credentials cannot escape their context, and the SDK does
not claim to authenticate or cryptographically validate an external OIDC token.

Pentest stop: run an incremental pentest for the exact Commit 19 trusted
publishing surface.

## Commit 20 - Unified Client And Cargo Compatibility

Goal: make the checked path the easiest path for every admitted operation.

Deliverables: official crates.io client constructors; operation-to-prepared
request bindings for all 51 rows; automatic method, target, headers, body,
response bound, and decoder selection; sync, local-async, async, raw, and
streaming execution parity; and higher-level Cargo-compatible publish, owner,
yank, unyank, and search workflows.

Verification: generated coverage assertions against the operation matrix;
compile-checked examples; credential and permit routing; shared concurrent rate
limiting; cancellation; partial transport writes; checked response decoding;
and byte-for-byte comparison with Cargo's seven stable contracts.

Exit criteria: no supported operation requires manual HTTP assembly, all 51
matrix rows execute through the official client, and the seven stable Cargo
operations have independently verified compatibility evidence.

Pentest stop: run an incremental pentest for the exact Commit 20 unified client
and Cargo-compatibility surface.

## Commit 21 - Live Evidence, Fuzzing, And Platform Qualification

Goal: produce current adversarial and platform evidence without granting CI
publication or account-mutation authority.

Deliverables: anonymous and least-scope live read harnesses; mock-only mutation,
publish, ownership, yank, token, and trusted-publishing staging; fuzz targets for
OpenAPI drift, paths, queries, pagination, JSON, publish framing, redirects,
tokens, and downloads; portable-target checks; SBOMs; and package verification.

Verification: full fuzz build and bounded campaigns, deterministic regression
corpora, no-secret CI proof, one-request-per-second live scheduling, identifying
user-agent checks, MSRV/stable/platform matrices, dependency review, advisory
checks, file-length policy, package contents, fresh SBOMs, and reproducible
archives.

Exit criteria: every public support, policy, platform, dependency, and live
claim has executable evidence, and CI cannot publish, mutate, revoke, yank, or
change ownership on crates.io.

Pentest stop: run an incremental pentest for the exact Commit 21 qualification
surface.

## Commit 22 - Scope Freeze And Release Candidate

Goal: freeze and qualify the complete selected crates.io integration without
adding features.

Deliverables: final 51-operation matrix; exact stable/experimental
classification; zero unclassified or model-only rows; provider README and
examples; threat model; authentication, access-policy, retry, mutation,
publishing, live-test, drift, deprecation, migration, and platform documentation;
release notes; provenance; and one candidate gate composing all prior gates.

Verification: rerun all 22 commit gates, live source and policy drift, full
workspace and provider tests, every execution mode, Cargo compatibility,
fuzz/adversarial suites, MSRV/platform matrices, dependency and SBOM checks,
public API and SemVer review, package reproduction from two clean clones, and
green GitHub CI and CodeQL.

Exit criteria: all 51 Commit 1 rows are executable and documented; every
excluded surface has a precise reason; no API, dependency, feature, or scope
change occurs after qualification; and the candidate can receive a version only
after a separate release decision.

Pentest stop: run a full-service pentest for the exact Commit 22 candidate,
remediate and retest every finding, rerun the complete release gate, then wait
for green GitHub CI and CodeQL before selecting a version, signing a tag, or
publishing crates.

## Deferred Surfaces

Commit 1 records exact exclusions, but the following are presumed deferred:

- crates.io private OpenAPI operations and undocumented backend routes;
- browser session-cookie ingestion and web-frontend session automation;
- implementing a Cargo registry server, sparse index server, or mirror;
- creating or packaging `.crate` archives from a source workspace;
- legacy Git index cloning and sparse-index bulk synchronization;
- RSS and database-dump parsers;
- automatic publication retries or unattended mutation workflows;
- bypassing the official API rate or identifying-user-agent requirements; and
- any public operation added after the Commit 1 source lock.

Deferral is not permanent rejection. A later release can add a source-locked
surface through a separate commit plan after its protocol, security, policy,
and maintenance costs are reviewed.

## Release Decision

This document deliberately does not name a release version. After Commit 22
passes its full-service pentest, complete release gate, GitHub CI, and CodeQL,
maintainers decide whether the accumulated compatible workspace changes warrant
a minor workspace release or another SemVer version. The
`cloud-sdk-cratesio` crate receives its own independently appropriate package
version under the post-1.0 workspace versioning policy.
