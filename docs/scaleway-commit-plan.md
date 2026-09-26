# Scaleway Commit Plan

Status: full public Scaleway API implementation roadmap, refreshed 2026-09-26.
No implementation checkpoint has started and no release version is assigned.
This document supersedes the earlier GA-only 30-checkpoint assessment and the
historical version-based Scaleway sketch in RELEASE_PLAN.md.

## Decision And Completeness Claim

The target is **full documented public Scaleway API coverage**, not a selected
GA product subset. The working estimate is **80 numbered commit checkpoints**
followed by one qualified provider release. A checkpoint is a logical review
unit, not one Git object: remediation commits do not advance the checkpoint.

Scope includes current public GA, beta, and alpha control-plane interfaces,
plus Scaleway-supported public HTTP data-plane interfaces: S3, SQS/SNS,
registry distribution, Cockpit HTTP APIs, and Generative APIs. Track labels
remain visible in the Rust API and documentation. A stable SDK version does
not upgrade an upstream alpha/beta stability guarantee.

"Full" means every applicable public operation and its supported fields in
the accepted source inventory, not every AWS, OpenAI, OCI, or observability
upstream feature. Scaleway's compatibility restrictions take precedence.
Private/internal/test/console-only routes, removed or superseded interfaces,
and unrelated application client protocols are not silently included.

Every product in the public catalog must have an explicit inventory disposition.
A difficult public interface cannot be deferred merely to finish this train.
Unknown public status blocks scope approval; unavailable credentials limit live
evidence, not implementation coverage. A genuine reduction of the full target
requires a maintainer-approved plan change and a narrower published claim.

The count may grow after inventory review. Add or split checkpoints before
implementing newly discovered work; never compress unfinished scope into the
final qualification checkpoint.

## Commit Checkpoint Workflow

Work remains on main. The initial accepted baseline is v1.1.0; record its full
commit hash when starting. If another release intervenes, explicitly approve
and record the replacement baseline.

1. Implement only the numbered scope and commit normally.
2. Run the checkpoint gate and pentest the entire range from the preceding
   accepted baseline to HEAD.
3. Fix findings with regression tests, commit, and retest the same complete
   range until accepted. Never silently advance the pentest baseline.
4. Record the accepted review range, result, implementation/remediation hashes,
   evidence, and residual boundaries. Wait for green GitHub CI and CodeQL on
   the final checkpoint commit. CI fixes are tested, documented, and retested
   when they change reviewed security behavior.
5. Record that final accepted hash as the next checkpoint baseline.
   No intermediate tags or crates.io publication.
6. Start the next numbered checkpoint only after the preceding stop is accepted.

The final checkpoint additionally receives a full-provider and affected-neutral
code review, remediation retest, complete release gate, and green GitHub CI and
CodeQL. Version selection, signed tagging, pushing, and publishing require the
maintainer's explicit release approval; finishing implementation grants none.

## Survey Evidence And Source Discrepancies

The 2026-09-26 review inspected the official API documentation build v1.8872.0,
downloaded and parsed 90 schema variants across 84 documentation routes, and
compared the official Go SDK at commit
684f67323db64f059ef53f4c53a970c0b1591a1d. The schemas contained 1,513 raw HTTP
operation entries; 49 stable-looking schema variants contained 752 entries.

These are discovery counts, NOT admitted support counts. They include
non-public candidates and multiple tracks, and omit separately documented
compatibility protocols. The previous August count of 583 is no longer a
sizing baseline, and is not an identical-scope growth comparison. No
authenticated live service testing was performed for this survey. Temporary
survey downloads are not the production source lock.

Commit 1 must reproduce the control-plane inventory; Commit 2 must complete
the separate protocol inventory. Source URLs, bytes, digests, observation time,
documentation build, provenance, parser versions, and discrepancies become
committed evidence. Discovery through documentation JavaScript is a hint only:
do not execute downloaded code or equate a bundled route with a public API.

| Discovery | Required resolution and implementation owner |
| --- | --- |
| Public Account API includes project operations, including delete-with-resources | Commit 14 explicitly implements projects, contracts, and annotations. Classify auxiliary organization/user/login/signup schemas individually; a v3 path is not proof of public support. |
| Public Billing docs show v2beta1; the pinned SDK has v2 budgets/electronic addresses, while the public v2 schema URL returned 404 | Commit 1 resolves public support and source authority. Commit 17 covers confirmed public versions separately, including beta invoicing. Missing source evidence cannot be papered over with a guessed v2 route. |
| Instance defaults to v2alpha1 while v1 remains documented | Inventory both and retain current non-superseded contracts with versioned identities; Commits 18-20 assign every unique operation and field. |
| Public Gateway v2 removed DHCP objects and entries | Commit 32 implements v2 IPAM associations and SSH bastion allowed-IP rules; no v1 fallback or invented DHCP endpoints. |
| Elastic Metal networking exposes v1 and v3 | Commit 1 establishes supported versus superseded versions; Commit 38 implements all current assigned rows. |
| RDB encryption-key reads and VPC NIC/next-hop views exist in the bundle | Resolve public status before assigning them to Commits 46 or 30-31. Do not assume private status merely because navigation omits them. |
| Web Hosting has multiple sub-APIs | Commits 60-61 cover all public hosting, offers, backup, database, DNS, free-domain, FTP, mail, website, and confirmed auxiliary operations. |
| Generative APIs use a separate authority and include non-JSON protocols | Commits 71-74 separate control plane, synchronous JSON, multipart/batches, and SSE; include exact compatibility restrictions. |
| IAM, DNS, and many products use pre-GA interfaces | Included in this full plan, with visible stability labels, source-locked semantics, explicit permissions, and no promise of upstream stability. |
| Documentation contains fake/test, console, and unauthenticated account schemas | Classify public contract evidence, not merely URL reachability; exclude private/testing routes with reasons. |
| Public compatibility APIs live outside the OpenAPI index | Commit 2 inventories S3, messaging, registry, Cockpit and AI specifications, supported-parameter tables, prose restrictions, and any further documented public HTTP surface. |

Primary sources for discovery and ongoing maintenance:

