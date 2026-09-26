# DigitalOcean Commit Plan

Status: full public API implementation roadmap, reviewed against official
sources on 2026-09-26. This plan does not start implementation, assign a release
version, or authorize publication. DigitalOcean follows Scaleway in the
[current provider order](IMPLEMENTATION_PLAN.md#post-10-provider-direction).

## Decision Summary

The estimated train is **75 numbered implementation checkpoints** followed by
one separately approved provider release. A checkpoint is a reviewed scope,
not a promise of exactly one Git object. Remediation adds ordinary commits;
split a checkpoint before implementation if it exceeds a safe review pass.

The target is **all current documented public DigitalOcean APIs**, including
GA, alpha, beta, preview, and publicly documented restricted-access contracts.
This includes the central control plane, AI/inference services, secure OAuth,
Spaces' supported S3 HTTP subset, registry distribution, Functions execution,
Droplet metadata, Secrets Manager, and the current Paperspace API. Public
status and exact provider-supported compatibility behavior must be source-locked;
neither an absent OpenAPI entry nor lack of a test account justifies exclusion.

"Full" means the provider's published contract, not every feature of upstream
AWS S3, OpenWhisk, OpenAI, Docker/OCI, or a hosted database engine. Native SQL,
Redis, Kafka, NFS, SSH and desktop clients, user-defined application APIs,
private console routes, and retired endpoints are not provider API omissions.
The precise boundary and evidence rules appear below.

The central survey now contains **715 operations across 484 paths**, of which
20 are marked deprecated. Its 695 non-deprecated operations are only one
inventory: separately documented APIs add further rows. A central-spec coverage
percentage alone must never become the full-provider support claim.

## Commit Checkpoint Workflow

Work stays on `main`; numbered checkpoints are not releases.

1. Record the approved release tag and accepted baseline SHA at implementation
   start. The latest published workspace baseline at this review is `v1.1.0`;
   use the later accepted release if Scaleway has shipped before this train.
2. Implement the checkpoint, run its local gate, and commit normally.
3. Pentest the entire delta from the preceding accepted baseline to `HEAD`.
4. Commit fixes and retest that complete delta until green.
5. Record accepted pentest evidence and wait for green GitHub CI and CodeQL
   on the exact candidate. CI fixes are committed, checked, and reflected in
   the evidence before proceeding.
6. Record the accepted SHA, then start the next numbered checkpoint. Do not
   tag or publish at intermediate stops.

Each checkpoint below inherits this workflow, the field-level coverage rules,
the feature/platform contract, and fail-closed verification. The final stop
also requires a full-provider review, complete release qualification, and
separate approval of versions, signed tag, and publication.

## Current Source Survey

Primary sources to lock and monitor:

- [Central OpenAPI JSON][digitalocean-openapi] and
  [official OpenAPI repository](https://github.com/digitalocean/openapi).
- [API index](https://docs.digitalocean.com/reference/api/),
  [token scopes](https://docs.digitalocean.com/reference/api/scopes/),
  [OAuth](https://docs.digitalocean.com/reference/api/oauth/), and
  [release notes](https://docs.digitalocean.com/release-notes/).
- [Spaces S3 compatibility](https://docs.digitalocean.com/reference/api/spaces/)
  and [Droplet metadata](https://docs.digitalocean.com/reference/api/metadata/).
- [Functions API](https://docs.digitalocean.com/products/functions/reference/api/),
  [execution capabilities](https://docs.digitalocean.com/reference/mcp/functions-mcp-tools/),
  and [registry documentation](https://docs.digitalocean.com/products/container-registry/reference/).
- [Secrets CLI](https://docs.digitalocean.com/reference/doctl/reference/secrets/),
  [official godo implementation](https://github.com/digitalocean/godo/blob/main/secrets.go),
  and the official SDK/CLI product inventories as discrepancy detectors.
- [Current Paperspace reference](https://docs.digitalocean.com/reference/paperspace/api-reference/),
  [Paperspace lifecycle notices](https://docs.digitalocean.com/products/paperspace/),
  and [legacy retirement warning](https://docs.digitalocean.com/reference/paperspace/gradient/install/).

The downloaded central OpenAPI 3.0 document has 1,030 schemas, no missing or
duplicate operation IDs, and methods: 370 GET, 155 POST, 104 DELETE, 67 PUT,
19 PATCH. Its SHA-256 is
`154f2cd2f99565d9cbf43250301cbdb6a4d9682651bb3433f82f481772d3f68a`.
The observed repository HEAD was
`cfce2902a8d3e0a03d5b0441b1924a9db41ad7e2`; this is secondary evidence, not
an assertion that the generated daily JSON came from that exact revision.
These are survey observations, not maintained release locks. Commit 1 fetches,
pins, and reconciles every source again.

Compared with the previous plan's 659-operation survey, the central snapshot
is larger by 56 operations. Notable coverage additions are Action Gateway,
public-preview VPC subnets/routes, GradientAI scenarios/simulations, and billing
prepayment reads. Exact structural differences must come from the maintained
drift inventory, not arithmetic alone.

The central document describes 700 operations at `api.digitalocean.com`,
13 at `inference.do-ai.run`, one templated agent authority and one returned
presigned-upload authority. These are not the provider's complete host list:
Spaces, metadata, Functions execution, OAuth and Paperspace need independent
service and credential policies. The current Paperspace reference uses
`https://api.paperspace.com/v1` with its own bearer credentials.

### Required Reconciliation

| Observation | Required disposition | Checkpoint owners |
| --- | --- | --- |
| Central OpenAPI is not the whole API catalog | Union official product, SDK/CLI and protocol inventories; reject unknown public rows | 1-3, 71 |
| VPC subnets/routes are public preview | Implement the track with exact lifecycle labels | 22 |
| Secrets Manager is present in official CLI/godo but absent from the central snapshot | Lock paths, scopes, fields and HTTP semantics; resolve the SDK's 204/body discrepancy without weakening general framing | 1, 27 |
| Functions namespace management does not cover action/package/activation execution | Inventory and implement the supported separate execution API | 37-38 |
| Spaces and registry management do not transfer objects/images | Add supported protocol operations, signing, streaming and integrity evidence | 31-36 |
| GradientAI and Action Gateway gained public surfaces | Include scenario/simulation and gateway operation/field coverage | 55-60 |
| Paperspace has a separate current reference | Cover current account, compute, network, storage, project and ML resource APIs inside the same provider crate | 62-68 |
| Legacy Paperspace docs remain online although old Core/Gradient endpoints are retired | Preserve retirement evidence, not executable legacy clients | 1-2, 62, 71 |
| OAuth examples need security/source reconciliation | Resolve revocation-host typo and secret-bearing token example; only verified safe code/refresh/revoke behavior, no implicit grant | 5-6, 69 |
| Returned session URLs do not prove a usable session client | Lock and qualify supported logs/console/upgrade contracts | 39-41, 70 |

Unknown status, conflicting official sources, inaccessible required source data,
or undocumented wire semantics blocks the affected checkpoint. Never convert
these cases into an empty inventory or a silently deferred public service.
Record source URLs, immutable revisions/digests, retrieval time and observed
release track per row. Pin SDK source revisions rather than relying on moving
`main` links as release evidence.

[digitalocean-openapi]:
  https://docs.digitalocean.com/reference/api/reference/openapi.json

## Architecture And Coverage Rules

1. One provider crate: `cloud-sdk-digitalocean`, with bounded product modules,
   including Paperspace. No separate crate for each product or protocol.
2. Keep reusable transport, signing, sanitization, polling, pagination, streaming
   and test helpers neutral. Reuse the reviewed Scaleway implementation if it
   exists by then, but retain independent DigitalOcean conformance tests.
3. Empty default features and `no_std` remain the foundation. Alloc, Serde,
   streaming, XML and transport capabilities are explicit optional boundaries;
   existing provider graphs must remain unchanged.
4. Retain the workspace's then-supported MSRV and stable development toolchain,
   platform matrix, dependency admission and 500-line code-file limit.
   FIPS stays deferred until the separately qualified Brynja work.
5. Use the admitted sanitization crate for protected secrets and base64-ng if
   needed. No new cryptographic primitive or unreviewed parser implementation.
6. Coverage includes every request field and union branch, exact path/query/body
   encoding, response field/status/media variant, error, auth scope, pagination,
   rate policy, retry/idempotency, cost and execution-mode association.
7. Every included operation receives typed preparation and checked execution
   through blocking, local async and Send async where applicable. Streaming
   contracts need qualified adapters, not only traits or returned URLs.
8. Bounded allocation, protected cleanup, cancellation, concurrency, uncertain
   delivery and diagnostic redaction are verified across shared paths.
   Public errors implement payload-free Display and core::error::Error.
9. Official origins are the safe default. Custom origins require explicit
   trusted configuration. Source-locked returned endpoints, registry realms,
   redirects and signed URLs require service/resource binding; tokens never
   follow arbitrary URLs or a hostname suffix alone.
10. Mutations, credential reads, destructive and billable operations require
    explicit intent. GET is not automatically non-billable or safe for smoke
    tests. Retry, polling and cleanup remain caller-authorized and bounded.
11. API track labels are preserved. Public alpha/beta/restricted-access APIs are
    included with their instability/access conditions, not disguised as GA.
12. Deprecated/superseded/retired rows retain evidence and replacement mappings,
    but do not become new executable SDK calls. The central survey's 20 rows
    cover the old singular registry API, node-pool recycle and retired model key.
13. Supported public fields cannot be dropped to meet a checkpoint count.
    Split or extend this plan before implementation, with the same pentest
    and CI stops; public scope growth before release must be reconciled.
14. The safe supported OAuth authorization-code path is included; implicit
    token fragments are deliberately rejected under
    [RFC 9700](https://www.rfc-editor.org/rfc/rfc9700.html). PKCE/public-client
    support requires official evidence, not inference from generic OAuth.
    Spaces uses reviewed SigV4; legacy SigV2 is not added merely for parity.

## Gate And Maintenance Contract

Each numbered checkpoint adds a reproducible local verification entry covering
its own changes and affected shared behavior. CI runs the accumulated checks.
Every product checkpoint uses the ledger from Commit 3 to test every admitted
operation and field, not only representative CRUD requests. Success, failure,
boundary and cancellation fixtures must actually reach their assertions.

The drift tool must offer documented offline verification, live read-only
comparison, and a separate explicit reviewed lock-refresh command. Fetches use
bounded responses, redirect policy, wall-clock deadlines, worker cleanup and
immutable digest evidence. A live failure is an error, never "no changes."
Nested schema, compatibility-table, auth/scope, protocol, lifecycle and public
product additions all produce reviewable differences. CI verifies generated
coverage against implementations; a digest update cannot approve missing code.

Commit 71 repeats the whole public catalog reconciliation before candidate
qualification. Commit 75 runs predecessor gates and its own candidate checks;
it must not recursively invoke itself. No command names or generated artifacts
in this section are claimed to exist before their implementation checkpoint.

## Commit 1 - Source Lock And Finite Scope

Goal: establish the exact DigitalOcean support claim before provider code exists.

Deliverables: bounded retrieval of the daily OpenAPI document, API reference, OAuth
guide, scope catalog, Spaces and metadata references, Paperspace reference, Functions
execution documentation, official SDK/CLI evidence, and product catalogs; exact digest,
path, method, operation ID, tag, server, stability, deprecation, scope, request,
response, and protocol records; and an included/excluded/superseded matrix for every
row. Public GA, alpha, beta, preview, and documented restricted-access contracts are
included; unknown public status blocks admission instead of silently dropping the row.

Verification: reject cross-origin redirects, malformed JSON or Markdown, unresolved
references, duplicate identities, malformed servers, unknown auth schemes, undocumented
scope combinations, and unclassified rows; independently reproduce all source digests
and the preliminary 715-operation central count; independently inventory every external
contract and detect documentation/SDK operations missing from the central specification.

Exit criteria: the exact included count is reviewable, all 20 deprecated rows are
classified, every public preview or limited-access contract has an implementation owner,
and any estimate change is recorded here before implementation.

Pentest stop: stop after Commit 1; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 2 - Drift, Changelog, And Lifecycle Detection

Goal: turn upstream changes into fail-closed maintenance events.

Deliverables: a DigitalOcean adapter for the neutral drift engine; operation, parameter,
schema, authority, auth-scope, rate, pagination, deprecation, and protocol fingerprints;
documentation and repository observations; and a reviewed lock-refresh workflow across
every admitted source, not just central OpenAPI. Compare nested field paths, union
branches, enums, defaults, nullability, requiredness, media/status variants, and
credential permissions.

Verification: fixtures for added, removed, moved, deprecated, retired, and changed
operations; scope and requiredness changes; source disagreement; redirect, timeout,
size, and parser failure; and local-only plus live modes that never accept drift
automatically.

Exit criteria: CI detects every matrix category, including returned-authority contract
changes, and cannot turn an incomplete observation into a green run.

Pentest stop: stop after Commit 2; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 3 - Operation, Field, And Protocol Coverage Ledger

Goal: make the complete public contract measurable before implementing endpoints.

Deliverables: a generated ledger joining all sources to product, API track, operation,
nested request/response field, enum/union variant, parameter, status, media type, auth,
and executable client symbol. Track path/query/body encoding, success/error decoding,
blocking/local-async/Send-async execution, examples, and tests separately.

Verification: negative fixtures deleting one field, metric type, union branch, error
status, streaming mode, preview product, or separately documented operation; fail on an
unknown or duplicate mapping and on stale generated artifacts.

Exit criteria: every public row has an implementation checkpoint; unknown rows block the
plan, and no percentage treats a model-only row as complete.

Pentest stop: stop after Commit 3; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 4 - Crate, Identity, And Module Boundaries

Goal: add DigitalOcean without coupling provider behavior to neutral core code.

Deliverables: `cloud-sdk-digitalocean`; provider/service identities; empty default
features; bounded product modules; feature ownership for models, Serde, and transport
adapters; README, package metadata, licensing, and docs.rs setup.

Verification: default/all-feature and `no_std` builds; external identity tests;
forbidden dependency graphs; package contents; file-length policy; and proof that
unrelated provider crates do not depend on DigitalOcean code.

Exit criteria: the crate is independently consumable, contains no endpoint
implementation yet, and preserves workspace platform and dependency contracts.

Pentest stop: stop after Commit 4; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 5 - Authorities, Redirects, And Returned URLs

Goal: make every admitted authority safe before credentials or operations exist.

Deliverables: constructors for the control plane, serverless inference, Agent Inference
suffix-bound hosts, OAuth authority, and presigned upload targets; Paperspace and
Functions authorities, Spaces regions, registry challenges, workload-local metadata
exceptions, custom endpoint policy, redirect rules, and credential-stripping
transitions. Returned URLs require source-locked operation/account/resource binding, not
merely a matching hostname suffix.

Verification: host, port, user-info, Unicode, suffix confusion, encoded separator,
downgrade, redirect, DNS-independent authority, wildcard, presigned query, replay,
expiry, and wrong-service tests.

Exit criteria: credentials cannot cross authorities, agent hosts cannot escape the
reviewed suffix, and presigned uploads use only exact returned HTTPS URLs.

Pentest stop: stop after Commit 5; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 6 - Credentials, OAuth Secrets, And Rotation

Goal: represent every credential class without accidental copying or leakage.

Deliverables: protected personal, OAuth access, refresh, inference, agent, database,
registry, Kubernetes, Spaces, Functions namespace, Secrets Manager, Paperspace, and
presigned credential types; guarded ingestion, rotation, expiry, erasure, redacted
diagnostics, and authority/scope binding.

Verification: prefix, length, control byte, CRLF, empty, wrong-context, clone, drop,
refresh race, source cleanup, expired token, duplicate header, Debug, Display, and
error-chain redaction tests.

Exit criteria: secret classes are non-interchangeable, never stored as ordinary owned
strings by the SDK, and cannot be attached to an unrelated authority.

Pentest stop: stop after Commit 6; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 7 - Scope, Permit, Cost, And Retry Metadata

Goal: bind least privilege and execution intent to every operation.

Deliverables: source-generated scope constants and operation associations; read,
mutation, destructive, credential-view, and cost permits; retry and idempotency
classifications; and compile-time operation/preparation bindings.

Verification: all included rows have exact scope evidence; scope aliases cannot hide a
missing granular scope; mutations cannot use read permits; non-idempotent creates and
actions never retry automatically; and no operation is model-only.

Exit criteria: coverage tooling rejects any operation lacking scope, authority, cost,
retry, permit, request, response, or error classification.

Pentest stop: stop after Commit 7; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 8 - Wire, Errors, Pagination, And Rate Limits

Goal: define one checked wire contract for the complete provider.

Deliverables: common success/error envelopes; optional request IDs; content types;
bounded bodies; empty `204` handling; absolute-link pagination; quota headers; hourly,
burst, SSH-key, and CDN limits; and payload-free public errors.

Verification: status/content-type matrices, malformed and duplicate JSON, unknown
fields, oversized bodies, invalid links, header duplication, reset rollback, quota
arithmetic, 429 `Retry-After`, concurrent scheduling, and redacted diagnostics.

Exit criteria: errors cannot be decoded as success, pagination cannot escape an
authority, and all documented rate policies are representable and enforceable.

Pentest stop: stop after Commit 8; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 9 - Actions And Asynchronous Resource Drivers

Goal: support DigitalOcean action resources and long-running operations once.

Deliverables: action list/get models; status, timestamps, resource associations, and
error states; bounded polling; cancellation, timeout, progress, and terminal state
policies; and provider-specific action adapters.

Verification: stale/regressing actions, unknown states, wrong-resource association, busy
loops, rate exhaustion, timeout, cancellation, duplicate terminal state, and
sync/local-async/async parity.

Exit criteria: later product commits reuse one fail-closed action driver and no resource
module hand-rolls polling semantics.

Pentest stop: stop after Commit 9; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 10 - Account, Regions, Sizes, And 1-Clicks

Goal: implement the foundational catalog and account reads plus Kubernetes 1-Click
installation.

Deliverables: account, region, size, 1-Click list, and Kubernetes install operations;
typed availability, feature, price, slug, quota, and install models; and exact
cost/mutation metadata.

Verification: region/size mismatch, unavailable offerings, malformed prices, unknown
features, pagination, install target binding, cost permit, response association, and
transport parity.

Exit criteria: every included foundational row is executable and the install operation
cannot run without explicit billable mutation authority.

Pentest stop: stop after Commit 10; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 11 - SSH Keys, Tags, Projects, And Organizations

Goal: implement shared account organization and resource-labeling workflows.

Deliverables: SSH key, tag, project, default-project, project-resource, and
organization-team operations; bounded names and fingerprints; assignment and membership
models; and destructive/mutation permits.

Verification: SSH fingerprint and key grammar, tag encoding, default-project identity,
duplicate assignments, cross-team confusion, last-owner-like cases, special SSH list
rate limit, and complete wire fixtures.

Exit criteria: all included rows in these areas are executable with exact team, project,
tag, and resource identity binding.

Pentest stop: stop after Commit 11; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 12 - Billing And Invoice Artifacts

Goal: implement billing reads with bounded structured and binary responses.

Deliverables: balance, billing history, insights, invoice list/detail/summary, CSV, PDF,
prepayment configuration, and prepayment status operations; money/currency/time models;
binary streaming; and invoice preview handling.

Verification: decimal precision, currency mismatch, negative/overflow values, malformed
periods, preview identity, content disposition/type, oversized PDF or CSV, truncation,
cancellation, and response association.

Exit criteria: all source-locked billing rows are executable without lossy money
conversion or full artifact buffering.

Pentest stop: stop after Commit 12; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 13 - Marketplace Add-Ons

Goal: implement add-on catalog and subscription lifecycle with explicit cost.

Deliverables: list/get/app metadata, create, update, plan update, and delete operations;
plan, dimension, feature, resource, price, and billing models; and cost/destructive
permits.

Verification: app/resource identity mismatch, unavailable plan, price and dimension
overflow, conflicting patches, duplicate create, timeout after send, delete
confirmation, and no automatic mutation retries.

Exit criteria: all eight included add-on rows are executable and every potentially
billable transition requires explicit cost authority.

Pentest stop: stop after Commit 13; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 14 - Droplet Read Models And Inventory

Goal: implement complete Droplet inventory before lifecycle mutations.

Deliverables: list/get, neighbors, backups, snapshots, kernels, firewalls, associated
resources, destroy status, and backup-policy reads; complete Droplet, network, disk,
image, region, size, and feature models.

Verification: IPv4/IPv6 parsing, cross-resource IDs, unknown status/features, legacy
kernels, pagination, null/omitted fields, associated-resource bounds, backup policy
coherence, and large inventory decoding.

Exit criteria: every included Droplet read row is executable and reusable by later
mutation postconditions.

Pentest stop: stop after Commit 14; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 15 - Droplet Lifecycle And Actions

Goal: implement Droplet creation, actions, and destruction with strong intent.

Deliverables: create, single/tag actions, standard deletion, associated-resource
preview/status, selective deletion, dangerous deletion, and retry-with-associated
resources; user-data protection; SSH/VPC/image bindings; and action polling.

Verification: required constructor fields, user-data redaction, tag fan-out, wrong
action target, conflicting image/size/network inputs, timeout after send, partial
deletion, dangerous permit, cost permit, and retry prohibition.

Exit criteria: all included lifecycle rows are executable and dangerous or billable
behavior is impossible without the exact explicit permit.

Pentest stop: stop after Commit 15; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 16 - Droplet Workload Metadata

Goal: cover the documented workload-local HTTP API without weakening remote transport
security.

Deliverables: an opt-in metadata service with the exact link-local origin and permitted
paths; typed identity, networking, DNS, tags, features, reserved/floating/virtual IP and
user-data responses; protected handling of user-data and explicit local-workload
authority.

Verification: all source-locked metadata paths; proxy, redirect, alternate host/port,
IPv6 alias, path traversal, oversized plaintext/JSON, secret diagnostic, and
off-workload failures; ensure ordinary clients still reject HTTP.

Exit criteria: metadata has complete path/response coverage, never accepts a
DigitalOcean bearer token, and cannot become a generic link-local fetch facility.

Pentest stop: stop after Commit 16; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 17 - Droplet Autoscale Pools

Goal: implement autoscale pools without unbounded or contradictory policies.

Deliverables: list/get/create/update/delete, member list, history, and dangerous delete
operations; target, min/max, utilization, cooldown, and pool models.

Verification: min/max contradictions, zero/overflow targets, cooldown bounds, member
duplication, history pagination, cost growth, update races, ordinary versus dangerous
delete, and action/result association.

Exit criteria: all eight included autoscale rows are executable with bounded scale and
explicit cost/destructive authority.

Pentest stop: stop after Commit 17; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 18 - Images, Snapshots, And Transfers

Goal: implement image and snapshot inventory, actions, transfers, and deletion.

Deliverables: image/snapshot list/get/create/update/delete; image actions;
account-transfer create/accept/decline/cancel; typed checksums, regions, status, and
transfer tokens; and polling/permit integration.

Verification: URL import policy, checksum syntax, region mismatch, transfer token
redaction, cross-account association, replay, stale actions, deletion authority,
pagination, and deprecated-field compatibility.

Exit criteria: every included image, snapshot, action, and transfer row is executable
without credential or transfer-token leakage.

Pentest stop: stop after Commit 18; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 19 - Block Storage And Volume Actions

Goal: implement volumes, snapshots, and attachment actions coherently.

Deliverables: volume and snapshot CRUD/read operations; name/ID lookup;
attach/detach/resize/action list/get; filesystem and region models; and Droplet-volume
association checks.

Verification: size monotonicity, filesystem grammar, region/Droplet mismatch, name
ambiguity, duplicate attachment, action replay, snapshot lineage, destructive delete,
cost growth, and polling.

Exit criteria: all included block-storage rows are executable with exact resource
association and explicit cost/destructive permits.

Pentest stop: stop after Commit 19; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 20 - Public IP Resources And BYOIP

Goal: implement floating, reserved IPv4/IPv6, action, and BYOIP lifecycles.

Deliverables: Floating IP, Reserved IP, Reserved IPv6, action, and BYOIP operations;
IP/prefix/region/resource models; assign/unassign actions; remote resource listing; and
mutation/destructive permits.

Verification: canonical IP/CIDR parsing, host bits, address-family mismatch, region
mismatch, duplicate assignment, wrong-resource action, prefix ownership, pagination,
delete authority, and action polling.

Exit criteria: all included public-address rows are executable and cannot widen or
reassign a prefix through ambiguous input.

Pentest stop: stop after Commit 20; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 21 - VPCs, Peerings, And NAT Gateways

Goal: implement private networking with canonical ranges and route identity.

Deliverables: VPC, member, peering, VPC-nested peering, and NAT gateway operations;
region/range/route models; patch/update distinctions; and cost/destructive permits.

Verification: canonical network ranges, overlap, cross-region peering, self-peering,
duplicate route, nested/top-level identity equivalence, NAT size and cost, deletion
dependencies, and pagination.

Exit criteria: all included VPC, peering, and NAT rows are executable with fail-closed
topology validation.

Pentest stop: stop after Commit 21; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 22 - Public-Preview VPC Subnets And Routes

Goal: cover preview networking as an explicit API track, not an omission from VPC
support.

Deliverables: subnet lifecycle, membership, and route operations from both new preview
tags; typed address ranges, destinations, next hops, ownership, pagination, and exact
preview stability labels.

Verification: IPv4/IPv6 boundaries, canonical identity, conflicting route targets,
resource mismatches, duplicate members, pagination, destructive changes, and
preview-field drift.

Exit criteria: all eleven survey operations and their locked fields are executable;
documentation states preview instability without withholding supported operations.

Pentest stop: stop after Commit 22; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 23 - Firewalls And Load Balancers

Goal: implement ingress/egress and load-distribution controls safely.

Deliverables: firewall CRUD/rules/tag/Droplet assignment and load-balancer
CRUD/Droplet/forwarding-rule/cache operations; canonical protocol, port, CIDR,
certificate, health, algorithm, and redirect models.

Verification: empty/wide firewall rules, port ranges, conflicting sources, duplicate
assignments, forwarding loops, certificate/protocol mismatch, cache deletion, health
thresholds, cost changes, and destructive permits.

Exit criteria: all included firewall and load-balancer rows are executable and cannot
silently broaden access or traffic exposure.

Pentest stop: stop after Commit 23; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 24 - Domains, Certificates, And CDN

Goal: implement DNS control, certificate lifecycle, and CDN endpoints.

Deliverables: domain/record CRUD, certificate CRUD, and CDN endpoint/cache operations;
DNS names, record values, TTL, certificate/key, origin, and cache models; sensitive PEM
handling; and special CDN rate policy.

Verification: DNS canonicalization, record-type semantics, CAA/MX/SRV bounds,
private-key redaction, certificate chain limits, hostile origin URLs, cache file limits,
five-per-ten-second scheduling, and destructive authority.

Exit criteria: all included domain, record, certificate, and CDN rows are executable
without key leakage or accidental DNS/cache widening.

Pentest stop: stop after Commit 24; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 25 - Partner Network Connect And NFS

Goal: implement partner attachments, credentials, NFS shares, access points, snapshots,
and actions.

Deliverables: all Partner Network Connect and NFS operations; BGP, route, service-key,
share, export, access-point, snapshot, and action models; protected credential outputs;
and cost/destructive permits.

Verification: ASN/key redaction, route canonicalization, cross-region/VPC binding, NFS
path and client range validation, snapshot lineage, action polling, partial deletion,
and response association.

Exit criteria: every included partner-network and NFS row is executable with
credential-safe outputs and exact topology binding.

Pentest stop: stop after Commit 25; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 26 - Security Scans And Uptime

Goal: implement provider security findings and availability monitoring.

Deliverables: security scan/rule/suppression/settings operations and uptime
check/alert/state operations; finding, affected-resource, schedule, endpoint, region,
threshold, and notification models.

Verification: suppression scope, stale scan association, finding bounds, hostile check
URLs, SSRF-sensitive documentation boundaries, interval/timeout coherence, alert
threshold contradictions, delete authority, and pagination.

Exit criteria: all included Security and Uptime rows are executable without silently
suppressing findings or admitting malformed monitored targets.

Pentest stop: stop after Commit 26; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 27 - Secrets Manager

Goal: cover the public secret-management surface absent from the central survey
specification.

Deliverables: source reconciliation with official godo/doctl; secret CRUD, version
listing, restore, region and version preconditions, partial-region results, and
protected value maps. Classify CLI set/unset as endpoint workflows rather than inventing
routes; reconcile the SDK's unusual 204/body behavior before transport admission.

Verification: secret-name encoding, stale versions, concurrent updates, missing regions,
partial results, clear/drop/cancellation, diagnostic redaction, and independently
captured legal HTTP response framing; never relax general 204 framing from a sample
alone.

Exit criteria: every publicly documented operation has locked wire evidence and tests;
unresolved response/auth contracts block this checkpoint instead of excluding Secrets
Manager.

Pentest stop: stop after Commit 27; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 28 - Kubernetes Read And Credential Boundaries

Goal: implement Kubernetes inventory and protect cluster access material.

Deliverables: cluster/node-pool lists and gets, options, upgrades, lint results, status
messages, associated resources, cluster user, kubeconfig, and credential operations;
protected kubeconfig/certificate/token models.

Verification: cluster/node association, region/version coherence, malformed kubeconfig
and PEM, credential expiry, access-scope enforcement, redaction, pagination, and
omission of deprecated recycle behavior.

Exit criteria: all included Kubernetes reads are executable and credential outputs
cannot enter ordinary logs or unrelated requests.

Pentest stop: stop after Commit 28; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 29 - Kubernetes Lifecycle And Integrations

Goal: implement cluster/node-pool lifecycle, upgrades, registry integration, lint runs,
and selective/dangerous destruction.

Deliverables: create/update/delete/upgrade cluster; add/update/delete nodes and node
pools; run lint; registry add/remove; associated-resource deletion; action drivers; and
cost/destructive permits.

Verification: version transitions, surge and autoscale contradictions, node-count cost
bounds, registry identity, deletion previews, selective versus dangerous destruction,
timeout after send, and action polling.

Exit criteria: all included Kubernetes mutations are executable and deprecated recycle
is represented only as an excluded matrix row.

Pentest stop: stop after Commit 29; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 30 - Container Registries

Goal: implement the current plural registry API without preserving superseded singular
endpoints.

Deliverables: all current registry list/get/create/delete, subscription, credential,
validation, repository/tag/manifest, garbage collection, and option operations;
protected Docker credential outputs; and exact supersession mapping.

Verification: registry/repository path encoding, digest grammar, credential
expiry/redaction, subscription cost, garbage-collection state, deletion scope,
pagination, and tests proving all 18 singular operations remain inaccessible.

Exit criteria: all 19 current registry rows are executable and no deprecated route can
be prepared through the public API.

Pentest stop: stop after Commit 30; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 31 - Container Registry Distribution

Goal: make the supported image-transfer protocol executable as well as registry
management.

Deliverables: the provider-supported Docker/OCI HTTP subset, challenge/token exchange,
repository-bound credentials, manifests, blobs, resumable uploads, and digest-checked
downloads; explicit upload session and transactional sink contracts.

Verification: malicious auth realms and Location headers, audience/repository confusion,
cross-origin redirects, digest/length mismatch, chunk offset errors, cancellation,
upload completion ambiguity, and independent registry fixtures.

Exit criteria: the locked provider-supported protocol is covered, secrets never follow
arbitrary challenges, and unsupported upstream OCI extensions are listed separately from
provider gaps.

Pentest stop: stop after Commit 31; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 32 - Spaces Key Control Plane

Goal: implement Spaces access-key management ahead of separately reviewed S3 signing,
bucket, object, and multipart checkpoints.

Deliverables: list/get/create/update/patch/delete Spaces key operations; protected
access-key and secret outputs; bucket/scope models; rotation workflow; and destructive
permits.

Verification: one-time secret capture, source/destination cleanup, scope confusion, key
ID grammar, rotation ordering, partial failure, self-lockout, redaction, and proof that
no S3 object operation is exposed.

Exit criteria: all six included Spaces key rows are executable and generated secrets
remain protected throughout their SDK lifetime.

Pentest stop: stop after Commit 32; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 33 - Spaces Signing And XML Boundaries

Goal: establish independently tested SigV4 and bounded XML before object operations.

Deliverables: regional addressing and signing adapters; exact canonical URI/query/header
and payload rules; protected key ownership, caller-supplied time, bounded XML without
entities or DTDs, and source-locked error decoding. Reuse admitted neutral helpers if
available; do not implement cryptographic primitives.

Verification: independent signing vectors, duplicate query keys, encoded slashes,
virtual-host/path-style rules, skew, wrong region, parser expansion/depth limits, and
failed allocation cleanup.

Exit criteria: Spaces requests cannot receive provider bearer tokens; signing, errors,
and XML have adversarial evidence, and any new dependency has admission review.

Pentest stop: stop after Commit 33; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 34 - Spaces Buckets And Configuration

Goal: cover the supported bucket-level API and its authorization consequences.

Deliverables: the source-listed bucket lifecycle, listing/location, permissions, policy,
CORS, lifecycle, versioning, and other supported configuration operations, each with the
exact supported field subset and destructive/public-access permits.

Verification: supported/unsupported compatibility fixtures, policy and CORS replacement,
version-state transitions, access expansion, truncated XML, pagination tokens, and
bucket/account/region confusion.

Exit criteria: all bucket rows from the Spaces support table are executable or proven
provider-unsupported; no supported row is deferred as an AWS compatibility detail.

Pentest stop: stop after Commit 34; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 35 - Spaces Objects And Versions

Goal: provide bounded object operations with correct integrity and conditional
semantics.

Deliverables: supported object list/get/head/put/copy/delete, version, metadata, tag,
ACL, range, and conditional request contracts; streaming sinks/sources, signed URLs,
exact expiry and length limits.

Verification: binary names and percent encoding, cross-bucket copy policy,
version/delete-marker behavior, preconditions and ranges, pagination, cancellation,
digest mismatches, and metadata/header injection.

Exit criteria: every supported object operation is exercised; multipart ETags are not
represented as universal content checksums and unverified bytes never become committed
output.

Pentest stop: stop after Commit 35; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 36 - Spaces Multipart And Transfer Workflows

Goal: complete large-object transfers without hidden buffering, retries, or leaked
uploads.

Deliverables: supported multipart initiation, part upload/copy/list, completion and
abort; bounded resumable workflow state; cleanup outcomes, explicit retry policy and
presigning, and independently qualified streaming adapter integration.

Verification: partial writes, duplicate/missing parts, maximum part/total sizes,
completion error bodies, uncertain delivery, expired signatures, cancellation, abort
failure, and sink commit failure.

Exit criteria: all multipart rows and transfer states have deterministic fixtures; no
automatic remote cleanup or retry runs without caller authority.

Pentest stop: stop after Commit 36; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 37 - Functions

Goal: implement namespace, trigger, and access-key lifecycle.

Deliverables: namespace and trigger CRUD/read operations; access-key
list/create/update/delete; schedule, function, namespace, route, and protected key
models; and admin-scope/mutation permits.

Verification: namespace/trigger association, schedule grammar, hostile routes, key
one-time output, admin versus granular scope, deletion authority, retries, pagination,
and redaction.

Exit criteria: all 13 included Functions rows are executable with exact namespace
binding and protected credentials.

Pentest stop: stop after Commit 37; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 38 - Functions Execution Data Plane

Goal: cover public function execution separately from namespace and trigger management.

Deliverables: the documented OpenWhisk-compatible action/package/activation subset,
invocation, logs, and publicly supported configuration; namespace key and returned
service endpoint binding, bounded code/payload handling, and explicit invocation cost
authority.

Verification: wrong namespace/host/key, action replacement, blocking/nonblocking
invocation, activation/result association, truncated logs, timeouts, cancellation, and
payload/code redaction.

Exit criteria: official Functions documentation and SDK/CLI capabilities reconcile to
the ledger; a working namespace client alone cannot establish Functions completeness.

Pentest stop: stop after Commit 38; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 39 - Apps Reads, Logs, Metrics, And Console Access

Goal: implement App Platform inspection and sensitive runtime access.

Deliverables: app/deployment/event/job/instance/alert/health/region/size reads; logs and
aggregate logs; bandwidth metrics; exec and active-deployment exec; bounded streaming
and protected console/session outputs.

Verification: app/deployment/component association, log injection and bounds, cursor
pagination, event ordering, metric ranges, console credential redaction, expiry,
cancellation, and access-console scope.

Exit criteria: all included App read/log/metric/exec rows are executable and runtime
access material cannot leak or cross app boundaries.

Pentest stop: stop after Commit 39; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 40 - Apps Lifecycle, Deployments, Jobs, And Rollbacks

Goal: implement App Platform mutations and rollback state machines.

Deliverables: app create/update/delete/restart; deployment creation/cancel; event and
job cancellation; job invocation; alert destinations; database trusted source; app-spec
validation; rollback create/validate/commit/revert; and cost/destructive permits.

Verification: app-spec bounds and secrets, deployment races, job replay, rollback
ancestry, validate-before-commit, cancel terminal states, database association, timeout
after send, cost changes, and action polling.

Exit criteria: all included App mutations are executable through explicit stateful
workflows and no deployment or rollback runs implicitly.

Pentest stop: stop after Commit 40; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 41 - Logs, Console, And HTTP Upgrade Sessions

Goal: close streaming and session gaps hidden behind URL-returning control endpoints.

Deliverables: source-locked log/console session protocols and supported HTTP upgrades;
explicit transport features, bounded frames, backpressure, credential expiry,
deadline/cancellation, and typed session intent. User application protocols remain
outside scope.

Verification: upgrade status/header validation, cross-origin session tokens,
oversized/fragmented frames, termination, slow consumers, cancellation, and neutral
adapter isolation.

Exit criteria: each documented provider session protocol has a tested execution path or
evidence it only describes an external native application protocol; returned URLs alone
do not count as session support.

Pentest stop: stop after Commit 41; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 42 - Database Catalog, Clusters, And Sensitive Reads

Goal: implement database inventory and credential-bearing reads first.

Deliverables: options, cluster/replica/user/pool/firewall/backup/event/log-sink, Kafka,
OpenSearch, config, autoscale, migration, CA, and metric-credential reads;
engine/version/region/size models; and protected credentials.

Verification: engine-specific unions, preview edition status, cluster/resource
association, credential/CA redaction, pagination, malformed config, unknown versions,
backup lineage, and bounded logs/events.

Exit criteria: every included database read row is executable and all credential outputs
are context-bound and protected.

Pentest stop: stop after Commit 42; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 43 - Database Cluster, User, Pool, And Replica Lifecycle

Goal: implement core database mutations with explicit cost and credential handling.

Deliverables: cluster/replica/user/pool create/update/delete; resize, region,
maintenance, major-version, auth reset, promotion, firewall, SQL mode, eviction, and
autoscale operations; state drivers; and cost/destructive permits.

Verification: engine/operation compatibility, resize cost, irreversible upgrade, region
migration, maintenance windows, password rotation cleanup, replica promotion, firewall
widening, timeout after send, and polling.

Exit criteria: all included core database mutations are executable and every costly,
destructive, or credential-changing transition is explicit.

Pentest stop: stop after Commit 43; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 44 - Database Kafka, Migration, Config, And Logs

Goal: finish specialized database APIs without generic untyped maps leaking into the
public surface.

Deliverables: Kafka schema/topic CRUD/config, online migration, engine config,
DigitalOcean settings, log sink, update installation, and OpenSearch index operations;
typed engine-specific request families and secret-safe fields.

Verification: schema/topic names, version identity, compatibility modes, migration
credentials, config key/type bounds, unknown settings, log sink targets, update state,
index deletion, and operation coverage.

Exit criteria: all remaining included database rows are executable with typed engine
contracts and no unbounded configuration object.

Pentest stop: stop after Commit 44; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 45 - Vector Databases

Goal: implement vector database lifecycle, backups, restore, resize, and credentials.

Deliverables: list/get/create/update/delete, credentials, backups, restore status,
restore, resize, and tag operations; engine, node, backup, status, and protected
credential models.

Verification: dimension/node/size bounds, cost changes, backup lineage, cross-database
restore, credential redaction, tag replacement, destructive delete, polling, and
timeout-after-send behavior.

Exit criteria: all 11 included vector database rows are executable with exact backup
association and explicit cost/destructive authority.

Pentest stop: stop after Commit 45; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 46 - Droplet, App, And Autoscale Metrics

Goal: implement the first bounded Monitoring metric families.

Deliverables: Droplet CPU/bandwidth/filesystem/load/memory, App CPU/memory/restart, and
autoscale current/target metric operations; timestamp ranges, metric series, labels,
samples, and precision-safe values.

Verification: range ordering, maximum windows, NaN/infinity rejection, label bounds,
duplicate/out-of-order samples, resource association, empty series, large responses, and
operation coverage.

Exit criteria: every included metric row in these families is executable through shared
bounded metric models.

Pentest stop: stop after Commit 46; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 47 - Database And Load Balancer Metrics

Goal: implement the remaining high-volume metric families without copy-pasted decoders.

Deliverables: MySQL database and load-balancer Droplet/frontend metrics; metric-name
associations; percentile, response, throughput, TLS, firewall, health, and connection
series; and shared query preparation.

Verification: metric/label mismatch, percentile semantics, counter versus gauge, network
units, sample ordering, oversized series, unknown metrics, exact operation mapping, and
transport parity.

Exit criteria: every included database and load-balancer metric row is executable and
generated associations prevent endpoint/model swaps.

Pentest stop: stop after Commit 47; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 48 - Monitoring Alerts, Destinations, And Sinks

Goal: complete Monitoring control-plane mutations and reads.

Deliverables: alert policy, destination, and sink list/get/create/update/delete
operations; expression, threshold, comparison, window, channel, endpoint, and credential
models; and mutation/destructive permits.

Verification: expression bounds, threshold/time coherence, hostile webhook targets,
secret redaction, duplicate channels, sink resource scope, no-op patches, delete
authority, rate limits, and response association.

Exit criteria: all remaining included Monitoring rows are executable without silently
widening alert delivery or exposing destination credentials.

Pentest stop: stop after Commit 48; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 49 - Dedicated Inference

Goal: implement dedicated inference resources and token lifecycle.

Deliverables: list/get/create/update/delete, sizes, accelerators, GPU config, CA, and
token list/create/delete operations; accelerator/region/model/status models; protected
inference tokens; and cost/destructive permits.

Verification: model/accelerator/size compatibility, capacity and cost bounds, CA/token
redaction, token one-time output, expiry, deletion authority, polling, and
dedicated-inference scope mapping.

Exit criteria: all 13 included dedicated inference rows are executable and credential
outputs remain protected.

Pentest stop: stop after Commit 49; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 50 - GradientAI Workspaces, Catalogs, And Provider Keys

Goal: establish GradientAI shared models, workspaces, catalogs, regions, and external
provider credentials.

Deliverables: workspace CRUD/list/get, region/model/catalog/card lists, OpenAI and
Anthropic key list/get/create/update/delete, current model-key operations, and protected
provider-key models.

Verification: workspace/model association, lifecycle status including preview, provider
key redaction and rotation, wrong-provider substitution, pagination, region mismatch,
and proof the retired model-key operation is inaccessible.

Exit criteria: all assigned GradientAI foundation rows are executable and external
provider secrets cannot escape their context.

Pentest stop: stop after Commit 50; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 51 - GradientAI Agents, Versions, Functions, And Guardrails

Goal: implement agent lifecycle and composition safely.

Deliverables: agent list/get/create/update/delete, children, usage, versions, rollback,
API keys, deployment visibility, agent attach/detach, functions, guardrails, workspace
moves, and provider-key filtered lists.

Verification: agent graph cycles, parent/workspace mismatch, rollback ancestry, function
schema bounds, guardrail attachment, visibility transitions, API-key rotation/redaction,
delete authority, and pagination.

Exit criteria: all assigned agent rows are executable with acyclic identity and
protected key handling.

Pentest stop: stop after Commit 51; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 52 - GradientAI Knowledge Bases And Indexing

Goal: implement knowledge bases, data sources, uploads, and indexing jobs.

Deliverables: knowledge base/data source CRUD and attachment; indexing job
create/list/get/cancel; scheduled indexing; source-specific models; presigned
upload/download URLs; and bounded job polling.

Verification: source/knowledge-base association, path and bucket bounds, presigned
authority/expiry/replay, no bearer forwarding, indexing state regression, cancellation,
schedule coherence, and signed-result URL handling.

Exit criteria: all assigned knowledge/indexing rows are executable and every returned
URL is used through a credential-isolated checked path.

Pentest stop: stop after Commit 52; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 53 - GradientAI Evaluation

Goal: implement evaluation datasets, test cases, metrics, presets, and runs.

Deliverables: dataset/test-case/custom-metric/evaluation-run CRUD and execution;
presigned dataset uploads/downloads; presets and result retrieval; prompt result models;
cancellation; and bounded asynchronous drivers.

Verification: dataset/test-case/run association, metric type/range, prompt and result
bounds, presigned URL isolation, cancel/terminal races, result download size,
pagination, and destructive permits.

Exit criteria: all assigned evaluation rows are executable with exact lineage and
bounded sensitive content.

Pentest stop: stop after Commit 53; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 54 - GradientAI Custom Models, Routers, And Integrations

Goal: complete GradientAI custom models, routers, scheduled behavior, and OAuth
integration endpoints.

Deliverables: custom model import/get/list/update/delete; router/preset/task preset
list/get/create/update/delete; Dropbox OAuth URL/token operations; and remaining
assigned GradientAI rows with exact lifecycle models.

Verification: model source and checksum, router cycles/weights, preset identity, OAuth
state/redirect binding, integration token redaction, schedule conflicts, delete
authority, and generated coverage for all assigned GradientAI rows.

Exit criteria: every assigned GradientAI operation is executable and no retired or
unassigned row remains.

Pentest stop: stop after Commit 54; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 55 - GradientAI Scenario Libraries And Sets

Goal: implement the newly documented scenario authoring and transfer surface.

Deliverables: library reads and scenario-set creation, generation, CRUD, duplication,
item pagination, and upload/download URL workflows; resource associations and bounded
protected scenario content.

Verification: set/library mismatch, generation cost, unknown item variants, duplicate
pages, presigned expiry/scope, upload integrity and partial failure, destructive delete,
and errors embedded in nominal success.

Exit criteria: every scenario operation and field is linked to execution tests, with no
automatic fetching of returned content.

Pentest stop: stop after Commit 55; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 56 - GradientAI Simulation Runs And Results

Goal: make simulation lifecycle and nested result access explicit and bounded.

Deliverables: run CRUD/start/cancel, journeys, trajectory and trajectory-URL access;
billing intent, asynchronous state transitions, resource-bound result retrieval, and
bounded output.

Verification: run/journey mismatch, contradictory terminal states, cancellation races,
large traces, unexpected result URLs, polling budgets, and uncertain delivery of run
creation.

Exit criteria: simulation rows are executable and result retrieval cannot escape the
run's reviewed authority or cost policy.

Pentest stop: stop after Commit 56; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 57 - Action Gateway Catalog And Toolbelts

Goal: support the new gateway catalog and tool selection without executing tools
implicitly.

Deliverables: provider/toolkit/tool definitions and toolbelt lifecycle/tool association
operations; bounded dynamic schemas, identifiers, pagination, and separate
inspection/mutation permissions.

Verification: schema depth and size, unexpected catalog variants, cross-toolbelt
association, duplicate tools, untrusted tool descriptions, and public URL handling.

Exit criteria: all catalog/toolbelt rows have full request and response coverage; SDK
parsing cannot invoke a discovered tool or fetch a schema URL.

Pentest stop: stop after Commit 57; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 58 - Action Gateway Connections, Users, And Sessions

Goal: cover connection and session management while separating third-party authority.

Deliverables: connection lifecycle, user inspection, and session lifecycle operations;
protected connection material, expiry/revocation states, and account/toolbelt/session
associations.

Verification: cross-user session binding, credential echoes, stale connections,
concurrent revocation, forbidden redirects, permission escalation, and non-idempotent
retry suppression.

Exit criteria: all remaining gateway operations are executable; documentation separates
gateway administration from arbitrary third-party API automation.

Pentest stop: stop after Commit 58; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 59 - Serverless, Embedding, And Agent Inference

Goal: implement direct inference on its separate authorities with bounded buffered
responses and separate credential classes.

Deliverables: serverless model list, chat, messages, responses, images, async invoke,
embeddings, and customer Agent Inference; inference/agent credentials; bounded buffered
responses, usage and finish models; streaming variants are inventoried here and
completed in Commit 60.

Verification: authority/credential mismatch, hostile agent host, prompt and media
bounds, malformed response variants, cancellation, token usage overflow, model mismatch,
response limits, and no automatic mutation retry.

Exit criteria: all included buffered serverless, embedding, and Agent Inference variants
are executable without cross-authority credential leakage.

Pentest stop: stop after Commit 59; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 60 - Inference Streaming And Compatibility Variants

Goal: qualify streamed model output independently from JSON request models.

Deliverables: all provider-supported completion, embedding, agent and other
source-locked inference variants; bounded SSE/event framing, typed termination/error
events, usage accounting, cancellation and synchronous/async adapter integration.

Verification: every byte-split boundary, invalid UTF-8, oversized events, fragmented
terminal markers, provider errors after partial output, slow streams, cancellation and
wrong model/agent credentials.

Exit criteria: buffered and streamed variants both have client evidence; upstream OpenAI
features not supported by DigitalOcean are not implied.

Pentest stop: stop after Commit 60; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 61 - Batch Inference And Presigned Uploads

Goal: implement batch files, jobs, results, cancellation, and one-time uploads.

Deliverables: create/upload batch file, create/list/get/cancel batch, result retrieval,
short-lived presigned upload request, streaming input/results, and bounded batch state
polling.

Verification: exact returned URL validation, no DigitalOcean bearer on upload,
expiry/replay, method/content-type binding, file and result size limits, partial upload,
cancellation, job/file association, and terminal-state coherence.

Exit criteria: all seven included batch rows are executable and presigned requests
cannot become general-purpose authenticated HTTP calls.

Pentest stop: stop after Commit 61; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 62 - Paperspace Authentication And Account Catalog

Goal: admit the separate current Paperspace API inside the same provider crate.

Deliverables: locked current reference coverage for authentication/session, teams,
users, tags and catalog primitives; distinct credentials and origin, pagination/error
rules, current/retired source reconciliation, and module features.

Verification: bearer confusion with DigitalOcean, null session, team/user mismatch,
scope checks, nested response bounds, and tests keeping retired Core/Gradient routes
non-executable.

Exit criteria: all current account/catalog rows are owned and tested; legacy
documentation cannot silently resurrect retired endpoints.

Pentest stop: stop after Commit 62; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 63 - Paperspace Machines And Access

Goal: cover current compute inventory, lifecycle and access control.

Deliverables: machine/type/template reads, machine lifecycle and events, accessors,
desktop-settings responses, typed state and cost permits, and protected returned
credentials.

Verification: machine/accessor ownership, action association, conflicting terminal
states, polling budgets, cost-changing updates, unsupported retired types, and
credential redaction.

Exit criteria: all current machine rows are executable; desktop settings do not imply
implementation of native remote-desktop clients.

Pentest stop: stop after Commit 63; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 64 - Paperspace Images, Snapshots, And Startup Scripts

Goal: complete reusable compute artifacts and their sensitive configuration.

Deliverables: custom/OS templates, snapshot lifecycle, startup-script lifecycle and
associations, exact update semantics, bounded script bodies, and provenance/restore
links.

Verification: cross-machine restore, template permissions, destructive deletion, omitted
versus cleared scripts, payload limits, allocation errors, and diagnostics containing
script secrets.

Exit criteria: every current artifact/configuration row is executable and sensitive
configuration follows protected-buffer rules.

Pentest stop: stop after Commit 64; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 65 - Paperspace Networks And Storage

Goal: cover current network and storage management without inventing native file
clients.

Deliverables: private networks, public IPs, shared drives, storage resources,
attachment/ownership rules, lifecycle actions, and capacity/cost metadata.

Verification: cross-team attachment, address and size bounds, repeated/destructive
actions, partial responses, cancellation and billing transitions.

Exit criteria: all current network/storage rows have typed execution tests; storage
management is distinct from native filesystem access.

Pentest stop: stop after Commit 65; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 66 - Paperspace Projects And Membership

Goal: complete project administration and resource association independently of machine
access.

Deliverables: all current project operations and nested membership/resource
associations; exact role, ownership and authorization models with explicit
access-changing permits.

Verification: cross-project identifiers, duplicate membership, role changes, stale
associations, pagination, and attempts to infer write authority from a read response.

Exit criteria: all project rows, including nested operations, have coverage and cannot
be hidden behind a generic project CRUD claim.

Pentest stop: stop after Commit 66; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 67 - Paperspace Datasets, Models, And Versions

Goal: support current data/model lifecycle and transfer contracts.

Deliverables: dataset, model, version and associated documented transfer operations;
immutable/versioned identifiers, bounded metadata, protected transfer credentials, and
checksum/transactional sink integration where specified.

Verification: version mismatch, transfer URL confusion, partial upload/download, hostile
metadata, allocation limits, cancellation, and delete/update ambiguity.

Exit criteria: every current data/model/version row has executable evidence; retired
Gradient-only features remain explicitly retired rather than assumed current.

Pentest stop: stop after Commit 67; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 68 - Paperspace Deployments And Registry Integration

Goal: complete current deployment and external registry configuration.

Deliverables: deployment lifecycle/configuration, documented logs and scaling controls,
registry records/validation, protected endpoint material, and cross-resource
associations; exact supported payload variants from current references.

Verification: malicious registry/health-check URLs treated as inert configuration,
secret redaction, cost bounds, replacement semantics, rollout failure, and explicitly
approved endpoint invocation.

Exit criteria: all current deployment/registry rows are covered; user-defined
model/application routes are not invented as provider API methods.

Pentest stop: stop after Commit 68; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 69 - OAuth Authorization Code, Refresh, And Revocation

Goal: support secure delegated authorization without exposing the implicit grant.

Deliverables: authorization-code URL builder with required state; callback validation;
token exchange; single-use refresh rotation; revocation; client-secret protection; exact
redirect URI binding; and optional PKCE only if source-locked as supported. Resolve the
documentation's revocation-host typo and token-example query-secret handling before
constructing executable requests; use verified endpoints and a supported secret-safe
exchange. Without verified PKCE, do not claim a secure public-client authorization flow.

Verification: CSRF state mismatch, redirect confusion, code replay, client secret
leakage, refresh races, old-token retirement, revocation authority, error responses, URL
redaction, and compile-fail rejection of implicit token callbacks.

Exit criteria: all admitted OAuth workflows are executable, refresh rotation is atomic,
and no public API supports `response_type=token`.

Pentest stop: stop after Commit 69; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 70 - Unified Client And Workflow Coverage

Goal: make the checked path the easiest path for every included operation.

Deliverables: official provider client constructors; operation-to-prepared request
bindings for every matrix row; automatic method, authority, scope, headers, body,
response bound, and decoder selection; sync, local-async, async, raw, and streaming
parity; and high-level create/poll and rotate workflows.

Verification: generated coverage assertions; compile-checked examples; credential, cost,
and permit routing; concurrency and cancellation; checked response decoding; returned
URL handling; and proof no supported operation requires manual HTTP assembly. Include
every separately inventoried product and protocol, with safe constructors and
source-derived examples; generated request support alone never establishes executable
coverage.

Exit criteria: every included matrix row is executable through the official client and
all operation associations have independent test evidence.

Pentest stop: stop after Commit 70; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 71 - Full Public Surface Reconciliation

Goal: prove that implementation matches the entire public provider scope, not just the
first source snapshot.

Deliverables: fresh central, product, Paperspace, SDK/CLI, protocol and lifecycle
observations; independent operation/field/execution reconciliation; closure records for
source disagreements and all excluded deprecated/private/unsupported rows.

Verification: delete a nested option, metric, public-preview operation, external
protocol, error variant or client mapping and require gate failure; compare the live
catalog against both the initial lock and the implemented ledger.

Exit criteria: zero unknown, unowned, unsupported-by-SDK public rows remain. New public
APIs discovered during the train require extra reviewed checkpoints before final
qualification, not an automatic post-release deferral.

Pentest stop: stop after Commit 71; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 72 - Opt-In Live Workflows And Cleanup Evidence

Goal: test real service behavior without exposing routine CI to credentials or bills.

Deliverables: least-privilege read-only smoke paths for each credential class, explicit
separate approvals and budgets for mutations/billable reads, resource ownership tags,
cleanup ledger and failure instructions; public-preview/access-limited evidence labels.

Verification: no-token default runs, wrong-identity refusal, rate ceilings, quota/budget
exhaustion, credential leakage checks and deterministic simulated failure; approved live
create/read/update/delete workflows only on disposable resources.

Exit criteria: all executed probes and unavailable account/region capabilities are
reported honestly; missing live access never becomes a false green, and public contract
coverage is not narrowed to the available account.

Pentest stop: stop after Commit 72; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 73 - Fuzzing And Adversarial Protocol Qualification

Goal: exercise parser, signing, streaming and ownership boundaries beyond happy-path
fixtures.

Deliverables: bounded fuzz targets and deterministic corpora for JSON/XML, unions,
URI/signing, metadata, OAuth, registry challenges, Functions, Paperspace, SSE/upgrade
framing and returned URLs; allocation, timeout and cancellation fault injection.

Verification: compile every target with warnings denied, run bounded campaigns and
exact-bound seeds, compare applicable parsers/signatures against independent oracles,
and enforce fail-closed tests with complete target inventory.

Exit criteria: all admitted protocols have adversarial evidence with minimized
regression cases; default CI remains offline and cannot execute real mutations.

Pentest stop: stop after Commit 73; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 74 - Platforms, Dependencies, And Package Qualification

Goal: qualify the complete feature graph without weakening the stable core.

Deliverables: MSRV/current Rust, documented native/portable targets and execution-mode
evidence; admitted dependency/features, licenses, advisories, tooling freshness, SBOMs,
crate READMEs/examples and package contents.

Verification: default/alloc/Serde/transport feature combinations; no_std checks,
all-target forbidden dependency graphs, Linux/Windows/macOS and documented BSD/mobile
evidence, doc tests, file length checks, package verification and reproducibility.

Exit criteria: support claims match available evidence, all package relationships are
valid, and new protocol dependencies remain optional and neutral where reusable; FIPS
remains deferred to qualified Brynja work.

Pentest stop: stop after Commit 74; pentest the entire delta from the preceding accepted
checkpoint, fix and retest findings, record the accepted SHA, and wait for green GitHub
CI and CodeQL before the next checkpoint.

## Commit 75 - Scope Freeze And Release Candidate

Goal: qualify the full DigitalOcean provider without changing its scope during release.

Deliverables: final versioned public operation/field/protocol matrix, user documentation
and examples, threat model and migration notes, source-lock/drift maintenance commands,
reproducible package evidence, and one final candidate gate.

Verification: rerun the preceding 74 checkpoint gates plus candidate-specific checks
without recursive self-invocation; fresh upstream drift, complete workspace/provider
suites, execution/feature/platform matrices, dependency/SBOM review, public API/SemVer
review, and two clean-clone reproductions.

Exit criteria: every current public source-locked contract is executable and documented,
every exclusion has evidence, all findings are closed, and any change after
qualification invalidates the affected evidence. No version, tag or publication is
authorized by this plan.

Pentest stop: run the full-provider pentest and retest for the exact Commit 75
candidate, rerun complete qualification, and wait for green GitHub CI and CodeQL. Only
then request a separate release/version/tag/publication decision.

## Exclusions And Full-Scope Boundaries

The following need evidence-backed ledger dispositions rather than executable
provider methods:

- Private, undocumented, console-only, internal test and retired APIs; deprecated
  rows retain lifecycle and replacement evidence. Public restricted-access
  contracts are not excluded merely because credentials are unavailable.
- Retired Paperspace Core/Gradient endpoints. Their online documentation is not
  evidence of current availability.
- Provider-unsupported portions of S3, OCI, OpenWhisk or inference compatibility
  protocols. Every claimed omission must cite the supported subset.
- Native hosted-engine protocols, SSH, NFS, remote desktop implementations,
  Kubernetes' own API server, and user-deployed application routes. Managing
  their provider resources, credentials and documented provider sessions remains
  in scope.
- Browser-cookie ingestion, console scraping, implicit OAuth, legacy SigV2,
  arbitrary URL execution, automatic credential forwarding, and unattended
  retries/mutations. These are deliberate security boundaries, not missing
  modern endpoint coverage.

No current public API may be moved here simply because it is expensive or
difficult to implement. Resolve source ambiguity or extend the checkpoint
train. An API discovered before final qualification gets a reviewed scope
amendment; changes after the final source lock become tracked maintenance work,
and any detected pre-publication drift requires an explicit candidate decision.

## Release Decision

No release version is assigned. After Commit 75 has accepted full-provider
pentest evidence, complete qualification, green GitHub CI and CodeQL,
maintainers select the appropriate SemVer versions and approve publication.
The provider receives its independently appropriate crate version; neutral
crates change only under the existing workspace version policy. Nothing in
this document tags, publishes, or marks DigitalOcean as already supported.
