# Scaleway Commit Plan

Status: provider candidate assessment; this plan does not select Scaleway as
the next provider and does not assign a release version.

## Decision Summary

The estimated implementation train is **30 planned commits** followed by one
provider release. These are logical reviewed implementation commits, not a
promise that remediation, documentation corrections, or merge mechanics will
produce exactly 30 Git objects. A security fix may add commits without changing
the numbered scope.

The estimate targets the complete finite set of current General Availability
(GA) Scaleway HTTP APIs admitted by Commit 1. It does not mean every product
shown in Scaleway's catalog. Alpha and beta interfaces, deprecated versions,
S3-compatible Object Storage data operations, OpenAI-compatible streaming data
operations, and separately authenticated adjacent services remain excluded
unless Commit 1 proves that they belong in the first stable provider scope.

Every numbered commit is an implementation stop. It must be locally green and
receive an incremental pentest against the preceding accepted commit before
the next numbered commit begins. The final commit additionally receives a
full-provider pentest and final retest before any version is selected, tagged,
or published.

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

- the official [Scaleway API index](https://www.scaleway.com/en/developers/api/),
  which reported documentation build `v1.8643.0` during review;
- the downloadable OpenAPI schemas linked by the official product pages; and
- official [`scaleway-sdk-go`](https://github.com/scaleway/scaleway-sdk-go)
  commit `783550b709424d737382e4190752f7618f0a35a7` from 2026-08-19 as
  secondary implementation evidence.

The API index exposed 58 product documentation entries. Twenty-four entries
defaulted to a GA path version such as `v1`, `v2`, or `v3`; the remainder
defaulted to alpha or beta tracks. A preliminary fetch of 25 GA or known stable
schema pages found 583 raw HTTP operation entries. This is sizing evidence,
not the final support count: pages can expose overlapping operations, some
stable alternatives are not the default documentation selection, and source
classification has not yet removed deprecated or adjacent-protocol rows.

Scaleway's official overview establishes several provider-wide contracts that
must be source-locked:

- authenticated control-plane requests normally use `X-Auth-Token`;
- authorities can be global, regional, or zonal under `api.scaleway.com`;
- GA, beta, and alpha status is encoded in path versions;
- pagination varies between page numbers, page sizes, page tokens,
  `X-Total-Count`, body `total_count`, and `next_page_token`;
- APIs use `GET`, `POST`, `PUT`, `PATCH`, and `DELETE`, including empty success
  responses; and
- Scaleway Resource Names encode product, partition, locality, resource type,
  and resource identity.

The old six-release sketch is therefore too coarse for this repository's
request-fidelity, bounded-decoding, execution-parity, mutation-authority, and
source-drift requirements. Thirty commits are a reasonable planning estimate;
Commit 1 is allowed to increase or reduce the number only through a reviewed
plan amendment backed by the exact operation inventory.

## Scope Rules

1. One provider maps to one crate: `cloud-sdk-scaleway`.
2. Provider-neutral transport, sanitization, and testkit behavior remains in
   the existing neutral crates.
3. Default provider features remain empty and `no_std` compatible.
4. No generated source file or hand-written code file may exceed 500 lines.
5. Every admitted operation receives request, response, error, authority,
   retry, idempotency, cost, and execution classifications.
6. No mutation, destructive action, or billable order executes implicitly.
7. Official endpoints are the safe default. Custom endpoints are explicit and
   must not receive credentials derived from untrusted configuration.
8. Pre-GA APIs are excluded by default even when the product itself is GA.
9. Provider coverage is claimed only for rows in the committed API matrix.
10. A numbered commit cannot widen scope assigned to a later commit.

## Commit 1 - Source Lock And Finite Scope

Goal: establish the exact Scaleway support claim before provider code exists.

Deliverables: bounded retrieval of the official API index and every candidate
schema; exact URL, redirect, size, digest, OpenAPI version, documentation build,
product, path-version, locality, operation, deprecation, and protocol records;
an API matrix assigning every discovered row to included, deferred, excluded,
or superseded; and explicit treatment of stable alternatives when a product
page defaults to alpha or beta.

Verification: independently reproduce all source digests; reject cross-origin
redirects, malformed YAML, duplicate operation identities, unknown versions,
and unclassified rows; compare the official Go SDK only as secondary evidence;
and prove that one command rebuilds the observation without rewriting locks.

Exit criteria: the exact GA operation count and exclusions are reviewable, no
row is unclassified, and any change to the estimated 30-commit train is made in
this document before implementation proceeds.

Pentest stop: run an incremental pentest for the exact Commit 1 implementation
and source-lock boundary.

## Commit 2 - Provider Drift And Changelog Detection

Goal: make upstream additions, removals, deprecations, and contract changes a
fail-closed maintenance event from the first provider commit.

Deliverables: a Scaleway adapter for the neutral provider-drift engine;
operation, parameter, request-body, response, schema, endpoint, auth,
pagination, and stability fingerprints; official documentation-build and
changelog observation; and an explicit reviewed lock-refresh workflow.

Verification: fixtures for added, removed, moved, deprecated, and changed
operations; parameter requiredness and enum changes; schema-only changes;
redirect, timeout, size, origin, and parser failures; and local-only plus live
fetch modes that never accept drift automatically.

Exit criteria: CI can detect every category used by the API matrix and cannot
turn a failed fetch or incomplete inventory into a green result.

Pentest stop: run an incremental pentest for the exact Commit 2 drift tooling.

## Commit 3 - Provider Crate And Module Boundaries

Goal: add the publishable provider crate without weakening the stable neutral
foundation.

Deliverables: `cloud-sdk-scaleway` with empty defaults, `no_std` foundations,
optional admitted Serde support, provider and service identities, bounded
module layout, package metadata, README, docs.rs configuration, SBOM inclusion,
release-governance classification, and one-primary-crate-per-provider checks.

Verification: default, no-default-feature, all-feature, MSRV, stable, portable
target, package-content, dependency-boundary, documentation, and file-length
checks; confirm no provider-specific transport, testkit, or sanitization crate.

Exit criteria: the empty provider compiles everywhere claimed by `cloud-sdk`,
publishes no unsupported API, and changes no frozen neutral contract.

Pentest stop: run an incremental pentest for the exact Commit 3 crate boundary.

## Commit 4 - Authorities, Locality, And Resource Identity

Goal: encode global, regional, and zonal routing without credential confusion.

Deliverables: typed regions and zones, validated project and organization
identities, UUID and provider-defined identifiers, bounded Scaleway Resource
Names, canonical `api.scaleway.com` endpoint derivation, service/version path
prefixes, and visibly explicit custom endpoint construction.

Verification: path injection, authority confusion, Unicode, percent encoding,
maximum target length, unknown locality, region-zone mismatch, SRN grammar,
custom endpoint, and credential-destination tests.

Exit criteria: no service can construct an unclassified authority or attach a
token before official/custom endpoint policy is resolved.

Pentest stop: run an incremental pentest for the exact Commit 4 endpoint and
identity boundary.

## Commit 5 - Authentication And Secret Lifecycle

Goal: implement Scaleway credentials with explicit ownership and rotation.

Deliverables: protected `X-Auth-Token` ingestion from owned and caller-guarded
bytes, generation-scoped attempts, redacted diagnostics, rotation and lockout
policy, least-capability guidance, and separate authentication policies for any
admitted endpoint that does not use the normal control-plane header.

Verification: header injection, invalid token bytes, source-buffer cleanup,
clone/drop/rotation behavior, concurrent attempt generations, custom endpoint
denial, authentication rejection classification, and logging redaction tests.

Exit criteria: secrets have one documented owner at every stage and no safe API
requires an ordinary unprotected heap copy.

Pentest stop: run an incremental pentest for the exact Commit 5 credential
boundary.

## Commit 6 - Shared Wire, Error, And Pagination Contracts

Goal: implement provider-wide protocol behavior before product models duplicate
it.

Deliverables: bounded JSON envelopes; empty-response handling; static,
payload-free errors; request IDs where source-defined; page/per-page,
page/page-size, token, `X-Total-Count`, body-total, and next-token pagination;
RFC 3339 timestamps, durations, exact money, and unknown-enum preservation.

Verification: duplicate keys, depth and byte limits, malformed UTF-8, unknown
fields and enums, integer and decimal limits, conflicting pagination evidence,
header/body count disagreement, empty/non-empty status mismatches, and
allocation-failure paths.

Exit criteria: every Commit 1 operation maps to one reviewed common protocol
variant or an explicit product-specific exception.

Pentest stop: run an incremental pentest for the exact Commit 6 wire boundary.

## Commit 7 - Operation Metadata And Async Resource Policy

Goal: classify execution risk before exposing product actions.

Deliverables: read-only, mutation, destructive, and cost-bearing metadata;
retry and idempotency classes; delivery phases; reconciliation requirements;
provider task and waiter contracts; polling bounds; and direct/shared permit
associations for every admitted operation.

Verification: complete matrix coverage, permit mismatch, stale intent, replay,
not-sent/possibly-sent/response-started faults, contradictory task state,
polling exhaustion, cancellation, and unknown terminal-state tests.

Exit criteria: no admitted mutation lacks an explicit authority, retry, cost,
delivery, and reconciliation decision.

Pentest stop: run an incremental pentest for the exact Commit 7 operation
policy.

## Commit 8 - Account, Contracts, And Annotations

Goal: implement the admitted account-contract and annotation operations.

Deliverables: complete typed requests and responses, organization/project
scope, SRN associations, pagination, checked decoding, operation metadata, and
blocking, Send-async, and local-async client methods for every assigned row.

Verification: source-derived wire fixtures, ownership boundaries, cross-scope
denial, malformed SRNs, pagination, executor parity, and read-only live smoke.

Exit criteria: all Commit 8 matrix rows are executable and no account or
annotation row is represented only by a model.

Pentest stop: run an incremental pentest for the exact Commit 8 implementation.

## Commit 9 - Billing, Marketplace, And Partner Catalogs

Goal: implement admitted commercial reads and controlled commercial mutations.

Deliverables: exact money and tax models, invoice/file boundaries, budgets and
alerts where source-admitted, marketplace/catalog reads, partner operations,
cost metadata, explicit billable authority, and bounded file handling.

Verification: currency and nanos boundaries, negative and oversized amounts,
file-name/content-type injection, pagination, cost-permit mismatch, uncertain
delivery, non-executing mutation staging, and read-only live smoke.

Exit criteria: every commercial operation is classified and no billable route
can execute through a read-only or generic mutation path.

Pentest stop: run an incremental pentest for the exact Commit 9 commercial
boundary.

## Commit 10 - Instance Reads And Catalog Models

Goal: complete admitted Instance inventory, image, type, volume, snapshot,
security-group, placement-group, IP, and dashboard reads.

Deliverables: complete models and filters, zonal paths, list/get associations,
token and legacy pagination where source-defined, checked decoding, quota
metadata, and all execution modes.

Verification: one fixture per response shape, unknown enum and nullable-field
tests, address and size bounds, pagination progression, oversized catalogs,
cross-executor parity, and least-capability live smoke.

Exit criteria: every assigned read row is typed, bounded, client-reachable, and
counted by executable coverage checks.

Pentest stop: run an incremental pentest for the exact Commit 10 Instance read
surface.

## Commit 11 - Instance Mutations, Actions, And User Data

Goal: complete admitted Instance creation, updates, lifecycle actions,
attachments, templates, security rules, and user-data operations.

Deliverables: required-field constructors, PATCH intent, bounded JSON and raw
user data, secret-safe cloud-init handling, mutation/destructive/cost permits,
action polling, delivery phases, idempotency decisions, and reconciliation.

Verification: absent-versus-clear PATCH tests, body escaping, user-data
cleanup, attachment conflicts, security-rule widening, action-state conflicts,
permit mismatch, fault injection, and non-executing live staging.

Exit criteria: every assigned Instance mutation is executable only through its
typed policy and no secret payload appears in diagnostics or ordinary clones.

Pentest stop: run an incremental pentest for the exact Commit 11 Instance
mutation surface.

## Commit 12 - Block Storage

Goal: implement the complete admitted Block Storage scope.

Deliverables: volume types, volumes, snapshots, import/export associations,
zonal constraints, exact sizes, attachment references, status models,
pagination, permits, asynchronous workflows, and all execution modes.

Verification: size overflow, zone mismatch, attached-delete denial,
import/export authority and bucket-name boundaries, task polling, uncertain
delivery, malformed large responses, and read-only live smoke.

Exit criteria: every assigned Block Storage row has exact request fidelity and
safe lifecycle behavior.

Pentest stop: run an incremental pentest for the exact Commit 12 storage
surface.

## Commit 13 - VPC And IPAM

Goal: implement admitted VPC, Private Network, subnet, route, ACL, and IPAM
operations.

Deliverables: strict IP/CIDR models, regional paths, SRNs, route and subnet
identity, allocation models, pagination, controlled mutation, conflict-safe
PATCH behavior, and all execution modes.

Verification: differential IP parsing, host-bit and canonical-identity policy,
overlapping ranges, route widening, locality mismatch, duplicate resources,
pagination, permit faults, and read-only live smoke.

Exit criteria: every network range has documented match-versus-identity
semantics and every assigned row is executable.

Pentest stop: run an incremental pentest for the exact Commit 13 VPC/IPAM
surface.

## Commit 14 - Public Gateways

Goal: implement the current admitted Public Gateway API without retaining a
silent legacy-version fallback.

Deliverables: gateway, network, DHCP, IP, PAT, route, and action models;
regional/zonal associations; async resources; permits; reconciliation; and
explicit exclusion of superseded API versions.

Verification: version confinement, address and port boundaries, PAT conflicts,
route authority, task state, uncertain delivery, cross-executor parity, and
read-only live smoke.

Exit criteria: the selected gateway version is complete and no request can
fall back to an older authority or path.

Pentest stop: run an incremental pentest for the exact Commit 14 gateway
surface.

## Commit 15 - Load Balancers

Goal: implement the complete admitted zonal Load Balancer scope.

Deliverables: load balancers, frontends, backends, certificates, ACLs, routes,
IPs, subscriptions, health checks, statistics, actions, pagination, secrets,
cost permits, and all execution modes.

Verification: protocol/port combinations, certificate and key cleanup, ACL
widening, header injection, health-check bounds, duplicate routes, asynchronous
actions, uncertain delivery, large responses, and read-only live smoke.

Exit criteria: every selected Load Balancer row is executable with no secret or
billable action crossing an untyped boundary.

Pentest stop: run an incremental pentest for the exact Commit 15 load-balancer
surface.

## Commit 16 - Elastic Metal Core

Goal: implement admitted Elastic Metal servers, offers, options, operating
systems, BMC access, installation, metrics, and lifecycle actions.

Deliverables: zonal models, protected installation credentials, partitioning
schemas, cost and destructive permits, metrics bounds, BMC expiry handling,
task/action reconciliation, and all execution modes.

Verification: password and service-secret cleanup, partition validation,
billable order authority, BMC redaction/expiry, metrics query limits,
possibly-sent faults, action state, and read-only live smoke.

Exit criteria: every core Elastic Metal row is complete and credential-bearing
installation cannot be logged, replayed, or implicitly retried.

Pentest stop: run an incremental pentest for the exact Commit 16 Elastic Metal
core surface.

## Commit 17 - Elastic Metal Networking And Dedibox Reads

Goal: complete admitted Elastic Metal private networking and the read-only
Dedibox inventory/catalog surface.

Deliverables: server-private-network associations, Dedibox server, offer,
option, IP, network, hardware, rescue, and installation-state reads; separate
service authorities where required; pagination; and all execution modes.

Verification: authority and identity separation, private-network locality,
cross-account denial, nullable legacy fields, bounded hardware/catalog lists,
pagination, executor parity, and least-capability live smoke.

Exit criteria: adjacent dedicated-server identities cannot be confused with
Elastic Metal resources and every assigned read is executable.

Pentest stop: run an incremental pentest for the exact Commit 17 dedicated
server read boundary.

## Commit 18 - Dedibox Mutations And Sensitive Workflows

Goal: complete admitted Dedibox installation, rescue, boot, network, reverse,
firewall, and server mutation workflows.

Deliverables: protected credentials, atomic forms/JSON as source-defined,
mutation and destructive permits, explicit retry denial where idempotency is
absent, delivery phases, bounded task polling, and reconciliation guidance.

Verification: credential cleanup, firewall widening, reverse-DNS validation,
boot/install conflicts, lockout and auth rejection, uncertain delivery,
permit replay, non-executing staging, and mock end-to-end workflows.

Exit criteria: all assigned Dedibox mutations are typed and no sensitive or
disruptive operation uses a generic execution escape hatch.

Pentest stop: run an incremental pentest for the exact Commit 18 Dedibox
mutation surface.

## Commit 19 - Kubernetes

Goal: implement the complete admitted Kubernetes control-plane API.

Deliverables: clusters, pools, nodes, versions, ACLs, kubeconfig, upgrades,
autoscaling fields, regional paths, pagination, tasks, protected kubeconfig
output, permits, and all execution modes.

Verification: version and CNI/CNI-option conflicts, node-pool bounds,
autoscaling invariants, ACL widening, kubeconfig cleanup, upgrade reconciliation,
pagination, uncertain delivery, and read-only live smoke.

Exit criteria: every selected Kubernetes row is executable and downloaded
credentials remain protected through caller handoff.

Pentest stop: run an incremental pentest for the exact Commit 19 Kubernetes
surface.

## Commit 20 - Registry And Serverless Containers

Goal: implement admitted Container Registry and Serverless Container control
planes.

Deliverables: namespaces, images/tags where source-defined, containers,
deployments, domains, secrets, environment variables, scaling, cron and
trigger fields, pagination, costs, tasks, and all execution modes.

Verification: image-reference and domain validation, secret/environment
cleanup, scale and timeout limits, registry deletion authority, deployment
state, pagination, uncertain delivery, and read-only live smoke.

Exit criteria: registry and runtime resources are complete without treating
container image contents or runtime data planes as SDK control-plane payloads.

Pentest stop: run an incremental pentest for the exact Commit 20 container
surface.

## Commit 21 - IoT Hub

Goal: implement the complete admitted IoT Hub control-plane scope.

Deliverables: hubs, devices, routes, networks, certificates/keys, metrics,
events where source-defined, pagination, protected outputs, permits, and all
execution modes.

Verification: device identity, route destination and topic validation,
certificate/key cleanup, permission boundaries, metrics limits, duplicate
routes, uncertain delivery, and read-only live smoke.

Exit criteria: every selected IoT row is executable and device credentials
cannot escape protected output handling.

Pentest stop: run an incremental pentest for the exact Commit 21 IoT surface.

## Commit 22 - Managed PostgreSQL/MySQL Reads

Goal: complete admitted read-only RDB inventory, catalog, monitoring, and
configuration surfaces before database mutations.

Deliverables: instances, nodes, engines, versions, settings, users, databases,
backups, endpoints, logs, metrics, privileges, snapshots, pagination, and
checked large-response handling.

Verification: exact decimal and storage sizes, endpoint/address validation,
log and metrics bounds, unknown settings, pagination, malformed large payloads,
executor parity, and least-capability live smoke.

Exit criteria: every assigned RDB read is bounded, typed, and client-reachable.

Pentest stop: run an incremental pentest for the exact Commit 22 RDB read
surface.

## Commit 23 - Managed PostgreSQL/MySQL Mutations

Goal: complete admitted RDB provisioning, update, credential, backup, restore,
upgrade, failover, endpoint, and deletion workflows.

Deliverables: protected passwords, PATCH intent, cost/destructive permits,
tasks, retry/idempotency classifications, delivery phases, maintenance and
backup constraints, and reconciliation.

Verification: password cleanup, privilege escalation, restore-target mismatch,
backup deletion, maintenance conflicts, cost changes, task state, uncertain
delivery, permit replay, and non-executing staging.

Exit criteria: every assigned RDB mutation is executable only through explicit
authority and secret-safe handling.

Pentest stop: run an incremental pentest for the exact Commit 23 RDB mutation
surface.

## Commit 24 - Managed Redis And MongoDB

Goal: implement the complete admitted Redis and MongoDB control-plane APIs.

Deliverables: engines, versions, clusters/instances, users, ACLs, endpoints,
backups, settings, metrics/logs, protected credentials, pagination, tasks,
permits, reconciliation, and all execution modes.

Verification: credential cleanup, ACL and privilege escalation, endpoint
validation, backup/restore conflicts, scaling cost changes, malformed large
responses, uncertain delivery, and read-only live smoke.

Exit criteria: all selected Redis and MongoDB rows are complete and their
provider differences remain typed rather than forced into one lossy model.

Pentest stop: run an incremental pentest for the exact Commit 24 managed-data
surface.

## Commit 25 - Cockpit Observability

Goal: implement the complete admitted Cockpit control-plane scope.

Deliverables: endpoints, data sources, tokens, alerts, contacts, plans,
retention, usage, Grafana access, regional/global routing, protected outputs,
cost metadata, pagination, and all execution modes.

Verification: token cleanup, datasource URL authority, alert/contact injection,
retention and cost limits, Grafana credential redaction, pagination, uncertain
delivery, and read-only live smoke.

Exit criteria: every selected Cockpit row is executable without exposing
observability credentials or silently widening external destinations.

Pentest stop: run an incremental pentest for the exact Commit 25 Cockpit
surface.

## Commit 26 - Web Hosting

Goal: implement the finite admitted Web Hosting API exposed by the source lock.

Deliverables: offers, hosting, backups/restores, domains, databases and users,
FTP/mail accounts, websites, sessions, passwords, pagination, tasks, cost and
destructive permits, protected credentials, and all execution modes.

Verification: password cleanup, domain and DNS authority, account privilege,
backup item traversal, restore/delete conflicts, billable creation, session
redaction, uncertain delivery, and read-only live smoke.

Exit criteria: every selected Web Hosting row is complete and no credential or
restore path bypasses typed policy.

Pentest stop: run an incremental pentest for the exact Commit 26 Web Hosting
surface.

## Commit 27 - Generative And Dedicated Inference Boundaries

Goal: implement only the GA AI operations explicitly admitted by Commit 1 while
keeping control-plane and data-plane trust boundaries separate.

Deliverables: dedicated deployment control-plane models; any admitted
OpenAI-compatible request/response subset; bearer-versus-`X-Auth-Token`
separation; model, deployment, quota, token, streaming, and response-size
policies; and explicit exclusions for unsupported streaming or bulk payloads.

Verification: credential/header confusion, cross-authority token denial,
prompt/output redaction policy, SSE truncation if streaming is admitted,
token/rate-limit headers, model identifier and byte bounds, cost authority,
uncertain delivery, and non-executing staging.

Exit criteria: the support matrix states exactly which AI control and data
operations are executable; no generic JSON route implies full OpenAI API
compatibility.

Pentest stop: run an incremental pentest for the exact Commit 27 AI boundary.

## Commit 28 - Unified Scaleway Client And Workflow Drivers

Goal: make the secure end-to-end path uniform across every selected service.

Deliverables: official-endpoint client constructors, blocking, Send-async, and
local-async parity; bounded prepared requests; checked typed decoding; pagers,
waiters, reconciliation drivers, cleanup ownership, and compile-checked
examples for representative read, mutation, destructive, cost, and secret
workflows.

Verification: complete operation-to-client association, compile-fail provider
mixing tests, cross-executor differential scenarios, cancellation, cleanup,
transport faults, custom endpoint warnings, and no raw assembly requirement for
the documented happy path.

Exit criteria: every selected operation is reachable through one typed official
client and no execution mode changes policy or decoding behavior.

Pentest stop: run an incremental pentest for the exact Commit 28 client and
workflow surface.

## Commit 29 - Live Evidence, Fuzzing, And Platform Qualification

Goal: produce current adversarial and platform evidence for the completed
provider without granting CI mutation authority.

Deliverables: least-capability read-only live harness, secure local credential
ingestion, mock mutation staging, fuzz targets for paths, pagination, errors,
JSON, SRNs, IPs, tasks, secrets, and product-specific decoders; portable target
checks; SBOMs; package verification; and dependency/advisory review.

Verification: full fuzz build and bounded campaigns, deterministic regression
corpora, live-harness trust and permission tests, no-credential CI proof,
MSRV/stable/platform matrices, file-length policy, package contents, fresh
SBOMs, Cargo audit/deny, and reproducible archives.

Exit criteria: every public support, platform, dependency, and live-evidence
claim has current executable proof and CI cannot create, mutate, or delete a
Scaleway resource.

Pentest stop: run an incremental pentest for the exact Commit 29 qualification
surface.

## Commit 30 - Scope Freeze And Release Candidate

Goal: freeze and qualify the complete selected Scaleway provider without adding
features.

Deliverables: final API matrix and exact operation count; zero unclassified or
model-only rows; provider README and examples; threat model; authentication,
retry, cost, mutation, live-test, drift, deprecation, migration, and platform
documentation; release notes; package selection; provenance; and one candidate
release gate that composes all prior gates.

Verification: rerun all 30 commit gates, live source drift, full workspace and
provider tests, all execution modes, fuzz and adversarial suites, MSRV and
platform matrices, dependency and SBOM checks, public API and SemVer review,
package reproducibility from two clean clones, and green GitHub CI and CodeQL.

Exit criteria: all Commit 1 included rows are executable and documented; every
excluded row has a precise reason; no API, dependency, feature, or scope change
occurs after qualification; and the candidate can receive a version only after
the release decision is made.

Pentest stop: run a full-provider pentest for the exact Commit 30 candidate,
remediate and retest every finding, rerun the complete release gate, then wait
for green GitHub CI and CodeQL before selecting a version, signing a tag, or
publishing crates.

## Deferred Surfaces

Commit 1 must record exact rows, but the following are presumed deferred from
the first provider release unless source review justifies a narrower exception:

- every `v1alpha*`, `v2alpha*`, `v1beta*`, or `v2beta*` interface;
- Object Storage bucket/object data operations using the S3 protocol;
- products whose only official API is pre-GA, including IAM, Domains and DNS,
  Secret Manager, Key Manager, Serverless Functions, Serverless Jobs, File
  Storage, InterLink, and currently pre-GA data/analytics services;
- separately authenticated or differently hosted data planes not explicitly
  admitted in Commit 1;
- deprecated and superseded versions;
- provider-console-only workflows; and
- any product added upstream after the Commit 1 source lock.

Deferral is not permanent rejection. A later Scaleway release can add a
source-locked surface through a separate commit plan after its stability,
protocol, security, and maintenance costs are reviewed.

## Release Decision

This document deliberately does not name `1.1.0` or any other version. After
Commit 30 passes its full-provider pentest, complete release gate, GitHub CI,
and CodeQL, maintainers decide whether the accumulated compatible workspace
changes warrant `1.1.0` or another SemVer version. The `cloud-sdk-scaleway`
crate receives its own independently appropriate package version under the
post-1.0 workspace versioning policy.