- [Scaleway API index](https://www.scaleway.com/en/developers/api/)
- [Official Go SDK at the reviewed revision](https://github.com/scaleway/scaleway-sdk-go/tree/684f67323db64f059ef53f4c53a970c0b1591a1d)
- [Account and public project scope](https://www.scaleway.com/en/developers/api/account/project)
- [Billing](https://www.scaleway.com/en/developers/api/billing) and
  [pinned v2 SDK](https://github.com/scaleway/scaleway-sdk-go/blob/684f67323db64f059ef53f4c53a970c0b1591a1d/api/billing/v2/billing_sdk.go)
- [Instance v1](https://www.scaleway.com/en/developers/api/instance/v1)
- [Public Gateway v2 migration](https://www.scaleway.com/en/docs/public-gateways/reference-content/understanding-v2/)
- [Elastic Metal private networking](https://www.scaleway.com/en/developers/api/elastic-metal/private-network)
- [Web Hosting](https://www.scaleway.com/en/developers/api/webhosting/hosting)
- [IAM](https://www.scaleway.com/en/developers/api/iam)
- [Supported S3 operations](https://www.scaleway.com/en/docs/object-storage/api-cli/using-api-call-list/)
- [SQS supported actions and parameters](https://www.scaleway.com/en/docs/queues/reference-content/queues-support/)
- [SNS supported actions and parameters](https://www.scaleway.com/en/docs/topics-and-events/reference-content/topics-and-events-support/)
- [Cockpit supported HTTP endpoints](https://www.scaleway.com/en/docs/cockpit/reference-content/cockpit-supported-endpoints/)
- [Generative APIs and restrictions](https://www.scaleway.com/en/developers/api/generative-apis)

## Architecture And Non-Negotiable Boundaries

- Exactly one provider crate: cloud-sdk-scaleway. Reusable transport,
  sanitization, signing integration, and testkit capabilities stay neutral.
- Reuse the released 1.1.0 prepared-request, execution, streaming, credential,
  parser, permit, pagination, task, and drift infrastructure. Do not rebuild it.
- Empty default features, no_std-first core/provider, optional allocation and
  protocol/transport dependencies with explicit admission and portable targets.
  Heavy protocol adapters must not leak into the default dependency graph.
- No code file exceeds 500 lines, including generated files and tests. Split
  by domain and wire responsibility, not by creating per-product crates.
- Preserve existing public APIs and all Hetzner/crates.io behavior. Neutral
  extensions require SemVer checks and cross-provider regression evidence.
- Protected secret ownership uses cloud-sdk-sanitization and its admitted
  sanitization dependency. Use admitted base64-ng when needed, not ad hoc
  codecs or a substitute secret-erasure dependency.
- Crypto primitives, XML/protobuf/compression codecs, and protocol engines
  require reviewed dependencies or proven existing helpers; no improvised
  cryptography. FIPS remains deferred until the separate Brynja qualification.
- Global, regional, and zonal locality is not the same as HTTP authority.
  Derive control-plane paths under api.scaleway.com and separately bind
  documented data-plane authorities, partitions, ports, audiences, and keys.
- Unknown response fields/enums can be preserved where safe, but cannot grant
  capabilities, authorize requests, choose destinations, or imply task success.
- No implicit mutation, order, token creation, destructive action, retry,
  cross-origin redirect, remote URL retrieval, or credential discovery.
- IAM permissions remain server-enforced. Local permits express caller intent;
  they do not prove remote authorization or current resource state.
- Current alpha/beta versions stay versioned and documented. Migration to a
  replacement is explicit, never an automatic retry or fallback.

## Common Deliverables And Acceptance Contract

Every checkpoint supplies an executable gate, tests for its scope, documentation,
and a record of its predecessor and accepted final hashes. Full release
qualification composes these gates. Fuzzing and adversarial tests start with
each parser or protocol change, not at the final fuzz checkpoint.

Run applicable formatting, warning-denied Clippy, workspace/default/alloc/all-
feature tests, doctests, examples, file-length policy, dependency-boundary
checks, and existing-provider regressions. Record non-applicable checks rather
than returning success before exercising assertions. Keep toolchain, dependency,
and GitHub action updates explicit and independently reviewed.

The coverage ledger records, per operation: public-source evidence, product,
version/track, method/path/authority, all parameters and body fields, status and
media variants, response/error models, auth/permission requirements, pagination,
cost, risk, retries, idempotency, delivery/reconciliation, streaming behavior,
three execution modes, tests, examples, and owning checkpoint.

Completeness includes optional/nested fields, null-versus-absent PATCH intent,
raw bodies, headers, formats, variants, and prose-only restrictions. Shared
code counts only through checked associations to every consuming operation.
"Model exists", a generic JSON escape hatch, or a fixed endpoint count is not
evidence of executable field-level support.

Every product checkpoint implements blocking, Send-async, and local-async
client paths using the existing core. Before final workflow aggregation, each
operation is already typed, prepared, transported, and checked on response.
Stage cross-domain dependencies explicitly; do not execute a future
checkpoint's operations through a temporary generic route.

Live tests are locally opted in and least-privileged; CI never receives
mutation credentials. Read-only still needs an operation allowlist: billable
reads, dequeue/visibility changes, secret downloads, and one-time URLs are not
ordinary inventory reads. Real mutations require separate user approval,
isolated resources, cost limits, cleanup and reconciliation instructions.
Unavailable live evidence is a disclosed limitation, never fabricated proof.

## Checkpoint Index

| Checkpoints | Work |
| --- | --- |
| 1-4 | Complete public inventory, protocol sources, field coverage, drift |
| 5-13 | Crate layout, routing, credentials, authentication, codecs, policy |
| 14-17 | Account, IAM, billing and commercial APIs |
| 18-29 | Compute, storage, S3, registry control and distribution |
| 30-45 | Networking, bare metal, Kubernetes and serverless |
| 46-57 | Databases, analytics, messaging control and compatible HTTP APIs |
| 58-63 | IoT, DNS/domains, Web Hosting, email/mailbox |
| 64-70 | Observability, audit, footprint, secrets, KMS and quantum |
| 71-74 | Generative AI control, JSON, multipart/batch and streaming |
| 75-80 | Workflows, completeness audit, live/fuzz/platform evidence and release |

## Commit 1 - Public Control-Plane Inventory

Goal: Identify every documented public Scaleway control-plane contract.

Deliverables: Bounded retrieval of the index, nested navigation, all versions and
schemas; source digests, public/private evidence, deprecations, supported-version
decisions, and ownership assignments. Resolve Billing and auxiliary-schema
discrepancies; record unresolved candidates explicitly.

Verification: Reproduce digests; reject duplicate YAML keys, unsafe tags, unbounded
aliases/depth, unreviewed references, cross-origin redirects, malformed versions, and
incomplete discovery. Enforce per-source and aggregate byte limits, whole-fetch
deadlines, bounded workers and process cleanup. Compare the pinned SDK without
treating it as sole public-support proof.

Exit criteria: Every discovered control-plane row has a supported, superseded, private,
or unresolved disposition; no unresolved public candidate may pass this gate. Necessary
checkpoint splits are approved.

Pentest stop: Run the incremental pentest for the complete Commit 1 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 2 - Public Data-Plane Inventory

Goal: Complete discovery beyond the central OpenAPI documentation.

Deliverables: Source-lock Scaleway's S3, SQS/SNS, registry distribution, Cockpit and AI
operation/parameter support tables, protocol versions, endpoint/auth policies, and
restrictions. Discover additional documented public HTTP surfaces and assign owners; pin
underlying standards separately from provider support.

Verification: Cross-check product documentation and compatibility tables; detect
unsupported upstream features, missing protocol rows, contradictory prose/schema
behavior, and partial source retrieval.

Exit criteria: The finite full public inventory covers control and data planes. No
public product is excluded merely because it is alpha/beta, lacks OpenAPI, or requires a
different protocol.

Pentest stop: Run the incremental pentest for the complete Commit 2 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 3 - Field-Level Coverage Ledger

Goal: Make omissions detectable before model generation.

Deliverables: Generate stable operation and field identities, per-checkpoint
assignments, request/response/error/auth/media dimensions and three-mode execution
links. Record supported, provider-unsupported, superseded, and private reasons
independently from implementation status.

Verification: Mutation fixtures add optional/nested fields, rename paths, change
response statuses, remove enum cases, and create duplicate identities. Confirm missing
implementations fail even with unchanged operation totals.

Exit criteria: Every in-scope field and operation has exactly one implementation owner,
and incomplete rows cannot appear as supported.

Pentest stop: Run the incremental pentest for the complete Commit 3 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 4 - Drift And Source Refresh

Goal: Provide one maintainable offline verifier and one explicit live drift command.

Deliverables: Extend the neutral drift engine with Scaleway discovery,
source/schema/policy fingerprints, version and changelog observations, human-readable
differences, and reviewed lock-refresh instructions. Never rewrite locks during
observation.

Verification: Exercise additions, removals, schema compositions, defaults, requiredness,
nullability, bounds, media/auth/locality changes, docs-only restrictions, 404s, slow
reads and conflicting SDK evidence.

Exit criteria: Failed fetches and unresolved discrepancies fail closed; new services and
fields create review work. Reformat/build churn is distinguished from semantic changes
without silently accepting either.

Pentest stop: Run the incremental pentest for the complete Commit 4 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 5 - Provider Crate And Feature Layout

Goal: Introduce one publishable Scaleway crate without changing existing provider
behavior.

Deliverables: Empty defaults, no_std layout, explicit alloc/Serde/protocol/transport
features, service/version identities, small modules, docs.rs metadata, license, package
selection, SBOM and governance entries.

Verification: Default/all-target dependency graphs, MSRV/portable checks, isolated
packaging, feature combinations, file lengths and external-provider conformance.

Exit criteria: No provider-specific transport/testkit/sanitization crate is created and
no optional native or runtime dependency reaches defaults.

Pentest stop: Run the incremental pentest for the complete Commit 5 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 6 - Authorities And Resource Identity

Goal: Bind resource scope and destination before any credential use.

Deliverables: Typed projects, organizations, resource IDs, SRNs, service-specific
regions/zones and versions; official endpoint derivation; explicit custom endpoints and
data-plane endpoint provenance policy.

Verification: Path injection, escaped separators, IDNA/IP confusion, port/scheme
changes, longest targets, unknown locality, cross-project/service/audience mismatch and
returned-URL substitution.

Exit criteria: Every request has a classified origin and locality; discovered URLs
cannot automatically receive credentials.

Pentest stop: Run the incremental pentest for the complete Commit 6 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 7 - Credentials And Secret Lifecycle

Goal: Support each credential family with explicit ownership and rotation.

Deliverables: Protected X-Auth-Token, bearer, access/secret key pairs and product-issued
tokens; generation-scoped attempts, expiry/rotation, one-time outputs, caller-buffer
ingestion and redacted errors. Preserve distinct credential families.

Verification: Invalid bytes, source erasure, clone/drop/rotation, concurrent snapshots,
revoked generation, scope mismatch and debug/serialization redaction.

Exit criteria: Secret copies and cleanup boundaries are documented; no safe happy path
requires an ordinary unprotected owned secret.

Pentest stop: Run the incremental pentest for the complete Commit 7 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 8 - X-Auth-Token Transport Integration

Goal: Prove Scaleway's main authentication header end to end before product expansion.

Deliverables: Official control-plane execution using protected named-header injection,
prepared policy, checked response handling, shared client instances and
blocking/Send/local parity. Extend neutral adapters only where necessary.

Verification: Loopback fixtures inspect exact header/name, single injection, origin
binding, concurrent rotation, timeouts, cancellation, redirect refusal and cleanup after
failures.

Exit criteria: A source-locked read operation can run through the real adapters without
a bearer-header workaround or manual raw assembly.

Pentest stop: Run the incremental pentest for the complete Commit 8 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 9 - Signature V4 And Signed Requests

Goal: Implement reviewed signing support for the documented compatible services.

Deliverables: Admitted crypto-backed canonical request construction, service/region
scope, query/header signing, payload hashes, presigning and supported
streaming-signature variants. Caller-supplied time and expiry limits remain explicit.

Verification: Independent known-answer vectors, duplicate headers/query keys, canonical
URI edge cases, wrong service/date/region, body substitution, clock boundaries and
signing-key cleanup.

Exit criteria: Every required signing mode has independent evidence; unsupported
provider modes fail explicitly and signatures never grant broader scope than the
prepared operation.

Pentest stop: Run the incremental pentest for the complete Commit 9 range, remediate and
retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 10 - Data-Plane Authentication And Transfers

Goal: Make separate hosts and large payloads safe using neutral transports.

Deliverables: Origin-bound bearer/basic/product credentials, challenge handling where
documented, bounded upload/download and transactional sink integration, explicit content
encoding, deadlines and endpoint validation. Product-specific restrictions remain in the
provider.

Verification: Host/challenge substitution, credential forwarding, upload partial writes,
early responses, truncation, cancellation, byte limits, decompression limits and DNS
timeout/resource bounds.

Exit criteria: Supported transfer/auth combinations work in all modes without weakening
existing no-redirect and delivery-phase guarantees.

Pentest stop: Run the incremental pentest for the complete Commit 10 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 11 - Wire Formats And Bounded Decoding

Goal: Provide shared codecs needed by the complete protocol inventory.

Deliverables: Bounded JSON, XML/query, multipart, SSE framing and required binary codec
admission; protobuf-style numeric strings, money, timestamps/durations, empty responses,
unknown variants, and payload-free Display/Error implementations.

Verification: Duplicate keys/elements, XML external entities, namespace confusion, deep
structures, oversized parts/events, invalid UTF-8, numeric precision, allocation
failures and malformed codec frames.

Exit criteria: Each admitted wire variant has an implemented bounded path or assigned
protocol-specific extension before its product checkpoint; no unbounded generic parser
is exposed.

Pentest stop: Run the incremental pentest for the complete Commit 11 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 12 - Pagination And Rate Policy

Goal: Handle provider-specific iteration and throttling consistently.

Deliverables: Page/per-page/page-size, cursor/token/header totals, continuation policy,
rate/retry metadata, caller-bounded scheduling and service/account isolation. Keep no
implicit retries as the default.

Verification: Stalled or repeated cursors, conflicting totals, malicious continuation
URLs, exhausted page/byte budgets, Retry-After overflow, cancellation and independent
rate gates.

Exit criteria: Every paginated/throttled operation maps to a source-defined variant with
bounded progress and explicit retry policy.

Pentest stop: Run the incremental pentest for the complete Commit 12 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 13 - Operation Risk And Resource Workflows

Goal: Classify intent and uncertain delivery before mutations.

Deliverables: Read/mutation/destructive/cost metadata, permits, retry/idempotency
decisions, tasks/waiters, bulk per-item results and reconciliation. Distinguish remote
state preconditions from client-side validation.

Verification: Permit replay, stale intent, mismatched operation/body, task success with
errors, partial batches, not-sent/possibly-sent faults, cancellation and poll
exhaustion.

Exit criteria: No included mutation or billable read lacks explicit authority, delivery
and reconciliation behavior.

Pentest stop: Run the incremental pentest for the complete Commit 13 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 14 - Projects, Contracts And Annotations

Goal: Complete public account and annotation workflows.

Deliverables: Project list/create/get/update/delete and delete-with-resources, public
contract operations, annotations/SRN associations, pagination and every additional
account row confirmed public by Commit 1.

Verification: Cross-organization/project denial, protected/default projects,
cascading-delete authority, malformed SRNs, contracts with legal/cost consequences,
exact payloads and executor parity.

Exit criteria: All assigned account rows are executable; private login/signup/account
administration is not inferred from downloadable schemas.

Pentest stop: Run the incremental pentest for the complete Commit 14 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 15 - IAM Identities And SSH Keys

Goal: Implement public identity and key inventory/mutations.

Deliverables: Users/members, groups, applications, membership and SSH-key management
with exact identities, permissions, pagination, protected fields and versioned pre-GA
documentation.

Verification: Privilege widening, group replacement versus addition, duplicate members,
SSH-key syntax/bounds, stale identifiers, secret redaction and partial failure.

Exit criteria: Every assigned identity/key operation is typed and usable without
requiring console-only SSH-key setup.

Pentest stop: Run the incremental pentest for the complete Commit 15 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 16 - IAM Policies, Credentials And Federation

Goal: Complete the public IAM security-management surface.

Deliverables: Policies/rules, permission sets, quotas, API keys/JWTs, audit-related IAM
reads, SAML/SCIM/security settings and other confirmed public IAM rows; protected tokens
and destructive revocation workflows.

Verification: Escalation, lockout, scope confusion, one-time token erasure,
certificate/metadata bounds, revocation races, replay and uncertain delivery. No
browser-session automation.

Exit criteria: All public IAM operations are covered and dangerous security changes
require explicit authority; server-side IAM remains the actual authorization boundary.

Pentest stop: Run the incremental pentest for the complete Commit 16 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 17 - Billing, FinOps And Commercial Catalogs

Goal: Implement every confirmed public commercial interface, regardless of track.

Deliverables: Distinct supported Billing versions, consumption/invoices/files, confirmed
budgets/electronic addresses, FinOps, product catalog, Marketplace/console API when
public, Partner and public Reseller rows. Exact currency and contractual risk metadata.

Verification: SDK/docs discrepancy decisions, money sign/nanos, file limits,
coupon/order side effects, restricted account eligibility, body fidelity and
unauthorized cross-account calls.

Exit criteria: No beta surface is silently relabeled GA, no undocumented v2 route is
invented, and every commercial row has an explicit capability and execution
classification.

Pentest stop: Run the incremental pentest for the complete Commit 17 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 18 - Instance Inventory And Reads

Goal: Complete current Instance read contracts.

Deliverables: All supported versions' servers, images, types, volumes, snapshots,
security/placement groups, IPs, private NICs, user-data reads, availability, dashboard
and assigned auxiliary reads.

Verification: Version-specific fixtures, echoed identity, nullability, unknown fields,
address/size bounds, secret user-data output, page limits and all execution modes.

Exit criteria: No admitted read is model-only and versions sharing similar types cannot
be silently conflated.

Pentest stop: Run the incremental pentest for the complete Commit 18 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 19 - Instance Mutations And Lifecycle

Goal: Complete server, storage and network actions in the principal Instance version.

Deliverables: Creation, PATCH/PUT, snapshots/images, lifecycle, attachments, security
rules, IPs, private NICs, placement, raw user data and migration actions, each with
permits and reconciliation.

Verification: Absent/clear intent, raw body media, secret cleanup, rule widening,
delete/attach conflicts, per-item failure, cost changes and uncertain delivery.

Exit criteria: Every assigned mutation has exact wire fidelity and an executable safe
path; no retry switches versions.

Pentest stop: Run the incremental pentest for the complete Commit 19 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 20 - Instance Alternate Versions And Autoscaling

Goal: Close non-superseded alternate-version and scaling-group coverage.

Deliverables: Current alpha/beta Instance and Instance Volume operations not owned by
Commits 18-19, Autoscaling Groups, templates/policies/actions, version-specific
differences and migration guidance.

Verification: Same-name divergent fields, wrong-version identities, scale limits,
cooldown/time arithmetic, capacity cost, template secrets and polling/cancellation.

Exit criteria: The ledger assigns every supported Instance/volume/autoscaling row once,
with clear track labels and no default-doc-version coverage loss.

Pentest stop: Run the incremental pentest for the complete Commit 20 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 21 - Block Storage

Goal: Complete Block Storage control-plane operations.

Deliverables: Types, volumes, snapshots, cloning/import/export and lifecycle actions
actually exposed by current supported versions; size/locality constraints and attachment
associations.

Verification: Checked sizes, region/zone mismatch, snapshot source binding,
import/export destinations, delete preconditions, task state and response limits.

Exit criteria: All assigned storage operations are executable; caller-visible state
checks never masquerade as race-free server enforcement.

Pentest stop: Run the incremental pentest for the complete Commit 21 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 22 - File Storage

Goal: Implement the public File Storage API including pre-GA contracts.

Deliverables: Filesystems, snapshots/attachments/exports and all source-defined
lifecycle/catalog operations, permissions, size models, costs and version-specific
errors.

Verification: Export destination confusion, access widening, locality mismatch, size
overflow, delete conflicts and uncertain completion.

Exit criteria: All File Storage HTTP rows are supported; mounting or implementing
NFS/SMB clients is not implied.

Pentest stop: Run the incremental pentest for the complete Commit 22 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 23 - Object Storage Endpoints And Control

Goal: Establish Object Storage's account, region and endpoint boundary.

Deliverables: Public control APIs, bucket/object identity, region service discovery,
addressing styles, signing integration, account/IAM associations, exact S3 support
matrix and checked XML/error dispatch.

Verification: Bucket host confusion, key encoding, dot segments, custom origins, region
redirects, wrong signing service and provider-unsupported calls.

Exit criteria: Every supported S3 action is assigned to Commits 24-27 with a verified
endpoint and credential policy.

Pentest stop: Run the incremental pentest for the complete Commit 23 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 24 - S3 Buckets And Configuration

Goal: Implement supported bucket-level management and discovery.

Deliverables: Bucket creation/list/head/delete, location, versioning, lifecycle, CORS
and other assigned settings with exact supported fields, signed requests and XML models.

Verification: Wrong owner/region, configuration replacement, retention-related deletion,
XML injection, unsupported fields and operation-level errors.

Exit criteria: All assigned bucket operations are executable without claiming
unsupported AWS features.

Pentest stop: Run the incremental pentest for the complete Commit 24 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 25 - S3 Objects And Versions

Goal: Implement bounded object reads and writes.

Deliverables: Supported get/head/put/copy/delete/list and version/tag/metadata
operations, range/conditional requests, transactional downloads, streaming uploads and
per-object batch results.

Verification: Key canonicalization, ranges, checksum-before-commit, copy-source binding,
partial deletion, secret metadata, short/long bodies and cancellation.

Exit criteria: Supported object/version operations preserve exact bytes, identity,
response conditions and mutation authority.

Pentest stop: Run the incremental pentest for the complete Commit 25 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 26 - S3 Multipart And Transfer Integrity

Goal: Support large transfers without unbounded memory or ambiguous cleanup.

Deliverables: Initiate/upload/list/complete/abort multipart operations, supported
checksums and signing modes, bounded part bookkeeping, resumable caller-owned state and
reconciliation.

Verification: Missing/reordered/duplicate parts, forged upload IDs, embedded error in
successful HTTP response, early responses, checksum mismatch, cancellation and uncertain
completion.

Exit criteria: Multipart completion cannot report success before protocol validation;
abandoned uploads have explicit caller-controlled cleanup.

Pentest stop: Run the incremental pentest for the complete Commit 26 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 27 - S3 Access, Retention And Advanced Settings

Goal: Complete the remaining supported S3 surface.

Deliverables: Supported ACL/policy, public access, retention/legal hold, encryption,
replication, website and other compatibility-table actions assigned by Commit 2;
unsupported entries remain explicitly rejected.

Verification: Policy widening, retention bypass intent, irreversible settings, external
destinations, missing conditional headers and per-field compatibility checks.

Exit criteria: The entire supported S3 matrix is executable, including rarely used
settings; unavailable AWS features are not falsely advertised.

Pentest stop: Run the incremental pentest for the complete Commit 27 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 28 - Container Registry Control Plane

Goal: Implement registry administration independently of image transfer.

Deliverables: Namespaces, repositories/images/tags, visibility, quotas,
garbage-collection or lifecycle actions where public, protected credentials and all
supported control versions.

Verification: Public/private transitions, cross-namespace access, deletion intent,
registry naming, quotas, pagination and uncertain delivery.

Exit criteria: Every registry control row is typed; image operations are not faked
through generic control JSON.

Pentest stop: Run the incremental pentest for the complete Commit 28 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 29 - Registry Distribution HTTP API

Goal: Implement Scaleway-supported registry data operations.

Deliverables: Source-locked distribution/auth contract, manifest/index and blob
read/write/delete, tag/catalog operations where supported, upload sessions, digests and
explicitly validated challenge origins.

Verification: Malicious auth realms/Location/Link values, digest mismatch, foreign
blobs, mount-source confusion, resume offsets, media negotiation, partial upload and
cleanup.

Exit criteria: All confirmed supported distribution operations work in three modes;
neither arbitrary URL fetching nor container building/execution is introduced.

Pentest stop: Run the incremental pentest for the complete Commit 29 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 30 - VPC And IPAM Reads

Goal: Complete regional network inventory and exact network identities.

Deliverables: VPCs, private networks, subnets, routes, ACLs, IPAM allocations and
confirmed public NIC/next-hop auxiliary views with source-derived pagination.

Verification: Differential IP parsing, canonical versus match CIDRs, overlapping fields,
allocation association, cross-project/locality mismatch and bounded large lists.

Exit criteria: Every assigned read is client-reachable and auxiliary public-status
decisions are recorded.

Pentest stop: Run the incremental pentest for the complete Commit 30 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 31 - VPC And IPAM Mutations

Goal: Complete network allocation and routing changes.

Deliverables: Network/subnet/route/ACL changes, IP reservations/releases and all
supported mutations; strict PATCH intent, caller permits and server-state
reconciliation.

Verification: Route/ACL widening, invalid host bits, address exhaustion, default-route
replacement, concurrent resource changes and uncertain delivery.

Exit criteria: All network mutations enforce local intent without inventing remote state
guarantees.

Pentest stop: Run the incremental pentest for the complete Commit 31 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 32 - Public Gateways v2

Goal: Implement the current supported gateway contract.

Deliverables: Gateways/types, private-network associations, IPAM-reserved IPs, public
IPs, PAT rules, upgrades and SSH bastion allowed-IP operations, with bulk replacement
distinctions.

Verification: Reject v1 paths and removed DHCP fields; exercise IP/port bounds, PAT
conflicts, default-route intent, bastion widening, upgrades and cancellation.

Exit criteria: Every v2 row is executable; legacy DHCP objects/entries and silent
version fallback are absent.

Pentest stop: Run the incremental pentest for the complete Commit 32 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 33 - InterLink And Site-to-Site VPN

Goal: Implement current public private-connectivity APIs.

Deliverables: InterLink and VPN catalogs, connections, gateways/tunnels, routing,
attachments and source-defined actions, credentials, locality, commercial constraints
and pre-GA labels.

Verification: Route leakage, project/provider association, tunnel-secret erasure,
redundant-link failures, billable capacity changes and uncertain provisioning.

Exit criteria: Every assigned connectivity row is typed; split this checkpoint before
coding if the locked contracts exceed one reviewable pass.

Pentest stop: Run the incremental pentest for the complete Commit 33 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 34 - Edge Services

Goal: Implement public edge configuration and lifecycle.

Deliverables: Pipeline/origin/backend, cache, DNS/TLS, routing, purge and other
source-defined resources, protected origin credentials and staged deployment semantics.

Verification: External-origin SSRF boundaries, credential forwarding, header injection,
cache/purge scope, certificate cleanup and partially applied configuration.

Exit criteria: Every edge row is executable and configured origin URLs remain data
rather than automatic SDK fetch targets.

Pentest stop: Run the incremental pentest for the complete Commit 34 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 35 - Load Balancer Reads

Goal: Implement complete load-balancer inventory and observability.

Deliverables: Zonal balancers, types, frontends/backends, routes, ACLs, certificates,
IPs, statistics/metrics and source-defined catalog reads.

Verification: Identity association, protocol/port variants, large statistics, secret
certificate fields, unknown states and pagination.

Exit criteria: All read rows are checked and all returned credentials or URLs receive
explicit handling.

Pentest stop: Run the incremental pentest for the complete Commit 35 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 36 - Load Balancer Mutations

Goal: Complete controlled traffic-management workflows.

Deliverables: Balancer creation/actions, backend/frontend/service changes, health
checks, ACL/routes, certificates, public IPs and cost-bearing type changes.

Verification: Traffic widening, private-key cleanup, conflicting routes, health-check
injection, destructive replacement, uncertain delivery and reconciliation.

Exit criteria: All assigned mutations are executable only with matching intent and
bounded secret-safe bodies.

Pentest stop: Run the incremental pentest for the complete Commit 36 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 37 - Elastic Metal Core

Goal: Implement physical-server inventory, ordering and lifecycle.

Deliverables: Offers, servers, OS/install/partitioning, options, metrics, BMC and
lifecycle operations, batch provisioning and protected installation output.

Verification: Disk-destructive intent, partition arithmetic, BMC expiry, batch partial
outcomes, billable order replay, credentials and polling faults.

Exit criteria: Every core Elastic Metal operation is typed and expensive or destructive
workflows never execute implicitly.

Pentest stop: Run the incremental pentest for the complete Commit 37 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 38 - Elastic Metal Networking And Flexible IPs

Goal: Complete public dedicated-server connectivity.

Deliverables: All current non-superseded private-network versions, interfaces, flexible
IP allocation/attachment/reverse and virtual MAC actions, including alpha interfaces.

Verification: Cross-version/locality confusion, MAC moves, source-server binding,
reverse validation, IP release and capacity cost.

Exit criteria: Every selected current version and flexible-IP row is covered, not
excluded because its path contains alpha.

Pentest stop: Run the incremental pentest for the complete Commit 38 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 39 - Dedibox Reads

Goal: Implement the public Dedibox service inventory.

Deliverables: Phoenix servers, offers/options, disks/hardware, installation/rescue/RAID
status, failovers, quotas, services/backups and any separately documented public legacy
API classified in Commit 1.

Verification: Authority/authentication separation, integer versus UUID identity, bounded
hardware lists, sensitive rescue output and account binding.

Exit criteria: Every public supported Dedibox read has a source-specific adapter;
adjacent consoles do not imply interchangeable APIs.

Pentest stop: Run the incremental pentest for the complete Commit 39 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 40 - Dedibox Mutations

Goal: Complete dedicated-server order and recovery workflows.

Deliverables: Provisioning, installation/cancellation, RAID, BMC/rescue, boot/lifecycle,
services/options, failover/MAC/reverse, backups/tags and other assigned public actions.

Verification: Password erasure, disk destruction, duplicate orders, firewall/network
widening where applicable, lockout, permit replay and uncertain delivery.

Exit criteria: All public mutation rows are covered with explicit recovery instructions
and no generic sensitive-action escape hatch.

Pentest stop: Run the incremental pentest for the complete Commit 40 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 41 - Apple Silicon

Goal: Implement public Apple silicon management and networking.

Deliverables: Machines, offers, OS/status/actions, private-network associations and
source-defined remote-access credential outputs, costs and current pre-GA versions.

Verification: Locality, reservation duration, destructive reinstall, credential
lifetime/redaction, address changes and uncertain actions.

Exit criteria: All public Apple silicon management rows are executable; the SDK does not
implement remote desktop or SSH.

Pentest stop: Run the incremental pentest for the complete Commit 41 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 42 - Kubernetes

Goal: Complete the managed Kubernetes control plane.

Deliverables: Clusters, pools, nodes, versions/upgrades, ACLs, autoscaling,
credentials/kubeconfig and all source-defined actions.

Verification: CNI/version invariants, node limits, ACL widening, kubeconfig cleanup,
upgrade partial failures, identity association and polling.

Exit criteria: Every managed-service row is complete; native Kubernetes API clients are
separate application integrations.

Pentest stop: Run the incremental pentest for the complete Commit 42 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 43 - Serverless Functions

Goal: Implement function management including pre-GA versions.

Deliverables: Namespaces, functions, deployment/code upload, domains, triggers, secrets,
environment, scaling and execution endpoints only where documented public provider APIs.

Verification: Code payload bounds, secret/environment erasure, upload origin binding,
invocation cost, timeout/scale limits and deployment races.

Exit criteria: All public function rows are executable; source compilation and arbitrary
application HTTP calls are not SDK behavior.

Pentest stop: Run the incremental pentest for the complete Commit 43 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 44 - Serverless Containers

Goal: Implement current supported container management versions.

Deliverables: Namespaces, containers, deployment, image references, domains, secrets,
scaling, cron and trigger operations with track-specific differences.

Verification: Registry credentials, scale-to-zero/cost changes, domain ownership, secret
updates, image-reference injection and conflicting deployment state.

Exit criteria: Every GA and non-superseded pre-GA row is implemented without silently
dropping version-specific fields.

Pentest stop: Run the incremental pentest for the complete Commit 44 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 45 - Serverless Jobs

Goal: Implement public scheduled and on-demand job APIs.

Deliverables: Definitions, runs, schedules, environment/secrets, resources, cancellation
and result/status APIs across current versions.

Verification: Schedule bounds, repeat execution cost, run identity, terminal-state
contradictions, secret output and uncertain cancellation.

Exit criteria: All job rows are covered and cancellation never promises that remote
execution has stopped without confirmation.

Pentest stop: Run the incremental pentest for the complete Commit 45 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 46 - Managed PostgreSQL/MySQL Reads

Goal: Complete RDB inventory and diagnostic contracts.

Deliverables: Instances/nodes, engines/versions, settings/users/databases,
backups/snapshots, endpoints, logs/metrics, privileges and confirmed public
encryption-key metadata.

Verification: Exact sizes/numbers, sensitive diagnostic output, log/metric bounds,
unknown settings, encrypted-resource association and pagination.

Exit criteria: Every RDB read and public encryption auxiliary row is assigned and
executable.

Pentest stop: Run the incremental pentest for the complete Commit 46 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 47 - Managed PostgreSQL/MySQL Mutations

Goal: Complete provisioning and recovery without implicit destructive work.

Deliverables: Instance changes, credentials/privileges, backups/restore,
upgrade/failover, endpoints, maintenance and deletion with explicit task reconciliation.

Verification: Privilege escalation, restore-target mismatch, password cleanup,
deletion/retention conflicts, cost changes and possibly-sent faults.

Exit criteria: Every RDB mutation is typed; no SQL driver or server-state race guarantee
is implied.

Pentest stop: Run the incremental pentest for the complete Commit 47 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 48 - Managed Redis

Goal: Implement all public Redis management APIs.

Deliverables: Clusters, versions, users, endpoints, ACLs, settings, backups and
lifecycle/catalog/diagnostic operations actually present in the locked contract.

Verification: Credential cleanup, ACL widening, persistence/restore semantics, capacity
changes, endpoint association and partial actions.

Exit criteria: All Redis management rows are executable with Redis-specific models
rather than lossy shared database abstractions.

Pentest stop: Run the incremental pentest for the complete Commit 48 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 49 - Managed MongoDB

Goal: Implement all public MongoDB management APIs.

Deliverables: Instances/clusters, versions, users/roles, databases, endpoints,
backup/restore, logs/metrics and lifecycle operations for current supported tracks.

Verification: Role escalation, connection-secret redaction, restore conflicts, document
size limits, topology changes and uncertain delivery.

Exit criteria: All MongoDB rows have exact wire coverage and documented version
boundaries.

Pentest stop: Run the incremental pentest for the complete Commit 49 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 50 - Serverless SQL Databases

Goal: Implement the serverless SQL provider surface.

Deliverables: Database and endpoint lifecycle, regions, scaling/usage and any explicitly
documented provider HTTP query interface, with protected credentials and cost metadata.

Verification: Tenant/database binding, query-body limits if applicable, credential
cleanup, scaling cost, delete intent and source-defined rate limits.

Exit criteria: All documented provider HTTP rows are covered; native SQL wire protocols
remain external clients.

Pentest stop: Run the incremental pentest for the complete Commit 50 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 51 - OpenSearch Management

Goal: Implement the public Cloud Essentials for OpenSearch surface.

Deliverables: Clusters, nodes, versions, users/endpoints, configuration, diagnostics and
lifecycle APIs, plus classification of any provider-documented HTTP data operations.

Verification: Credential handling, endpoint provenance, capacity bounds, unknown
versions, destructive changes and partial provisioning.

Exit criteria: All public provider-specific rows are executable; the plan does not
accidentally promise the entire upstream OpenSearch API.

Pentest stop: Run the incremental pentest for the complete Commit 51 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 52 - Kafka And NATS Management

Goal: Implement public managed messaging control planes.

Deliverables: Clusters/namespaces, versions, credentials, endpoints, quotas,
configuration and lifecycle APIs for Kafka and NATS, preserving their distinct models.

Verification: Credential rotation, namespace isolation, retention/storage costs,
endpoint trust, uncertain deletion and permissions.

Exit criteria: All management rows are complete; native Kafka/NATS wire-client
implementations remain outside the provider HTTP contract.

Pentest stop: Run the incremental pentest for the complete Commit 52 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 53 - Data Warehouse And Data Lab

Goal: Implement the public analytics management APIs.

Deliverables: Warehouse and Spark/Data Lab provisioning, catalogs, sessions/jobs and
storage/network associations where source-defined, protected credentials and bounded
diagnostics.

Verification: Job/session identity, external storage secrets, costs, result/log bounds,
cancellation and destructive changes.

Exit criteria: Every source-defined public HTTP operation is covered; any additional
documented data API gets an explicit owner rather than an implicit exclusion.

Pentest stop: Run the incremental pentest for the complete Commit 53 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 54 - Queues And Topics Control Plane

Goal: Implement messaging activation and credential management.

Deliverables: SQS/SNS project activation, credentials, permissions, endpoints and public
console-helper operations, with signing/auth scope and runtime namespaces.

Verification: Read/write/manage permission distinctions, one-time secrets, project
isolation, malicious returned endpoints and revocation.

Exit criteria: Every management operation is executable and data-plane credentials
cannot be reused as unrestricted control-plane credentials.

Pentest stop: Run the incremental pentest for the complete Commit 54 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 55 - SQS-Compatible Queue Operations

Goal: Implement Scaleway's complete supported queue action/parameter set.

Deliverables: Queue administration, send/receive/delete, batching, visibility,
dead-letter/FIFO behavior and long polling as documented; signed requests and
per-message results.

Verification: Unsupported AWS parameters, receipt-handle secrecy, duplicate/partial
batches, visibility side effects, timeout/cancellation and queue-URL substitution.

Exit criteria: Supported queue calls and fields are complete; receiving is correctly
classified as state-affecting, not harmless inventory.

Pentest stop: Run the incremental pentest for the complete Commit 55 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 56 - SNS-Compatible Topic Operations

Goal: Implement Scaleway's supported topic and event action set.

Deliverables: Topics, publishing/batching, subscriptions, confirmation, attributes and
endpoint types actually supported; signed requests and explicit delivery semantics.

Verification: Subscription destination widening, confirmation-token redaction, partial
publish failures, cross-namespace queues and unsupported AWS actions.

Exit criteria: All supported SNS rows are complete; the SDK does not follow subscription
URLs or implement unsolicited callback receivers.

Pentest stop: Run the incremental pentest for the complete Commit 56 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 57 - RabbitMQ Management

Goal: Implement the public MessageQ/RabbitMQ provider API.

Deliverables: Instances/clusters, catalogs, versions, endpoints, users/credentials,
configuration and lifecycle operations from the public contract.

Verification: Credential cleanup, vhost/account association where exposed, endpoint
boundaries, capacity cost and uncertain recovery.

Exit criteria: All provider operations are covered; generic AMQP clients and
undocumented broker-admin routes are not added.

Pentest stop: Run the incremental pentest for the complete Commit 57 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 58 - IoT Hub

Goal: Complete public IoT management and diagnostics.

Deliverables: Hubs/devices/routes/networks, certificates/keys, metrics/events and
lifecycle operations across current versions, with protected one-time outputs.

Verification: Device identity, route/topic validation, certificate/key erasure,
destination widening, metric bounds and permission changes.

Exit criteria: Every public control/HTTP data row is supported; native MQTT client
implementation is not implied.

Pentest stop: Run the incremental pentest for the complete Commit 58 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 59 - DNS And Domain Registration

Goal: Implement public zones/records and registrar/suggestion APIs.

Deliverables: Zones, records/RRSets, imports/exports, DNSSEC where exposed, domains,
contacts, orders/transfers/renewals and suggestions, with explicit public versions and
billing/PII policy.

Verification: DNS canonicalization and record types, zonefile bounds, transfer-secret
erasure, contact privacy, order replay, renewal cost and ambiguous delivery.

Exit criteria: All public DNS and registrar rows are executable; split reads/mutations
or registrar work before coding if the inventory exceeds one reviewable checkpoint.

Pentest stop: Run the incremental pentest for the complete Commit 59 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 60 - Web Hosting Core

Goal: Implement hosting inventory, ordering and lifecycle.

Deliverables: Offers, hosting creation/update/delete, domain associations, commitments,
sessions/passwords, backups/restores and all assigned core/offer rows.

Verification: Billable commitments, one-time session URL handling, password base64
decoding, restore traversal, destructive intent and uncertain ordering.

Exit criteria: Core hosting workflows are executable and protected session URLs are
never automatically fetched.

Pentest stop: Run the incremental pentest for the complete Commit 60 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 61 - Web Hosting Services

Goal: Complete hosting sub-API coverage.

Deliverables: Databases/users, websites, DNS/free domains, FTP/mail accounts and
confirmed public control-panel/Dedibox auxiliary routes, exact field models and scoped
secrets.

Verification: Account privilege widening, domain binding, credentials, duplicate
identities, deletion dependencies and sub-API endpoint confusion.

Exit criteria: Every public Web Hosting sub-API row is covered, not only the main
hosting schema.

Pentest stop: Run the incremental pentest for the complete Commit 61 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 62 - Transactional Email

Goal: Implement the public transactional email HTTP API.

Deliverables: Domains/verification, sending, templates/settings, message status/events,
suppression and other source-defined operations with PII and billable-action policy.

Verification: Recipient/header injection, attachment/MIME bounds, secrets, duplicate
sends, bounce/event parsing and uncertain delivery.

Exit criteria: All HTTP rows are executable; no implicit email sending or SMTP client
implementation is introduced.

Pentest stop: Run the incremental pentest for the complete Commit 62 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 63 - Mailbox

Goal: Implement public mailbox management including beta/alpha contracts.

Deliverables: Mail domains, accounts, aliases, credentials, configuration and lifecycle
operations as source-defined, with explicit destructive and privacy handling.

Verification: Alias/permission widening, domain ownership, password cleanup, account
deletion and status ambiguity.

Exit criteria: All provider mailbox rows are complete; IMAP/POP/SMTP sessions remain
separate application protocols.

Pentest stop: Run the incremental pentest for the complete Commit 63 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 64 - Cockpit Control Plane

Goal: Implement public global and regional observability management.

Deliverables: Data sources, tokens, alerts/contacts, plans, usage, retention, endpoints
and Grafana-access operations, excluding superseded rows only with source evidence.

Verification: Token erasure, cross-region datasource association, contact injection,
retention/cost changes and credential-bearing URL redaction.

Exit criteria: All current public Cockpit management rows are complete and data-plane
scopes are explicit.

Pentest stop: Run the incremental pentest for the complete Commit 64 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 65 - Cockpit Query And Administrative HTTP APIs

Goal: Implement documented supported observability data-source endpoints.

Deliverables: Source-locked metrics/logs/traces query, labels/series, rules/alerts and
administrative subsets supported by Scaleway, with dedicated credential and endpoint
policy.

Verification: Query/time-range cardinality bounds, response limits, tenant isolation,
untrusted links, unsupported upstream routes and cancellation.

Exit criteria: Every supported HTTP query/admin row is executable without promising all
upstream Prometheus/Loki/Tempo/Grafana APIs.

Pentest stop: Run the incremental pentest for the complete Commit 65 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 66 - Cockpit Ingestion And Binary Transfers

Goal: Complete provider-supported HTTP telemetry ingestion.

Deliverables: Documented write/ingest/export formats and required binary/compression
codecs, bounded streaming, precise error classification, secret-safe buffers and
cost/retry policy.

Verification: Compression bombs, codec malformed lengths, partial batches, replay/dedup
semantics, quota errors, byte limits and uncertain acceptance.

Exit criteria: All supported HTTP ingestion variants have tested bounded
implementations; unreviewed gRPC or native telemetry protocols are not silently claimed.

Pentest stop: Run the incremental pentest for the complete Commit 66 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 67 - Audit Trail And Environmental Footprint

Goal: Implement public audit and sustainability APIs.

Deliverables: Events, filters, exports, summaries, usage/impact metrics and all
source-defined public operations with sensitive audit payload policy.

Verification: Time/filter bounds, cross-project leakage, pagination, exact numeric
units, large exports and audit-event redaction.

Exit criteria: All rows are executable and sensitive export handling is not mistaken for
ordinary public metadata.

Pentest stop: Run the incremental pentest for the complete Commit 67 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 68 - Secret Manager And Key Manager Administration

Goal: Implement secret/key metadata and lifecycle.

Deliverables: Secrets/versions, key catalogs, policies, rotation,
scheduling/recovery/deletion, imports and all administrative actions, with explicit
access and destructive intent.

Verification: Version confusion, irreversible deletion, rotation races, scope
escalation, imported-material cleanup and retention errors.

Exit criteria: All administrative rows are supported without leaking key or secret
material through ordinary diagnostics.

Pentest stop: Run the incremental pentest for the complete Commit 68 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 69 - Secret Values And Remote Cryptographic Operations

Goal: Implement protected secret access and public KMS data operations.

Deliverables: Secret payload read/write, encryption/decryption, signing/verification and
data-key generation where documented; algorithm/context binding and protected
input/output.

Verification: Plaintext erasure, algorithm substitution, context mismatch, invalid
verification results, one-time outputs, allocation faults and remote call cost.

Exit criteria: All public secret/KMS data rows are complete; remote cryptography does
not imply local FIPS certification or a new hand-written crypto engine.

Pentest stop: Run the incremental pentest for the complete Commit 69 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 70 - Quantum As A Service

Goal: Implement the public QaaS control and job APIs.

Deliverables: Platforms/catalogs, sessions, jobs, inputs/results and lifecycle
operations present in the source inventory, with bounded payloads and costs.

Verification: Job/session identity, payload dimensions, result bounds, terminal-state
conflicts, cancellation and uncertain billable execution.

Exit criteria: Every public QaaS row is executable without implementing an unrelated
local simulator.

Pentest stop: Run the incremental pentest for the complete Commit 70 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 71 - Dedicated Inference And AI Control

Goal: Implement AI deployment and consumption management.

Deliverables: Dedicated deployments/models/catalogs, credentials, scaling,
private-network associations, limits/quotas and all confirmed public AI control rows.

Verification: Model/deployment binding, credentials, custom-model storage references,
scale costs, region selection and task state.

Exit criteria: All public AI control operations are executable, with consumption-limit
visibility resolved rather than ignored.

Pentest stop: Run the incremental pentest for the complete Commit 71 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 72 - Generative JSON APIs

Goal: Implement synchronous public inference requests and model discovery.

Deliverables: Supported responses, chat, embeddings, rerank and models operations; exact
Scaleway-supported fields and schemas, api.scaleway.ai bearer scope, model capabilities
and prompt/output privacy.

Verification: Cross-origin credentials, unsupported compatibility fields, tool-call
parsing, multimodal reference handling, token/byte limits, numeric vectors and billable
retries.

Exit criteria: Every non-streaming JSON operation and supported field is executable;
tool calls and remote media URLs are data, never automatically executed or fetched.

Pentest stop: Run the incremental pentest for the complete Commit 72 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 73 - Generative Audio And Batch APIs

Goal: Implement non-JSON and asynchronous AI data workflows.

Deliverables: Supported multipart audio upload, batch create/list/get/cancel/delete, S3
input/output references and protected metadata; reuse signed storage/transfer
foundations.

Verification: Multipart boundary injection, audio limits, bucket/object identity,
partial results, private URL handling, uncertain creation and cost/cancellation.

Exit criteria: All audio/batch rows are complete without claiming unsupported upstream
fields or automatic file creation/upload.

Pentest stop: Run the incremental pentest for the complete Commit 73 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 74 - Generative Streaming APIs

Goal: Complete documented AI SSE and streaming behavior.

Deliverables: Incremental typed events, terminal/error markers, per-event and total
budgets, backpressure, cancellation, usage accounting and all execution modes.

Verification: Split UTF-8/frames, multiline events, truncated streams, errors after HTTP
success, oversized events, stalled producers and cancellation cleanup.

Exit criteria: Every supported streaming variant has explicit partial-output semantics;
EOF without required completion cannot report a successful full response.

Pentest stop: Run the incremental pentest for the complete Commit 74 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 75 - Unified Client And Workflows

Goal: Make the complete provider usable without unsafe assembly work.

Deliverables: Official-endpoint clients, version/track discovery, pagers/waiters,
reconciliation, cross-product recipes, optional feature examples and three-mode parity
across every operation.

Verification: External-consumer compile tests, provider/version mixing rejection, custom
endpoint warnings, cancellations and representative secret/cost/destructive workflows.

Exit criteria: No documented happy path requires manual credential/header/path assembly
and every inventory row maps to a callable typed client.

Pentest stop: Run the incremental pentest for the complete Commit 75 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 76 - Completeness And Freshness Audit

Goal: Reconcile the implemented provider with the real API again.

Deliverables: Fresh control/data source observations, operation and field coverage,
compatibility restrictions, deprecated/superseded decisions, source discrepancies and
documentation capability tables.

Verification: Regenerate from locked sources, compare fresh official sources, mutate
optional fields/status/media variants and prove every omission fails; audit all public
product dispositions.

Exit criteria: No unclassified or model-only public row remains. New public scope is
implemented through an approved added checkpoint, or the release explicitly abandons the
full claim.

Pentest stop: Run the incremental pentest for the complete Commit 76 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 77 - Live Evidence And Cleanup Qualification

Goal: Validate real provider integration with honest access boundaries.

Deliverables: Opt-in least-privileged reads, approved isolated mutation/data-plane
scenarios where credentials/resources exist, cleanup/reconciliation records, cost caps
and reproducible local harnesses.

Verification: Credential-file permissions, destination allowlists, no-token CI,
budget/timeout enforcement, failed cleanup visibility and comparison of real wire
evidence to fixtures.

Exit criteria: Every live claim is backed by evidence; unavailable products/accounts are
explicitly listed, not represented as tested or omitted from implementation.

Pentest stop: Run the incremental pentest for the complete Commit 77 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 78 - Adversarial And Fuzz Qualification

Goal: Aggregate and extend the tests accumulated throughout the train.

Deliverables: Complete fuzz inventory, structured corpora for every parser/protocol
family, independent codec/signature oracles and adversarial shared execution scenarios.

Verification: Bounded campaigns, deterministic oversized inputs, allocation failure,
no-progress cases, redaction/cleanup, concurrent credential changes and cancellation at
each transfer boundary.

Exit criteria: All targets compile and run, assertions fail closed, and regressions
become durable fixtures rather than undocumented local probes.

Pentest stop: Run the incremental pentest for the complete Commit 78 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 79 - Compatibility, Packaging And Supply Chain

Goal: Qualify all supported configurations and release artifacts.

Deliverables: Current compiler/MSRV and platform evidence, default/feature graphs,
dependency/advisory/license review, SBOMs, docs/examples, SemVer baseline and package
publication plan.

Verification: Existing Hetzner/crates.io regressions, isolated feature combinations,
latest-tool freshness, two-clean-clone package/SBOM reproduction, first-publication
ownership and unchanged-package selection checks.

Exit criteria: Portable/no_std and compatibility claims have evidence; optional
protocols do not inflate defaults and only deliberately selected packages can publish.

Pentest stop: Run the incremental pentest for the complete Commit 79 range, remediate
and retest, then wait for green GitHub CI and CodeQL before continuing.

## Commit 80 - Full Provider Review And Release Candidate

Goal: Freeze and qualify the complete documented public Scaleway implementation.

Deliverables: Final source locks, exact operation/field counts, all checkpoint evidence,
threat model, stability/deprecation/migration docs, release notes, selected package
versions and one composed release gate.

Verification: Compose the preceding 79 gates with this checkpoint's final checks,
fresh API drift, complete workspace/provider tests, three modes, fuzz, platforms,
dependencies, docs, public API and reproducible artifacts. The final gate must not
invoke itself recursively; repeat affected qualification after fixes.

Exit criteria: All in-scope public rows are executable and documented, unresolved
discrepancies are zero, full-provider review/retest is accepted, and GitHub CI/CodeQL
are green on the exact approved commit.

Pentest stop: Run the full-provider pentest and affected-neutral review, remediate and
retest, rerun the complete release gate, then wait for green GitHub CI and CodeQL.
Tagging and publication require explicit approval.

## Outside The Provider API Claim

These are boundaries, not omissions hidden by a GA-only policy:

- Private/internal/fake APIs and browser-session, signup, CAPTCHA, payment-card
  or console automation without a documented public programmatic contract.
- Removed/deprecated interfaces selected as superseded, and AWS/OpenAI/OCI or
  other upstream operations/parameters that Scaleway explicitly does not support.
  Deprecation within an active response still requires a decoding decision.
- Native SQL, Redis, MongoDB, Kafka, NATS, MQTT, AMQP, SSH, RDP, NFS/SMB,
  SMTP/IMAP/POP, and generic Kubernetes client implementations. Provisioning and
  credential/endpoint handoff are in scope; these application protocols are not.
- Running customer workloads, executing returned AI tool calls, building images,
  compiling serverless source, or fetching arbitrary URLs returned by services.
- Local FIPS qualification, automatic disaster recovery, unattended spending,
  background credential harvesting and implicit mutation retries.

Public provider-specific HTTP data APIs must be inventoried even if related
native protocols are outside scope. A schema-less documented interface is not
excluded merely for lacking machine-readable definitions. Additional required
protocols or codec features trigger a reviewed checkpoint split, not a silent
fallback to a subset claim.

## Maintenance After Release

Commit 4 documents a single offline verification command and a single live
observation command with reviewed source refresh. Release qualification always
runs the live observation. Schedule credential-free CI observation at a bounded
frequency; failures and changed sources need review, never automatic lock
acceptance. Detect new products/versions as well as changes within known ones.

Freshness concerns operation/field contracts, auth, endpoints, locality,
supported parameters, limits, protocol formats, deprecations and prose-only
policy. A successful schema hash check alone is not API coverage evidence.
New upstream public functionality after the final accepted snapshot requires
an explicit follow-up scope, compatibility review, tests and pentest.

## Release Decision

Do not preassign a release version in this plan. After Commit 80 is qualified,
choose a SemVer-compatible workspace release and an appropriate initial
cloud-sdk-scaleway package version. Existing packages use the repository's
independent-version policy; adding a provider does not automatically justify
republishing every package.

The first publication must account for the initially absent provider namespace,
verify package contents and README status, and verify ownership immediately
after publication. A signed tag binds the exact qualified, GitHub-approved
commit. No extra unreviewed source changes are bundled into the tagging step.
