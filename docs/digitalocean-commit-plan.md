# DigitalOcean Commit Plan

Status: provider candidate assessment; this plan does not select DigitalOcean
as the next provider and does not assign a release version.

## Decision Summary

The estimated implementation train is **49 planned commits** followed by one
provider release. These are logical reviewed implementation commits, not a
promise that remediation, documentation corrections, or merge mechanics will
produce exactly 49 Git objects. Security fixes may add commits without changing
the numbered scope.

The target is every non-deprecated operation in the source-locked public
DigitalOcean OpenAPI document, plus the documented secure OAuth authorization
code, token exchange, refresh, and revocation workflows. The preliminary
inventory contains 659 OpenAPI operations: 639 current candidates and 20
explicitly deprecated operations. Commit 1 makes the authoritative stability
and inclusion decision before implementation begins.

Every numbered commit is an implementation stop. It must be locally green and
receive an incremental pentest against the preceding accepted commit before
the next numbered commit begins. The final commit additionally receives a full
provider pentest and final retest before any version is selected, tagged, or
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

Only the final qualified provider candidate receives a release decision and,
if approved, a signed release tag and crate publication.

## Preliminary Survey

This assessment was performed on 2026-08-20 against:

- the daily official [DigitalOcean OpenAPI document][digitalocean-openapi];
- the official
  [DigitalOcean API reference](https://docs.digitalocean.com/reference/api/reference/);
- the official
  [DigitalOcean OAuth API](https://docs.digitalocean.com/reference/api/oauth/);
- the official
  [DigitalOcean token-scope reference](https://docs.digitalocean.com/reference/api/scopes/);
- official [`digitalocean/openapi`](https://github.com/digitalocean/openapi)
  commit `a000023e3b90b13f4124bbcdeb80996f1c3f58a3` from 2026-08-14 as
  secondary source evidence; and
- [RFC 9700](https://www.rfc-editor.org/rfc/rfc9700.html) for current OAuth
  security guidance.

The observed OpenAPI 3.0 document exposed 445 paths, 659 operations, and 902
schemas. It had no missing or duplicate operation IDs. The operation methods
were 340 `GET`, 142 `POST`, 97 `DELETE`, 65 `PUT`, and 15 `PATCH`.

The largest areas were GradientAI Platform with 119 operations, Monitoring
with 72, Databases with 71, Apps with 38, Kubernetes with 28, the current
plural Container Registries API with 19, Droplets with 19, Functions with 13,
Dedicated Inference with 13, and several networking and storage groups.

The source exposes four distinct authority patterns:

- 644 operations inherit `https://api.digitalocean.com`;
- 13 operations use `https://inference.do-ai.run`;
- one Agent Inference operation uses a customer-specific
  `*.agents.do-ai.run` authority; and
- one batch upload operation consumes a short-lived presigned URL returned by
  DigitalOcean rather than a constructed provider URL.

DigitalOcean bearer tokens carry exact endpoint scopes. The official reference
also documents a 5,000-request hourly limit, a 250-request-per-minute burst
limit, response quota headers, and stricter endpoint-specific limits for SSH
key listing and CDN operations. Pagination normally uses `page`, `per_page`,
`meta.total`, and absolute `links.pages` URLs, with a documented maximum of 200
items per page.

The 20 deprecated operations are the Kubernetes node-pool recycle endpoint,
18 superseded singular Container Registry operations, and one retired
GradientAI model-key operation that returns `410 Gone`. They remain in the
matrix and drift evidence but are not exposed as executable public SDK calls.

The documented OAuth API supports both authorization-code and implicit flows.
RFC 9700 advises clients not to use the implicit grant because of token leakage
and replay risks. This plan therefore supports the authorization-code flow and
explicitly rejects implicit token responses. PKCE is admitted only if Commit 1
finds authoritative DigitalOcean support rather than assuming it.

The survey copies had these SHA-256 digests:

- OpenAPI JSON:
  `f177bd859de74cfa57a08bbf0d7b590e38b5c6c736baee11925d246c8c97c1bd`;
- OAuth documentation:
  `4bc8e86a006d1b212fdc89b47be7fc30debf4f1a1ef86db459f7009935e87ce4`;
- token-scope documentation:
  `3133e331421e1795626c9521bc28b0bfa6f1d5617759f72a53bbdcb14f4adede`.

These values are sizing evidence only. Commit 1 creates the maintained locks.

[digitalocean-openapi]:
  https://docs.digitalocean.com/reference/api/reference/openapi.json

## Scope Rules

1. One provider maps to one crate: `cloud-sdk-digitalocean`.
2. Provider-neutral transport, sanitization, testkit, polling, pagination, and
   execution controls remain in the existing neutral crates.
3. Default provider features remain empty and `no_std` compatible.
4. No generated source file or hand-written code file may exceed 500 lines.
5. Every included operation receives request, response, error, scope, retry,
   idempotency, cost, and execution-authority classifications.
6. Mutating, destructive, credential-returning, and billable operations never
   execute implicitly.
7. Official endpoints are the safe default. Custom endpoints are explicit and
   never receive credentials derived from untrusted configuration.
8. Absolute pagination and returned upload URLs are revalidated before use.
9. Presigned upload requests never receive DigitalOcean bearer credentials.
10. Deprecated operations are inventoried but not publicly executable.
11. Spaces key management is included; the S3-compatible object data plane is
    excluded from this provider release.
12. OAuth implicit grant support is excluded.
13. Coverage is claimed only for rows in the committed API matrix.
14. A numbered commit cannot widen scope assigned to a later commit.

## Commit 1 - Source Lock And Finite Scope

Goal: establish the exact DigitalOcean support claim before provider code
exists.

Deliverables: bounded retrieval of the daily OpenAPI document, API reference,
OAuth guide, scope catalog, and relevant source evidence; exact digest, path,
method, operation ID, tag, server, stability, deprecation, scope, request,
response, and protocol records; and an included/deferred/excluded/superseded
matrix for every row.

Verification: reject cross-origin redirects, malformed JSON or Markdown,
unresolved references, duplicate identities, malformed servers, unknown auth
schemes, undocumented scope combinations, and unclassified rows; independently
reproduce all source digests and the 659-operation preliminary count.

Exit criteria: the exact included count is reviewable, all 20 deprecated rows
are classified, every preview or limited-access contract has an explicit
decision, and any estimate change is recorded here before implementation.

Pentest stop: run an incremental pentest for the exact Commit 1 source-lock and
scope-classification boundary.

## Commit 2 - Drift, Changelog, And Lifecycle Detection

Goal: turn upstream changes into fail-closed maintenance events.

Deliverables: a DigitalOcean adapter for the neutral drift engine; operation,
parameter, schema, authority, auth-scope, rate, pagination, deprecation, and
protocol fingerprints; documentation and repository observations; and a
reviewed lock-refresh workflow.

Verification: fixtures for added, removed, moved, deprecated, retired, and
changed operations; scope and requiredness changes; source disagreement;
redirect, timeout, size, and parser failure; and local-only plus live modes that
never accept drift automatically.

Exit criteria: CI detects every matrix category, including returned-authority
contract changes, and cannot turn an incomplete observation into a green run.

Pentest stop: run an incremental pentest for the exact Commit 2 drift and
lifecycle boundary.

## Commit 3 - Crate, Identity, And Module Boundaries

Goal: add DigitalOcean without coupling provider behavior to neutral core code.

Deliverables: `cloud-sdk-digitalocean`; provider/service identities; empty
default features; bounded product modules; feature ownership for models, Serde,
and transport adapters; README, package metadata, licensing, and docs.rs setup.

Verification: default/all-feature and `no_std` builds; external identity tests;
forbidden dependency graphs; package contents; file-length policy; and proof
that unrelated provider crates do not depend on DigitalOcean code.

Exit criteria: the crate is independently consumable, contains no endpoint
implementation yet, and preserves workspace platform and dependency contracts.

Pentest stop: run an incremental pentest for the exact Commit 3 crate topology.

## Commit 4 - Authorities, Redirects, And Returned URLs

Goal: make every admitted authority safe before credentials or operations
exist.

Deliverables: constructors for the control plane, serverless inference, Agent
Inference suffix-bound hosts, OAuth authority, and presigned upload targets;
custom endpoint policy; redirect rules; and credential-stripping transitions.

Verification: host, port, user-info, Unicode, suffix confusion, encoded
separator, downgrade, redirect, DNS-independent authority, wildcard, presigned
query, replay, expiry, and wrong-service tests.

Exit criteria: credentials cannot cross authorities, agent hosts cannot escape
the reviewed suffix, and presigned uploads use only exact returned HTTPS URLs.

Pentest stop: run an incremental pentest for the exact Commit 4 authority and
returned-URL boundary.

## Commit 5 - Credentials, OAuth Secrets, And Rotation

Goal: represent every credential class without accidental copying or leakage.

Deliverables: protected personal, OAuth access, refresh, inference, agent,
database, registry, Kubernetes, Spaces, and presigned credential types; guarded
ingestion, rotation, expiry, erasure, redacted diagnostics, and authority/scope
binding.

Verification: prefix, length, control byte, CRLF, empty, wrong-context, clone,
drop, refresh race, source cleanup, expired token, duplicate header, Debug,
Display, and error-chain redaction tests.

Exit criteria: secret classes are non-interchangeable, never stored as ordinary
owned strings by the SDK, and cannot be attached to an unrelated authority.

Pentest stop: run an incremental pentest for the exact Commit 5 credential and
rotation surface.

## Commit 6 - Scope, Permit, Cost, And Retry Metadata

Goal: bind least privilege and execution intent to every operation.

Deliverables: source-generated scope constants and operation associations;
read, mutation, destructive, credential-view, and cost permits; retry and
idempotency classifications; and compile-time operation/preparation bindings.

Verification: all included rows have exact scope evidence; scope aliases cannot
hide a missing granular scope; mutations cannot use read permits; non-idempotent
creates and actions never retry automatically; and no operation is model-only.

Exit criteria: coverage tooling rejects any operation lacking scope, authority,
cost, retry, permit, request, response, or error classification.

Pentest stop: run an incremental pentest for the exact Commit 6 authority and
operation-metadata surface.

## Commit 7 - Wire, Errors, Pagination, And Rate Limits

Goal: define one checked wire contract for the complete provider.

Deliverables: common success/error envelopes; optional request IDs; content
types; bounded bodies; empty `204` handling; absolute-link pagination; quota
headers; hourly, burst, SSH-key, and CDN limits; and payload-free public errors.

Verification: status/content-type matrices, malformed and duplicate JSON,
unknown fields, oversized bodies, invalid links, header duplication, reset
rollback, quota arithmetic, 429 `Retry-After`, concurrent scheduling, and
redacted diagnostics.

Exit criteria: errors cannot be decoded as success, pagination cannot escape an
authority, and all documented rate policies are representable and enforceable.

Pentest stop: run an incremental pentest for the exact Commit 7 wire, error,
pagination, and quota surface.

## Commit 8 - Actions And Asynchronous Resource Drivers

Goal: support DigitalOcean action resources and long-running operations once.

Deliverables: action list/get models; status, timestamps, resource associations,
and error states; bounded polling; cancellation, timeout, progress, and terminal
state policies; and provider-specific action adapters.

Verification: stale/regressing actions, unknown states, wrong-resource
association, busy loops, rate exhaustion, timeout, cancellation, duplicate
terminal state, and sync/local-async/async parity.

Exit criteria: later product commits reuse one fail-closed action driver and no
resource module hand-rolls polling semantics.

Pentest stop: run an incremental pentest for the exact Commit 8 action and
polling surface.

## Commit 9 - Account, Regions, Sizes, And 1-Clicks

Goal: implement the foundational catalog and account reads plus Kubernetes
1-Click installation.

Deliverables: account, region, size, 1-Click list, and Kubernetes install
operations; typed availability, feature, price, slug, quota, and install models;
and exact cost/mutation metadata.

Verification: region/size mismatch, unavailable offerings, malformed prices,
unknown features, pagination, install target binding, cost permit, response
association, and transport parity.

Exit criteria: every included foundational row is executable and the install
operation cannot run without explicit billable mutation authority.

Pentest stop: run an incremental pentest for the exact Commit 9 foundation
surface.

## Commit 10 - SSH Keys, Tags, Projects, And Organizations

Goal: implement shared account organization and resource-labeling workflows.

Deliverables: SSH key, tag, project, default-project, project-resource, and
organization-team operations; bounded names and fingerprints; assignment and
membership models; and destructive/mutation permits.

Verification: SSH fingerprint and key grammar, tag encoding, default-project
identity, duplicate assignments, cross-team confusion, last-owner-like cases,
special SSH list rate limit, and complete wire fixtures.

Exit criteria: all included rows in these areas are executable with exact team,
project, tag, and resource identity binding.

Pentest stop: run an incremental pentest for the exact Commit 10 account
organization surface.

## Commit 11 - Billing And Invoice Artifacts

Goal: implement billing reads with bounded structured and binary responses.

Deliverables: balance, billing history, insights, invoice list/detail/summary,
CSV, and PDF operations; money/currency/time models; binary streaming; and
invoice preview handling.

Verification: decimal precision, currency mismatch, negative/overflow values,
malformed periods, preview identity, content disposition/type, oversized PDF or
CSV, truncation, cancellation, and response association.

Exit criteria: all eight included billing rows are executable without lossy
money conversion or full artifact buffering.

Pentest stop: run an incremental pentest for the exact Commit 11 billing and
artifact surface.

## Commit 12 - Marketplace Add-Ons

Goal: implement add-on catalog and subscription lifecycle with explicit cost.

Deliverables: list/get/app metadata, create, update, plan update, and delete
operations; plan, dimension, feature, resource, price, and billing models; and
cost/destructive permits.

Verification: app/resource identity mismatch, unavailable plan, price and
dimension overflow, conflicting patches, duplicate create, timeout after send,
delete confirmation, and no automatic mutation retries.

Exit criteria: all eight included add-on rows are executable and every
potentially billable transition requires explicit cost authority.

Pentest stop: run an incremental pentest for the exact Commit 12 add-on surface.

## Commit 13 - Droplet Read Models And Inventory

Goal: implement complete Droplet inventory before lifecycle mutations.

Deliverables: list/get, neighbors, backups, snapshots, kernels, firewalls,
associated resources, destroy status, and backup-policy reads; complete Droplet,
network, disk, image, region, size, and feature models.

Verification: IPv4/IPv6 parsing, cross-resource IDs, unknown status/features,
legacy kernels, pagination, null/omitted fields, associated-resource bounds,
backup policy coherence, and large inventory decoding.

Exit criteria: every included Droplet read row is executable and reusable by
later mutation postconditions.

Pentest stop: run an incremental pentest for the exact Commit 13 Droplet read
surface.

## Commit 14 - Droplet Lifecycle And Actions

Goal: implement Droplet creation, actions, and destruction with strong intent.

Deliverables: create, single/tag actions, standard deletion, associated-resource
preview/status, selective deletion, dangerous deletion, and retry-with-associated
resources; user-data protection; SSH/VPC/image bindings; and action polling.

Verification: required constructor fields, user-data redaction, tag fan-out,
wrong action target, conflicting image/size/network inputs, timeout after send,
partial deletion, dangerous permit, cost permit, and retry prohibition.

Exit criteria: all included lifecycle rows are executable and dangerous or
billable behavior is impossible without the exact explicit permit.

Pentest stop: run an incremental pentest for the exact Commit 14 Droplet
lifecycle surface.

## Commit 15 - Droplet Autoscale Pools

Goal: implement autoscale pools without unbounded or contradictory policies.

Deliverables: list/get/create/update/delete, member list, history, and dangerous
delete operations; target, min/max, utilization, cooldown, and pool models.

Verification: min/max contradictions, zero/overflow targets, cooldown bounds,
member duplication, history pagination, cost growth, update races, ordinary
versus dangerous delete, and action/result association.

Exit criteria: all eight included autoscale rows are executable with bounded
scale and explicit cost/destructive authority.

Pentest stop: run an incremental pentest for the exact Commit 15 autoscale
surface.

## Commit 16 - Images, Snapshots, And Transfers

Goal: implement image and snapshot inventory, actions, transfers, and deletion.

Deliverables: image/snapshot list/get/create/update/delete; image actions;
account-transfer create/accept/decline/cancel; typed checksums, regions, status,
and transfer tokens; and polling/permit integration.

Verification: URL import policy, checksum syntax, region mismatch, transfer
token redaction, cross-account association, replay, stale actions, deletion
authority, pagination, and deprecated-field compatibility.

Exit criteria: every included image, snapshot, action, and transfer row is
executable without credential or transfer-token leakage.

Pentest stop: run an incremental pentest for the exact Commit 16 image and
snapshot surface.

## Commit 17 - Block Storage And Volume Actions

Goal: implement volumes, snapshots, and attachment actions coherently.

Deliverables: volume and snapshot CRUD/read operations; name/ID lookup;
attach/detach/resize/action list/get; filesystem and region models; and
Droplet-volume association checks.

Verification: size monotonicity, filesystem grammar, region/Droplet mismatch,
name ambiguity, duplicate attachment, action replay, snapshot lineage,
destructive delete, cost growth, and polling.

Exit criteria: all included block-storage rows are executable with exact
resource association and explicit cost/destructive permits.

Pentest stop: run an incremental pentest for the exact Commit 17 block-storage
surface.

## Commit 18 - Public IP Resources And BYOIP

Goal: implement floating, reserved IPv4/IPv6, action, and BYOIP lifecycles.

Deliverables: Floating IP, Reserved IP, Reserved IPv6, action, and BYOIP
operations; IP/prefix/region/resource models; assign/unassign actions; remote
resource listing; and mutation/destructive permits.

Verification: canonical IP/CIDR parsing, host bits, address-family mismatch,
region mismatch, duplicate assignment, wrong-resource action, prefix ownership,
pagination, delete authority, and action polling.

Exit criteria: all included public-address rows are executable and cannot widen
or reassign a prefix through ambiguous input.

Pentest stop: run an incremental pentest for the exact Commit 18 address
surface.

## Commit 19 - VPCs, Peerings, And NAT Gateways

Goal: implement private networking with canonical ranges and route identity.

Deliverables: VPC, member, peering, VPC-nested peering, and NAT gateway
operations; region/range/route models; patch/update distinctions; and
cost/destructive permits.

Verification: canonical network ranges, overlap, cross-region peering,
self-peering, duplicate route, nested/top-level identity equivalence, NAT size
and cost, deletion dependencies, and pagination.

Exit criteria: all included VPC, peering, and NAT rows are executable with
fail-closed topology validation.

Pentest stop: run an incremental pentest for the exact Commit 19 private
networking surface.

## Commit 20 - Firewalls And Load Balancers

Goal: implement ingress/egress and load-distribution controls safely.

Deliverables: firewall CRUD/rules/tag/Droplet assignment and load-balancer
CRUD/Droplet/forwarding-rule/cache operations; canonical protocol, port, CIDR,
certificate, health, algorithm, and redirect models.

Verification: empty/wide firewall rules, port ranges, conflicting sources,
duplicate assignments, forwarding loops, certificate/protocol mismatch, cache
deletion, health thresholds, cost changes, and destructive permits.

Exit criteria: all included firewall and load-balancer rows are executable and
cannot silently broaden access or traffic exposure.

Pentest stop: run an incremental pentest for the exact Commit 20 traffic-control
surface.

## Commit 21 - Domains, Certificates, And CDN

Goal: implement DNS control, certificate lifecycle, and CDN endpoints.

Deliverables: domain/record CRUD, certificate CRUD, and CDN endpoint/cache
operations; DNS names, record values, TTL, certificate/key, origin, and cache
models; sensitive PEM handling; and special CDN rate policy.

Verification: DNS canonicalization, record-type semantics, CAA/MX/SRV bounds,
private-key redaction, certificate chain limits, hostile origin URLs, cache file
limits, five-per-ten-second scheduling, and destructive authority.

Exit criteria: all included domain, record, certificate, and CDN rows are
executable without key leakage or accidental DNS/cache widening.

Pentest stop: run an incremental pentest for the exact Commit 21 DNS,
certificate, and CDN surface.

## Commit 22 - Partner Network Connect And NFS

Goal: implement partner attachments, credentials, NFS shares, access points,
snapshots, and actions.

Deliverables: all Partner Network Connect and NFS operations; BGP, route,
service-key, share, export, access-point, snapshot, and action models; protected
credential outputs; and cost/destructive permits.

Verification: ASN/key redaction, route canonicalization, cross-region/VPC
binding, NFS path and client range validation, snapshot lineage, action polling,
partial deletion, and response association.

Exit criteria: every included partner-network and NFS row is executable with
credential-safe outputs and exact topology binding.

Pentest stop: run an incremental pentest for the exact Commit 22 partner-network
and NFS surface.

## Commit 23 - Security Scans And Uptime

Goal: implement provider security findings and availability monitoring.

Deliverables: security scan/rule/suppression/settings operations and uptime
check/alert/state operations; finding, affected-resource, schedule, endpoint,
region, threshold, and notification models.

Verification: suppression scope, stale scan association, finding bounds,
hostile check URLs, SSRF-sensitive documentation boundaries, interval/timeout
coherence, alert threshold contradictions, delete authority, and pagination.

Exit criteria: all included Security and Uptime rows are executable without
silently suppressing findings or admitting malformed monitored targets.

Pentest stop: run an incremental pentest for the exact Commit 23 security and
uptime surface.

## Commit 24 - Kubernetes Read And Credential Boundaries

Goal: implement Kubernetes inventory and protect cluster access material.

Deliverables: cluster/node-pool lists and gets, options, upgrades, lint results,
status messages, associated resources, cluster user, kubeconfig, and credential
operations; protected kubeconfig/certificate/token models.

Verification: cluster/node association, region/version coherence, malformed
kubeconfig and PEM, credential expiry, access-scope enforcement, redaction,
pagination, and omission of deprecated recycle behavior.

Exit criteria: all included Kubernetes reads are executable and credential
outputs cannot enter ordinary logs or unrelated requests.

Pentest stop: run an incremental pentest for the exact Commit 24 Kubernetes read
and credential surface.

## Commit 25 - Kubernetes Lifecycle And Integrations

Goal: implement cluster/node-pool lifecycle, upgrades, registry integration,
lint runs, and selective/dangerous destruction.

Deliverables: create/update/delete/upgrade cluster; add/update/delete nodes and
node pools; run lint; registry add/remove; associated-resource deletion; action
drivers; and cost/destructive permits.

Verification: version transitions, surge and autoscale contradictions,
node-count cost bounds, registry identity, deletion previews, selective versus
dangerous destruction, timeout after send, and action polling.

Exit criteria: all included Kubernetes mutations are executable and deprecated
recycle is represented only as an excluded matrix row.

Pentest stop: run an incremental pentest for the exact Commit 25 Kubernetes
lifecycle surface.

## Commit 26 - Container Registries

Goal: implement the current plural registry API without preserving superseded
singular endpoints.

Deliverables: all current registry list/get/create/delete, subscription,
credential, validation, repository/tag/manifest, garbage collection, and option
operations; protected Docker credential outputs; and exact supersession mapping.

Verification: registry/repository path encoding, digest grammar, credential
expiry/redaction, subscription cost, garbage-collection state, deletion scope,
pagination, and tests proving all 18 singular operations remain inaccessible.

Exit criteria: all 19 current registry rows are executable and no deprecated
route can be prepared through the public API.

Pentest stop: run an incremental pentest for the exact Commit 26 registry
surface.

## Commit 27 - Spaces Key Control Plane

Goal: implement Spaces access-key management without claiming the S3 data
plane.

Deliverables: list/get/create/update/patch/delete Spaces key operations;
protected access-key and secret outputs; bucket/scope models; rotation workflow;
and destructive permits.

Verification: one-time secret capture, source/destination cleanup, scope
confusion, key ID grammar, rotation ordering, partial failure, self-lockout,
redaction, and proof that no S3 object operation is exposed.

Exit criteria: all six included Spaces key rows are executable and generated
secrets remain protected throughout their SDK lifetime.

Pentest stop: run an incremental pentest for the exact Commit 27 Spaces key
surface.

## Commit 28 - Functions

Goal: implement namespace, trigger, and access-key lifecycle.

Deliverables: namespace and trigger CRUD/read operations; access-key
list/create/update/delete; schedule, function, namespace, route, and protected
key models; and admin-scope/mutation permits.

Verification: namespace/trigger association, schedule grammar, hostile routes,
key one-time output, admin versus granular scope, deletion authority, retries,
pagination, and redaction.

Exit criteria: all 13 included Functions rows are executable with exact
namespace binding and protected credentials.

Pentest stop: run an incremental pentest for the exact Commit 28 Functions
surface.

## Commit 29 - Apps Reads, Logs, Metrics, And Console Access

Goal: implement App Platform inspection and sensitive runtime access.

Deliverables: app/deployment/event/job/instance/alert/health/region/size reads;
logs and aggregate logs; bandwidth metrics; exec and active-deployment exec;
bounded streaming and protected console/session outputs.

Verification: app/deployment/component association, log injection and bounds,
cursor pagination, event ordering, metric ranges, console credential redaction,
expiry, cancellation, and access-console scope.

Exit criteria: all included App read/log/metric/exec rows are executable and
runtime access material cannot leak or cross app boundaries.

Pentest stop: run an incremental pentest for the exact Commit 29 App inspection
surface.

## Commit 30 - Apps Lifecycle, Deployments, Jobs, And Rollbacks

Goal: implement App Platform mutations and rollback state machines.

Deliverables: app create/update/delete/restart; deployment creation/cancel;
event and job cancellation; job invocation; alert destinations; database
trusted source; app-spec validation; rollback create/validate/commit/revert; and
cost/destructive permits.

Verification: app-spec bounds and secrets, deployment races, job replay,
rollback ancestry, validate-before-commit, cancel terminal states, database
association, timeout after send, cost changes, and action polling.

Exit criteria: all included App mutations are executable through explicit
stateful workflows and no deployment or rollback runs implicitly.

Pentest stop: run an incremental pentest for the exact Commit 30 App lifecycle
surface.

## Commit 31 - Database Catalog, Clusters, And Sensitive Reads

Goal: implement database inventory and credential-bearing reads first.

Deliverables: options, cluster/replica/user/pool/firewall/backup/event/log-sink,
Kafka, OpenSearch, config, autoscale, migration, CA, and metric-credential reads;
engine/version/region/size models; and protected credentials.

Verification: engine-specific unions, preview edition status, cluster/resource
association, credential/CA redaction, pagination, malformed config, unknown
versions, backup lineage, and bounded logs/events.

Exit criteria: every included database read row is executable and all
credential outputs are context-bound and protected.

Pentest stop: run an incremental pentest for the exact Commit 31 database read
surface.

## Commit 32 - Database Cluster, User, Pool, And Replica Lifecycle

Goal: implement core database mutations with explicit cost and credential
handling.

Deliverables: cluster/replica/user/pool create/update/delete; resize, region,
maintenance, major-version, auth reset, promotion, firewall, SQL mode, eviction,
and autoscale operations; state drivers; and cost/destructive permits.

Verification: engine/operation compatibility, resize cost, irreversible upgrade,
region migration, maintenance windows, password rotation cleanup, replica
promotion, firewall widening, timeout after send, and polling.

Exit criteria: all included core database mutations are executable and every
costly, destructive, or credential-changing transition is explicit.

Pentest stop: run an incremental pentest for the exact Commit 32 core database
lifecycle surface.

## Commit 33 - Database Kafka, Migration, Config, And Logs

Goal: finish specialized database APIs without generic untyped maps leaking into
the public surface.

Deliverables: Kafka schema/topic CRUD/config, online migration, engine config,
DigitalOcean settings, log sink, update installation, and OpenSearch index
operations; typed engine-specific request families and secret-safe fields.

Verification: schema/topic names, version identity, compatibility modes,
migration credentials, config key/type bounds, unknown settings, log sink
targets, update state, index deletion, and operation coverage.

Exit criteria: all remaining included database rows are executable with typed
engine contracts and no unbounded configuration object.

Pentest stop: run an incremental pentest for the exact Commit 33 specialized
database surface.

## Commit 34 - Vector Databases

Goal: implement vector database lifecycle, backups, restore, resize, and
credentials.

Deliverables: list/get/create/update/delete, credentials, backups, restore
status, restore, resize, and tag operations; engine, node, backup, status, and
protected credential models.

Verification: dimension/node/size bounds, cost changes, backup lineage,
cross-database restore, credential redaction, tag replacement, destructive
delete, polling, and timeout-after-send behavior.

Exit criteria: all 11 included vector database rows are executable with exact
backup association and explicit cost/destructive authority.

Pentest stop: run an incremental pentest for the exact Commit 34 vector
database surface.

## Commit 35 - Droplet, App, And Autoscale Metrics

Goal: implement the first bounded Monitoring metric families.

Deliverables: Droplet CPU/bandwidth/filesystem/load/memory, App CPU/memory/restart,
and autoscale current/target metric operations; timestamp ranges, metric series,
labels, samples, and precision-safe values.

Verification: range ordering, maximum windows, NaN/infinity rejection, label
bounds, duplicate/out-of-order samples, resource association, empty series,
large responses, and operation coverage.

Exit criteria: every included metric row in these families is executable through
shared bounded metric models.

Pentest stop: run an incremental pentest for the exact Commit 35 compute/app
metric surface.

## Commit 36 - Database And Load Balancer Metrics

Goal: implement the remaining high-volume metric families without copy-pasted
decoders.

Deliverables: MySQL database and load-balancer Droplet/frontend metrics;
metric-name associations; percentile, response, throughput, TLS, firewall,
health, and connection series; and shared query preparation.

Verification: metric/label mismatch, percentile semantics, counter versus gauge,
network units, sample ordering, oversized series, unknown metrics, exact
operation mapping, and transport parity.

Exit criteria: every included database and load-balancer metric row is
executable and generated associations prevent endpoint/model swaps.

Pentest stop: run an incremental pentest for the exact Commit 36 database and
load-balancer metric surface.

## Commit 37 - Monitoring Alerts, Destinations, And Sinks

Goal: complete Monitoring control-plane mutations and reads.

Deliverables: alert policy, destination, and sink list/get/create/update/delete
operations; expression, threshold, comparison, window, channel, endpoint, and
credential models; and mutation/destructive permits.

Verification: expression bounds, threshold/time coherence, hostile webhook
targets, secret redaction, duplicate channels, sink resource scope, no-op
patches, delete authority, rate limits, and response association.

Exit criteria: all remaining included Monitoring rows are executable without
silently widening alert delivery or exposing destination credentials.

Pentest stop: run an incremental pentest for the exact Commit 37 Monitoring
control surface.

## Commit 38 - Dedicated Inference

Goal: implement dedicated inference resources and token lifecycle.

Deliverables: list/get/create/update/delete, sizes, accelerators, GPU config, CA,
and token list/create/delete operations; accelerator/region/model/status models;
protected inference tokens; and cost/destructive permits.

Verification: model/accelerator/size compatibility, capacity and cost bounds,
CA/token redaction, token one-time output, expiry, deletion authority, polling,
and dedicated-inference scope mapping.

Exit criteria: all 13 included dedicated inference rows are executable and
credential outputs remain protected.

Pentest stop: run an incremental pentest for the exact Commit 38 dedicated
inference surface.

## Commit 39 - GradientAI Workspaces, Catalogs, And Provider Keys

Goal: establish GradientAI shared models, workspaces, catalogs, regions, and
external provider credentials.

Deliverables: workspace CRUD/list/get, region/model/catalog/card lists, OpenAI
and Anthropic key list/get/create/update/delete, current model-key operations,
and protected provider-key models.

Verification: workspace/model association, lifecycle status including preview,
provider key redaction and rotation, wrong-provider substitution, pagination,
region mismatch, and proof the retired model-key operation is inaccessible.

Exit criteria: all assigned GradientAI foundation rows are executable and
external provider secrets cannot escape their context.

Pentest stop: run an incremental pentest for the exact Commit 39 GradientAI
foundation surface.

## Commit 40 - GradientAI Agents, Versions, Functions, And Guardrails

Goal: implement agent lifecycle and composition safely.

Deliverables: agent list/get/create/update/delete, children, usage, versions,
rollback, API keys, deployment visibility, agent attach/detach, functions,
guardrails, workspace moves, and provider-key filtered lists.

Verification: agent graph cycles, parent/workspace mismatch, rollback ancestry,
function schema bounds, guardrail attachment, visibility transitions, API-key
rotation/redaction, delete authority, and pagination.

Exit criteria: all assigned agent rows are executable with acyclic identity and
protected key handling.

Pentest stop: run an incremental pentest for the exact Commit 40 GradientAI
agent surface.

## Commit 41 - GradientAI Knowledge Bases And Indexing

Goal: implement knowledge bases, data sources, uploads, and indexing jobs.

Deliverables: knowledge base/data source CRUD and attachment; indexing job
create/list/get/cancel; scheduled indexing; source-specific models; presigned
upload/download URLs; and bounded job polling.

Verification: source/knowledge-base association, path and bucket bounds,
presigned authority/expiry/replay, no bearer forwarding, indexing state
regression, cancellation, schedule coherence, and signed-result URL handling.

Exit criteria: all assigned knowledge/indexing rows are executable and every
returned URL is used through a credential-isolated checked path.

Pentest stop: run an incremental pentest for the exact Commit 41 GradientAI
knowledge and indexing surface.

## Commit 42 - GradientAI Evaluation

Goal: implement evaluation datasets, test cases, metrics, presets, and runs.

Deliverables: dataset/test-case/custom-metric/evaluation-run CRUD and execution;
presigned dataset uploads/downloads; presets and result retrieval; prompt result
models; cancellation; and bounded asynchronous drivers.

Verification: dataset/test-case/run association, metric type/range, prompt and
result bounds, presigned URL isolation, cancel/terminal races, result download
size, pagination, and destructive permits.

Exit criteria: all assigned evaluation rows are executable with exact lineage
and bounded sensitive content.

Pentest stop: run an incremental pentest for the exact Commit 42 GradientAI
evaluation surface.

## Commit 43 - GradientAI Custom Models, Routers, And Integrations

Goal: complete GradientAI custom models, routers, scheduled behavior, and OAuth
integration endpoints.

Deliverables: custom model import/get/list/update/delete; router/preset/task
preset list/get/create/update/delete; Dropbox OAuth URL/token operations; and
remaining assigned GradientAI rows with exact lifecycle models.

Verification: model source and checksum, router cycles/weights, preset identity,
OAuth state/redirect binding, integration token redaction, schedule conflicts,
delete authority, and generated coverage for all included GradientAI rows.

Exit criteria: every included GradientAI operation is executable and no retired
or unassigned row remains.

Pentest stop: run an incremental pentest for the exact Commit 43 remaining
GradientAI surface.

## Commit 44 - Serverless, Embedding, And Agent Inference

Goal: implement direct inference on its separate authorities with bounded
streaming and credential classes.

Deliverables: serverless model list, chat, messages, responses, images, async
invoke, embeddings, and customer Agent Inference; inference/agent credentials;
bounded SSE or response streaming where source-locked; usage and finish models.

Verification: authority/credential mismatch, hostile agent host, prompt and
media bounds, streaming fragmentation, malformed events, cancellation, token
usage overflow, model mismatch, response limits, and no automatic mutation
retry.

Exit criteria: all included serverless, embedding, and Agent Inference rows are
executable without cross-authority credential leakage.

Pentest stop: run an incremental pentest for the exact Commit 44 direct
inference surface.

## Commit 45 - Batch Inference And Presigned Uploads

Goal: implement batch files, jobs, results, cancellation, and one-time uploads.

Deliverables: create/upload batch file, create/list/get/cancel batch, result
retrieval, short-lived presigned upload request, streaming input/results, and
bounded batch state polling.

Verification: exact returned URL validation, no DigitalOcean bearer on upload,
expiry/replay, method/content-type binding, file and result size limits,
partial upload, cancellation, job/file association, and terminal-state
coherence.

Exit criteria: all seven included batch rows are executable and presigned
requests cannot become general-purpose authenticated HTTP calls.

Pentest stop: run an incremental pentest for the exact Commit 45 batch
inference surface.

## Commit 46 - OAuth Authorization Code, Refresh, And Revocation

Goal: support secure delegated authorization without exposing the implicit
grant.

Deliverables: authorization-code URL builder with required state; callback
validation; token exchange; single-use refresh rotation; revocation;
client-secret protection; exact redirect URI binding; and optional PKCE only if
source-locked as supported.

Verification: CSRF state mismatch, redirect confusion, code replay, client
secret leakage, refresh races, old-token retirement, revocation authority,
error responses, URL redaction, and compile-fail rejection of implicit token
callbacks.

Exit criteria: all admitted OAuth workflows are executable, refresh rotation is
atomic, and no public API supports `response_type=token`.

Pentest stop: run an incremental pentest for the exact Commit 46 OAuth surface.

## Commit 47 - Unified Client And Workflow Coverage

Goal: make the checked path the easiest path for every included operation.

Deliverables: official provider client constructors; operation-to-prepared
request bindings for every matrix row; automatic method, authority, scope,
headers, body, response bound, and decoder selection; sync, local-async, async,
raw, and streaming parity; and high-level create/poll and rotate workflows.

Verification: generated coverage assertions; compile-checked examples;
credential, cost, and permit routing; concurrency and cancellation; checked
response decoding; returned URL handling; and proof no supported operation
requires manual HTTP assembly.

Exit criteria: every included matrix row is executable through the official
client and all operation associations have independent test evidence.

Pentest stop: run an incremental pentest for the exact Commit 47 unified client
and workflow surface.

## Commit 48 - Live Evidence, Fuzzing, And Platform Qualification

Goal: produce current adversarial and platform evidence without granting CI
mutation or cost authority.

Deliverables: least-scope read-only live harness; mock-only mutation staging;
fuzz targets for OpenAPI drift, paths, IPs, pagination, JSON, metrics, actions,
OAuth, streams, returned URLs, and secrets; target checks; SBOMs; and package
verification.

Verification: full fuzz build and bounded campaigns, regression corpora,
no-secret CI proof, endpoint-specific quota tests, MSRV/stable/platform matrices,
dependency/advisory review, file-length policy, package contents, fresh SBOMs,
and reproducible archives.

Exit criteria: every support, security, policy, platform, dependency, and live
claim has executable evidence, and CI cannot create, mutate, delete, publish,
scale, or incur provider cost.

Pentest stop: run an incremental pentest for the exact Commit 48 qualification
surface.

## Commit 49 - Scope Freeze And Release Candidate

Goal: freeze and qualify the complete selected DigitalOcean provider without
adding features.

Deliverables: final operation matrix and exact included count; zero unclassified
or model-only rows; provider README/examples; threat model; auth, OAuth, scope,
rate, cost, mutation, streaming, live-test, drift, deprecation, migration, and
platform documentation; release notes; provenance; and one candidate gate.

Verification: rerun all 49 commit gates, live source drift, full workspace and
provider tests, every execution mode, fuzz/adversarial suites, MSRV/platform
matrices, dependency/SBOM checks, public API and SemVer review, reproduction
from two clean clones, and green GitHub CI and CodeQL.

Exit criteria: all Commit 1 included rows are executable and documented; every
excluded row has a precise reason; no API, dependency, feature, or scope change
occurs after qualification; and the candidate can receive a version only after
a separate release decision.

Pentest stop: run a full-provider pentest for the exact Commit 49 candidate,
remediate and retest every finding, rerun the complete release gate, then wait
for green GitHub CI and CodeQL before selecting a version, signing a tag, or
publishing crates.

## Deferred Surfaces

Commit 1 records exact exclusions, but the following are presumed deferred:

- all 20 explicitly deprecated OpenAPI operations;
- private, undocumented, console-only, or limited-access operations not admitted
  by the public matrix;
- the Spaces S3-compatible bucket/object data plane;
- Droplet metadata-service and other workload-local protocols;
- container registry OCI/Docker image push and pull data planes;
- implicit OAuth grant and token-bearing URL-fragment callbacks;
- creating OAuth applications through control-panel automation;
- arbitrary execution of returned URLs or automatic credential forwarding;
- automatically retried non-idempotent or billable operations; and
- any operation added upstream after the Commit 1 source lock.

Deferral is not permanent rejection. A later DigitalOcean release can add a
source-locked surface through a separate commit plan after its stability,
protocol, security, and maintenance costs are reviewed.

## Release Decision

This document deliberately does not name a release version. After Commit 49
passes its full-provider pentest, complete release gate, GitHub CI, and CodeQL,
maintainers decide whether the accumulated compatible workspace changes warrant
a minor workspace release or another SemVer version. The
`cloud-sdk-digitalocean` crate receives its own independently appropriate
package version under the post-1.0 workspace versioning policy.
