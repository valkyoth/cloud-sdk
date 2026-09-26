# Tenable Commit Plan

Status: researched implementation roadmap, surveyed on 2026-09-26. No provider
implementation, release version, tag, publication, or change to the existing
provider order is authorized. This is an additional candidate plan.

## Decision Summary

Build **one `cloud-sdk-tenable` crate**, with explicit product modules and
clients, following the separation of Hetzner Cloud and Robot rather than
flattening different products into one ambiguous API.

The proposed train contains **105 numbered implementation checkpoints**. They
are reviewed scopes, not exactly 105 Git objects: fixes add commits, and an
oversized checkpoint must be split before coding. The count is provisional
until licensed GraphQL and appliance contracts pass Commit 2. Do not compress
unknown APIs into a generic request method to preserve this estimate.

The target is every current, documented, supported Tenable API in the admitted
product/version matrix: the eleven REST specification families, Cloud Exposure
including current cloud Container Security, OT Exposure, Security Center
including Director, and the permitted Nessus API surface. Public preview or
documented licensed APIs are not excluded merely because access is restricted.
Private console APIs, retired interfaces, unsupported license modes and
arbitrary undocumented GraphQL fields are not part of that claim.

The eleven downloadable REST documents currently describe **695 operations**
across **520 document-local path entries**, with **28 explicit deprecated
flags**. These figures are not the total size of Tenable's APIs. GraphQL,
Security Center and Nessus require separate inventories, and prose/lifecycle
notices can retire operations without a machine-readable deprecated flag.

This is substantially more than a Vulnerability Management wrapper. Important
security boundaries include customer delegation, active scanning, collected
credentials, vulnerability evidence, private deployment TLS, GraphQL partial
errors, long-running exports and scan concurrency.

## Checkpoint Workflow

1. Record the then-current approved release tag and SHA before starting; the
   current published baseline is `v1.1.0`, not a permanently fixed baseline.
2. Work on `main`, implement one numbered scope, run its gate and commit.
3. Pentest the complete delta from the preceding accepted baseline to HEAD.
4. Commit remediations and retest that complete range until green.
5. Record accepted evidence and wait for green GitHub CI and CodeQL on the
   exact candidate. Reflect any CI fixes in the evidence.
6. Record the accepted SHA and proceed to the next checkpoint without a tag
   or publication.

Every checkpoint includes Goal, Deliverables, Verification, Exit criteria and
a Pentest stop. Product checkpoints inherit the full coverage/security rules
below. Final qualification includes a full-provider review; release versions
and publishing require a separate decision after all checkpoints pass.

## Survey And Source Evidence

The [navigation page](https://developer.tenable.com/reference/navigate) is an
entry point, not a complete inventory. The separate
[specification catalog](https://developer.tenable.com/reference/download-the-specs)
also lists Enclave APIs. The
[developer documentation index](https://developer.tenable.com/docs/llms.txt),
[reference index](https://developer.tenable.com/reference/llms.txt), product
guides and official SDK must be reconciled with those documents.

### Downloaded REST Specifications

Counts include deprecated entries and are scoped to each document. Shared
hosts do not imply shared product identifiers, contracts or credential rights.

| Product / specification | Paths | Operations | Deprecated flags | Product implementation |
| --- | ---: | ---: | ---: | --- |
| [Platform & Settings](https://developer.tenable.com/openapi/tenable-platform-settings.json) | 114 | 184 | 19 | 16-27 |
| [Vulnerability Management](https://developer.tenable.com/openapi/vulnerability-management.json) | 118 | 150 | 8 | 28-40 |
| [Web App Scanning](https://developer.tenable.com/openapi/web-app-scanning.json) | 36 | 46 | 0 | 41-44 |
| [Exposure Management](https://developer.tenable.com/openapi/exposure-management.json) | 24 | 24 | 0 | 45-47 |
| [PCI ASV](https://developer.tenable.com/openapi/pci-asv.json) | 6 | 6 | 0 | 48-49 |
| [MSSP](https://developer.tenable.com/openapi/mssp.json) | 25 | 34 | 1 | 50-53 |
| [Identity Exposure](https://developer.tenable.com/openapi/identity-exposure.json) | 88 | 131 | 0 | 54-60 |
| [Attack Surface Management](https://developer.tenable.com/openapi/asmcloudtenablecom-api-docs-oas3.json) | 73 | 79 | 0 | 61-65 |
| [Enclave Platform](https://developer.tenable.com/openapi/enclave-security-platform.json) | 2 | 3 | 0 | 66 |
| [Enclave Container Security](https://developer.tenable.com/openapi/enclave-container-security.json) | 31 | 35 | 0 | 67-69 |
| [Downloads](https://developer.tenable.com/openapi/downloads-api.json) | 3 | 3 | 0 | 70 |
| **Document totals** | **520** | **695** | **28** | |

The documents mix OpenAPI 3.0.0 and 3.1.2. Identity Exposure's 131 operations
have no operationId in this snapshot; derive stable product/method/path keys
rather than dropping them or using summary text as a unique identity.
A generated 667 non-deprecated-operation count is only a preliminary REST
candidate count, not a full-product completion metric.

Survey SHA-256 values, keyed by the catalog filename:

```text
tenable-platform-settings.json
  932cdf851b284c5ac287941fbf32c7ed81327f0d46b8b6460747f8df61de0040
vulnerability-management.json
  f5d5255181ba9351d09f824d0932167e8064f0858ee878f968db464b6f3af3bb
web-app-scanning.json
  f7fd6242ac134f1ad10ce32de8f97f27e9020dc0b912081e4d664efa1b9f4e4c
exposure-management.json
  1f1f9d6702f4d92bdd98f63cf780cd93f0475fdaf828b1a147b76701efc9337e
pci-asv.json
  8d5f09761f56eaaa6febde00ec5146c6c7067e8642e9a1d4572b6ea1389452a8
mssp.json
  023ff9222aaf522d08eb5ccea35817828ff9acd15546907c71e3fadc2279b9b3
identity-exposure.json
  373faffd7dd5f2416e96c8c0623d7acd826b3521cc20b7a6a9adf9f42df9b400
asmcloudtenablecom-api-docs-oas3.json
  465c06285e48e68beed96bccc5af47d2edd3ad68c9a3caf99fb8c42ab9d5a885
enclave-security-platform.json
  d880fe0beaec9c192e81ae7e7f33959f21c344f5de798d99ed1a45dc51afaacb
enclave-container-security.json
  578b82a3450cb52aa621dbc6aa38666b586f0554068b2444fbfdf75c90ef151f
downloads-api.json
  63aa7730faae0626e485ca19d13cc3b49e3810c04033a7403c8c8dbede2ee608
```

These are observations, not maintained implementation locks. Commit 1 repeats
retrieval and creates the reviewed lock artifacts with URLs, digests, dates,
licensing and source provenance. No protected customer schema or tenant data
is committed without permission and review.

### Additional Sources And Admission Gaps

| Family | Official evidence | Admission requirement | Checkpoints |
| --- | --- | --- | --- |
| Cloud Exposure / current cloud Container Security | [Integration guide](https://developer.tenable.com/docs/cloud-security-integrations), [full API docs](https://docs.app.tenable.com/docs/api) | Authorized documented GraphQL schema, product/tenant permissions and partner-validation constraints; the full API page was not accessible during this review | 2, 71-77 |
| OT Exposure | [GraphQL guide](https://developer.tenable.com/docs/ot-graphiql-playground), [integration guide](https://developer.tenable.com/docs/ot-integrations) | Authorized appliance schema and supported firmware/profile; examples alone are not a complete schema | 2, 78-82 |
| Security Center / Director | [API reference](https://docs.tenable.com/security-center/api/), [integration guide](https://developer.tenable.com/docs/sc-integrations) | Versioned resource/method/field ledger, including Director-only resources and edition differences | 2, 83-93 |
| Nessus | [Current API-key guide](https://docs.tenable.com/nessus/Content/GenerateAnAPIKey.htm), instance `/api#/overview` | Licensed instance documentation and allowed edition/use-case matrix; direct scan API restrictions must be respected | 2, 94-97 |
| Cross-product discrepancy check | [Official pyTenable](https://github.com/tenable/pyTenable), [SDK documentation](https://pytenable.readthedocs.io/en/stable/) | SDK presence is supporting evidence, not authority to expose private or retired routes | 1-4, 102 |

The observed pyTenable repository HEAD was
`4bc10c6229f1cd7a72fa77b184d120b87678606c`. Pin any implementation evidence
to an immutable revision when work starts. Do not import Python runtime code
or reproduce undocumented routes merely because pyTenable has a method.

Cloud Exposure's public guide describes an integration validation process and
specific polling/query limits. Those conditions must be recorded alongside the
schema. A successful request with empty results is not proof of complete
entitlement or correct integration. No product credentials or live scanning
were used for this planning survey.

### Important Scope Corrections

- **Exposure Management is not Cloud Exposure.** The former has the Tenable
  One REST inventory/attack-path APIs; the latter uses a separate GraphQL
  contract. Identity Exposure and ASM also have distinct clients and keys.
- **PCI is not a generic compliance automation API.** The current spec exposes
  six GET operations. Attestation submission, dispute creation and signing must
  not be invented. Complete API coverage does not confer PCI certification.
- **MSSP keys are a delegation boundary.** Parent credentials, generated child
  keys, account groups, branding and customer resources cannot share a mutable
  global tenant context.
- **Cloud Container Security and Enclave are different.**
  [Legacy cloud REST endpoints reached end of life](https://developer.tenable.com/changelog/end-of-life-for-legacy-container-security).
  Current cloud GraphQL and Enclave REST need independent coverage.
- **A public guide can contain outdated examples.** The generic
  [disclaimer](https://developer.tenable.com/reference/disclaimer) gives different
  concurrency figures from the dedicated
  [concurrency guidance](https://developer.tenable.com/docs/concurrency-limiting).
  The latter distinguishes exports, detailed requests, active scans and reports.
  Resolve applicable limits conservatively per operation; do not hard-code a
  single global number.
- **Nessus is license-qualified.** The current guide restricts direct scan
  configuration/launch except permitted enterprise solutions. API-key
  availability is not blanket authorization. No SDK feature bypasses this.
- **UI capability is not automatically a public API.** Products such as
  [Patch Management](https://docs.tenable.com/integrations/Tenable-Patch-Management/Content/welcome.htm)
  may integrate through existing VM/SC APIs. Catalog review must record whether
  a separate supported contract exists; do not invent Adaptiva/private-console
  routes. Discovering another supported public family requires a plan amendment.

Commit 2 is a hard prerequisite for an unconditional full-provider claim. If a
required supported contract cannot be obtained, stop and request the missing
authorized documentation or explicitly renegotiate scope with the user.
Do not mark the product complete, silently defer it, or manufacture endpoints.

## Crate And Module Design

Proposed layout, not code that already exists:

```text
crates/cloud-sdk-tenable/src/
  lib.rs
  client/
  shared/                         private provider helpers
  platform/
  vulnerability_management/
  web_app_scanning/
  exposure_management/
  pci_asv/
  mssp/
  identity_exposure/
  attack_surface_management/
  cloud_exposure/
    container_security/
  enclave/
    platform/
    container_security/
  ot_exposure/
  security_center/
    director/
  nessus/
  downloads/
```

Each product module separates requests, models, codecs, operations and tests
into small files. Product names remain visible in public import paths and
client types, for example `cloud_sdk_tenable::pci_asv` and
`cloud_sdk_tenable::mssp`. Do not merge VM, WAS, Nessus and Enclave scans into
one extensible enum that loses product-specific guarantees.

One Tenable crate does not mean one credential or one universal endpoint.
Cloud product clients may share a reviewed immutable cloud connection context;
Identity, ASM, Cloud Exposure, OT, Enclave, Security Center and Nessus receive
explicitly typed service/deployment contexts. Product IDs and job handles are
not interchangeable even when upstream represents them as the same integer
or UUID.

Use optional product features when they materially control generated code and
dependencies. Define their dependency graph in Commit 5, with empty defaults
and no hidden network/OS/runtime dependency. Publish per-product examples under
the one crate's documentation. Keep reusable HTTP, testkit, sanitization,
streaming and protocol helpers provider-neutral.

## Authentication And Deployment Boundaries

| Surface | Surveyed authentication / origin boundary |
| --- | --- |
| Platform, VM, WAS, Exposure Management, PCI, MSSP | Cloud access/secret pair in `X-ApiKeys`; service paths at `https://cloud.tenable.com` in the published specs |
| Identity Exposure | `x-api-key`, explicitly configured customer deployment; do not interpolate untrusted names into the spec's placeholder |
| ASM | `Authorization` at `https://asm.cloud.tenable.com/api/1.0`; lock the exact value format instead of assuming cloud-key or bearer syntax |
| Enclave | `x-apikey` access/secret pair, trusted custom appliance URL; the spec's `your-server-url` is not executable configuration |
| Security Center | Product-specific `x-apikey` pair or separately documented permitted session mechanism; versioned HTTPS appliance/Director profile |
| OT | `X-APIKeys` with `key=...`, not the cloud pair format; trusted appliance `/graphql` |
| Cloud Exposure / cloud Container Security | Explicit tenant/region and authorized schema-defined authentication; obtain the full contract rather than guess |
| Downloads | Public catalog and operation-specific optional bearer token at the documented download origin |
| Nessus | Qualified appliance key/session contract; separate credentials even where header spelling resembles cloud |

Header names are case-insensitive, but dash placement and value grammars matter.
The official [authorization guide](https://developer.tenable.com/docs/authorization)
distinguishes cloud and Enclave. Never infer credential compatibility from
header spelling. Product rules, token permissions and tenant identity apply
even when several products use the same host.

Appliance constructors accept explicitly trusted HTTPS hosts, ports and base
paths and configured private trust roots. No automatic host discovery,
unverified TLS, browser-session scraping or redirect-based credential routing.
Returned URLs, export tokens, uploaded file handles and download locations are
operation-, product-, deployment- and customer-bound.

## Shared Security And Coverage Contract

Every checkpoint inherits these requirements:

- Preserve the workspace's supported Rust/MSRV, no_std default graph, license
  policy and 500-line limit for code files. No version bump in this plan.
- Use the already admitted sanitization abstraction for secrets; use base64-ng
  when necessary. Review any new parser or crypto dependency. No custom
  cryptographic primitive, and FIPS remains separately deferred to Brynja.
- Source-lock every operation and field, including filter operators, optional
  settings, union branches, status/media variants, permissions and API versions.
  Missing models, serialization, checked decoding or client execution are
  distinct gaps, not "supported" endpoints.
- REST operation identity includes product, deployment profile, method and
  path. GraphQL identity includes schema revision, operation root, arguments,
  selected fields and relevant type/union variants. One `/graphql` endpoint
  cannot count as full GraphQL support.
- Provide checked blocking, local-async and Send-async paths where applicable,
  with qualified streaming adapters. Share existing workspace mechanisms,
  not parallel provider-specific runtimes.
- Separate HTTP errors, product envelope errors and GraphQL partial data.
  Unknown state, unavailable permission and empty results are not equivalent.
  Error Display/Debug/source chains must not disclose sensitive payloads.
- Protect API keys, third-party scan credentials, session/report/export tokens,
  evidence, request/response captures and generated configuration. Public
  metadata still needs bounded allocation and safe diagnostics.
- Treat scan launch, scheduled scan creation, agent instructions, connector
  tests, discovery and OT active queries as explicit potentially intrusive
  operations. Mutations and credential generation never run during ordinary
  read-only validation or retry automatically after uncertain delivery.
- Require least-privilege, single-tenant context and resource associations.
  Parent/child credentials, concurrency budgets, cursor caches and export jobs
  cannot bleed between MSSP customers.
- Apply [cloud rate policy](https://developer.tenable.com/docs/rate-limiting)
  and [API limitations](https://developer.tenable.com/docs/api-limitations)
  only to their documented products. Do not export cloud quotas to ASM,
  Cloud Exposure, OT or appliances. Use bounded delay, query and polling budgets.
- Build the recommended identifying
  [User-Agent](https://developer.tenable.com/docs/user-agent-header) without
  embedding secrets or spoofing an official Tenable client.
- Source parsers use bounded downloads, strict formats, redirect policies,
  wall-clock deadlines, immutable provenance and deterministic generation.
  No fetch failure can produce a green empty inventory.
- Tests cover success/error/boundaries/partial delivery/cancellation and feature
  combinations. Fixture failure must fail the test; independent oracles and
  fuzzing strengthen shared protocol code.
- No active probing of production/customer/industrial targets. Live mutations
  require explicit approval, licensed disposable resources, bounded scope and
  cleanup evidence. API support is not an authorization to scan.

## Work Breakdown

| Checkpoints | Scope |
| --- | --- |
| 1-15 | Source admission, crate architecture, security and protocol foundations |
| 16-27 | Platform and settings |
| 28-40 | Vulnerability Management |
| 41-44 | Web App Scanning |
| 45-47 | Exposure Management |
| 48-49 | PCI ASV |
| 50-53 | MSSP |
| 54-60 | Identity Exposure |
| 61-65 | Attack Surface Management |
| 66-69 | Enclave Platform and Container Security |
| 70 | Downloads |
| 71-77 | Cloud Exposure and current cloud Container Security |
| 78-82 | OT Exposure |
| 83-93 | Security Center and Director |
| 94-97 | Qualified Nessus |
| 98-105 | Unified workflows, adversarial/live evidence, scope closure and release qualification |

These groups are module boundaries, not separate packages or releases.
Commit 4 assigns every source row to an owner; each product checkpoint must
cover its whole assigned set, including nested variants not enumerated in the
short descriptions. No "remaining roots" checkpoint may absorb an unreviewably
large schema: split it and update numbering before implementation.

## Commit 1 - REST Catalog And Source Lock

Goal: establish a complete, reproducible portal inventory.

Deliverables: Lock all eleven published specifications, reference indexes, changelog and
role/permission guides; retain origin, retrieval time, digest, product, API track and
retirement evidence.

Verification: Reproduce the 695-operation survey, detect missing documents, duplicates,
unresolved references and prose-only deprecations; reject HTML/login responses
masquerading as JSON.

Exit criteria: Every REST operation is classified by product and lifecycle, with unknown
rows blocking scope approval.

Pentest stop: stop after Commit 1; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 2 - GraphQL And Appliance Contract Admission

Goal: define the full scope outside the portal OpenAPI files.

Deliverables: Obtain authorized Cloud Exposure/Container Security and OT schemas; lock
Security Center including Director, and licensed Nessus instance documentation. Record
supported product versions and customer/partner documentation redistribution
permissions.

Verification: Compare official product catalogs, SDK surfaces and reference pages;
reject private introspected fields, guessed contracts and undocumented version
equivalence.

Exit criteria: Required schemas and appliance profiles are available before dependent
implementation; missing access blocks approval rather than silently dropping a family.

Pentest stop: stop after Commit 2; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 3 - API Drift And Lifecycle Monitoring

Goal: detect changes across every independent API source.

Deliverables: Add offline verification, live comparison and separately approved refresh
modes; fingerprint operations, nested fields, GraphQL roots/types, permissions, quotas,
authorities and product lifecycle.

Verification: Fixtures for removed fields, enum changes, HTML layout changes, GraphQL
nullability, newly added specs, timeouts, redirects and incomplete downloads.

Exit criteria: Source failure cannot report no drift; every change has a
human-reviewable disposition and implementation owner.

Pentest stop: stop after Commit 3; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 4 - Field-Level Coverage And Generator Contracts

Goal: prevent superficial endpoint counts from becoming a completeness claim.

Deliverables: Ledger request/path/query/body fields, union branches, response variants,
status/media/error handling, execution modes, examples and tests; derive stable
product+method+path identities where operationId is absent.

Verification: Delete one field, mutation root, response branch or execution mapping and
require failure; test OpenAPI 3.0/3.1 semantics and deterministic collision-resistant
generated names.

Exit criteria: Every admitted operation and documented GraphQL field has a checkpoint
owner; generated models alone never count as supported.

Pentest stop: stop after Commit 4; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 5 - Provider Crate And Product Modules

Goal: add one provider without polluting the neutral foundation.

Deliverables: Create cloud-sdk-tenable with empty defaults, no_std foundation, explicit
alloc/Serde/protocol features, independent product modules and documentation skeleton;
retain neutral transport/sanitization/testkit.

Verification: Default and feature-combination builds, external-consumer paths,
dependency graphs, module privacy and 500-line code-file enforcement.

Exit criteria: No additional product-specific crates are introduced and existing
providers retain unchanged default graphs.

Pentest stop: stop after Commit 5; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 6 - Origins, Deployments, And TLS Trust

Goal: bind requests to the intended cloud or appliance deployment.

Deliverables: Safe service constructors, explicit trusted appliance HTTPS
origin/port/base-path profiles, regional source evidence, private CA integration,
redirect and artifact-target policy.

Verification: Cross-product/region confusion, suffix tricks, encoded separators,
userinfo, downgrade, hostile returned URLs, DNS deadlines and certificate failures.

Exit criteria: No verify=false escape hatch or cloud credential forwarding to arbitrary
hosts; configured appliances remain explicitly trusted destinations.

Pentest stop: stop after Commit 6; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 7 - Cloud Key Pairs And Credential Rotation

Goal: protect sessionless cloud authentication.

Deliverables: Typed access/secret pairs and exact header serialization, protected
mutable ingestion, zero-copy guarded access where supported, redaction and explicit
rotation semantics.

Verification: CRLF/semicolon injection, empty fields, duplicate headers,
source/drop/cancellation cleanup and rotation invalidating previous credentials.

Exit criteria: Keys are operation/origin-bound, no implicit key regeneration occurs, and
examples contain no live or hard-coded test secrets.

Pentest stop: stop after Commit 7; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 8 - Product-Specific Credentials And Sessions

Goal: keep similarly named authentication schemes non-interchangeable.

Deliverables: Distinct Identity, ASM, Enclave, Security Center, OT, Downloads, Cloud
Exposure and Nessus credential policies; only documented API session/token flows and
source-verified header value formats.

Verification: Compile/runtime cross-product rejection, expired sessions, revocation
races, token-bearing URLs, protected diagnostics and uncertain login delivery.

Exit criteria: Header-name similarity cannot reuse credentials across products;
unsupported browser-cookie automation is absent.

Pentest stop: stop after Commit 8; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 9 - Tenant, Permission, And Scan Intent Binding

Goal: prevent cross-customer actions and accidental active scanning.

Deliverables: Typed deployment/customer/container/resource identities, roles/custom
privileges, ReadOnly/Mutation/Destructive/ScanLaunch/credential-read distinctions and
endpoint retry/delivery classifications.

Verification: Wrong-tenant IDs, forged local permissions, mutation-through-GET cases,
bulk mixed tenants, active-scan targets and generic retry bypass.

Exit criteria: Local intent never claims server authorization; MSSP delegation is
explicit and every potentially intrusive request requires caller approval.

Pentest stop: stop after Commit 9; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 10 - REST Responses And Error Semantics

Goal: decode each family without treating transport success as business success.

Deliverables: Bounded JSON, product-specific envelopes, HTML/plaintext failures, binary
and empty responses, payload-free Display/Error and sensitive metadata ownership.

Verification: HTTP 200 provider errors, contradictory states, duplicate JSON
keys/headers, malformed numbers, unexpected media, truncated bodies, allocation failures
and 204 framing conflicts.

Exit criteria: Success/error policies are source-backed per operation; examples cannot
relax secure transport framing.

Pentest stop: stop after Commit 10; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 11 - Filters, Pagination, And Date Semantics

Goal: make bounded iteration and expressive queries safe.

Deliverables: Product-specific filter ASTs and encoding, offsets/cursors/links, field
projection/sort/recurrence/time types, loop budgets and absence/null/replacement
semantics.

Verification: Nested filter limits, injection, duplicate/cyclic pages, cross-tenant
cursors, inclusive/exclusive offsets, time overflow and cancellation.

Exit criteria: Supported filters remain expressible without raw URL assembly or silently
omitted records.

Pentest stop: stop after Commit 11; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 12 - Rate, Concurrency, And Retry Admission

Goal: honor service-specific limits without unbounded availability pauses.

Deliverables: Conservative cloud credential scheduling, container export/scan and
user-report budgets, bounded Retry-After, separate appliance/ASM/GraphQL policies and
explicit retry configuration.

Verification: Shared handles, per-tenant isolation, invalid/huge delays,
allocation/cancellation release, remote job slots outliving HTTP requests and duplicate
export rejection.

Exit criteria: No single global quota assumption; no scan, key generation or other
ambiguous mutation is automatically replayed.

Pentest stop: stop after Commit 12; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 13 - Asynchronous Jobs And Bulk Results

Goal: track export/scan work without collapsing partial outcomes.

Deliverables: Resource-bound job handles, explicit polling budgets,
terminal/error/partial states, cancellation and per-item bulk results; durable resume
descriptions without automatic persistence.

Verification: Out-of-order chunks, stale jobs, tenant mismatch, success-with-errors,
partial bulk failure and cancellation races.

Exit criteria: Unknown or contradictory state never maps to clean success; task handles
cannot change product or tenant.

Pentest stop: stop after Commit 13; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 14 - Streaming Imports, Exports, And Artifacts

Goal: support large and sensitive files without unbounded heap copies.

Deliverables: Bounded multipart sources, chunk decoders and transactional sinks;
compressed-size/expanded-size limits, integrity hooks and approved returned-download
authorities.

Verification: Partial writes, truncated compression, decompression bombs, hostile
filenames, mismatched hashes/lengths, secret buffers and cancellation/commit failures.

Exit criteria: Downloads are never automatically executed or extracted; complete,
validated output is committed only under the caller's sink contract.

Pentest stop: stop after Commit 14; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 15 - GraphQL Preparation And Checked Execution

Goal: make schema-driven GraphQL a reviewed protocol boundary.

Deliverables: Typed selection/variable builders and operation metadata, root/field
coverage, complexity/depth/alias/page bounds, errors+data policy and distinct
query/mutation handling; optional reviewed parser dependencies.

Verification: Fragment cycles, duplicate aliases, input injection, nullable bubbling,
partial data, unknown unions, oversized selections and mutation retries.

Exit criteria: No unrestricted query-string escape hatch substitutes for typed coverage;
HTTP 200 with GraphQL errors cannot silently report complete success.

Pentest stop: stop after Commit 15; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 16 - Platform API Access And Audit

Goal: implement API access policy and administrative observability.

Deliverables: Allowed-IP settings, activity logs and server metadata; explicit
access-changing intent with IPv4/IPv6 handling.

Verification: Self-lockout scenarios, mixed IP families, malformed ranges, audit
pagination, sensitive actors and server-version parsing.

Exit criteria: All assigned platform inspection/access operations are executable without
pretending the SDK can recover a remote lockout.

Pentest stop: stop after Commit 16; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 17 - Platform Users And Account Security

Goal: cover user administration and credential-changing workflows.

Deliverables: Users, roles assignment, authorizations, enablement, password changes, key
generation and documented two-factor operations.

Verification: Current-user restrictions, stale role changes, one-time-code leakage, key
rotation, partial multi-step setup and mutation replay.

Exit criteria: Account security operations have explicit permits and protected outputs;
no automatic fallback to weaker authentication.

Pentest stop: stop after Commit 17; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 18 - Platform Groups, Roles, And Permissions

Goal: represent current authorization resources exactly.

Deliverables: Group membership, custom roles/privilege catalogs, access-control
permissions and supported legacy-named but non-deprecated permission resources.

Verification: Required application-toggle privileges, overwrite versus patch, immutable
groups, AllAssets action restrictions and negative authorization fixtures.

Exit criteria: All non-deprecated access-control rows are covered; deprecated
access/target groups remain migration evidence only.

Pentest stop: stop after Commit 18; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 19 - Agents And Global Configuration

Goal: cover agent inventory and supported runtime configuration.

Deliverables: Agent details/list/rename/unlink, safe-mode summary, global settings and
exclusions.

Verification: Scanner association, secret metadata, unknown safe-mode states,
configuration replacement, unlink authority and version-specific fields.

Exit criteria: Every assigned agent/configuration row has typed request, response and
error coverage.

Pentest stop: stop after Commit 19; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 20 - Agent Groups, Profiles, And Tasks

Goal: support explicit fleet operations without hiding partial failures.

Deliverables: Agent groups, profiles, assignments, network changes, instructions, bulk
unlink and task status.

Verification: Cross-scanner IDs, duplicate members, mixed-tenant batches, restart
intent, profile replacement and interrupted task polling.

Exit criteria: Fleet changes preserve per-item outcomes; issuing instructions requires
active-operation approval.

Pentest stop: stop after Commit 20; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 21 - Scanner Inventory And Configuration

Goal: cover scanners and their sensitive deployment data.

Deliverables: Scanner registration/inspection, linking material, configuration and
documented lifecycle operations.

Verification: Wrong scanner/container, expired linking credentials, destructive unlink,
secret output and source-version differences.

Exit criteria: Scanner credentials remain protected and scanner management is distinct
from permission to launch scans.

Pentest stop: stop after Commit 21; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 22 - Scanner Groups, Profiles, And Bulk Tasks

Goal: complete scanner organization and control.

Deliverables: Group lifecycle/membership, profiles and supported asynchronous scanner
tasks.

Verification: Group ownership, ambiguous instruction delivery, replacement semantics,
bulk partial failures and cancellation.

Exit criteria: All scanner group/profile/task operations reconcile to the platform
ledger.

Pentest stop: stop after Commit 22; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 23 - Networks And Scanner Routing

Goal: manage logical networks with exact resource associations.

Deliverables: Network lifecycle, asset/scanner/agent movement and routing configuration.

Verification: Cross-network moves, default network restrictions, identifier confusion
and duplicate or partially applied associations.

Exit criteria: Network changes are explicit; host/CIDR validation never broadens a scan
target silently.

Pentest stop: stop after Commit 23; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 24 - Cloud And OT Connectors

Goal: cover connector management without connecting to downstream providers directly.

Deliverables: Source-locked cloud/OT connector lifecycle, sync/test actions, settings
and protected integration credentials.

Verification: Returned URLs, permission scope, connection-test side effects, credential
replacement and tenant/resource mismatch.

Exit criteria: Every connector row is executable; SDK actions do not autonomously fetch
supplied connector URLs.

Pentest stop: stop after Commit 24; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 25 - Managed Credentials And Dynamic Settings

Goal: support credential schemas without ordinary-string secret copies.

Deliverables: Credential type catalogs, managed credential CRUD, scan-specific
conversion and validated dynamic settings.

Verification: Required fields, masked/redacted placeholders on update, unknown secret
fields, conversion failures and drop/cancellation cleanup.

Exit criteria: Documented credential variants remain expressible and are redacted by
default across nested structures.

Pentest stop: stop after Commit 25; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 26 - Tags And Assignments

Goal: complete asset tagging and dynamic membership.

Deliverables: Category/value lifecycle, assignment operations, dynamic rules and
task/permission associations.

Verification: Duplicate/case-sensitive tags, replacement versus merge, rule limits,
scope changes and mixed-container assets.

Exit criteria: All tag rows are covered and dynamic matching never implies authority to
scan matched assets.

Pentest stop: stop after Commit 26; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 27 - Exclusions And Recast Rules

Goal: make suppression and risk changes conspicuous.

Deliverables: Scan exclusions, recast-rule lifecycle, schedules and scope bindings.

Verification: Invalid recurrence/timezone values, overbroad target patterns, unintended
severity changes and delete/reapply ambiguity.

Exit criteria: Suppression is classified as a security-impacting mutation, not ordinary
harmless configuration.

Pentest stop: stop after Commit 27; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 28 - VM Assets And Custom Attributes

Goal: cover inventory, lifecycle and custom property operations.

Deliverables: Asset reads/import-related records/bulk actions, custom attribute
definitions and values, source identity and timestamps.

Verification: UUID/IP confusion, null versus missing data, deleted/unassessed assets,
attribute type changes and partial batches.

Exit criteria: All current asset/attribute rows have typed execution; no data is dropped
because an attribute is unfamiliar.

Pentest stop: stop after Commit 28; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 29 - VM Editors, Plugins, And Policies

Goal: cover configuration discovery before scan assembly.

Deliverables: Editor catalogs/details, plugin families/details, policy
lifecycle/import/export and compliance-audit configuration.

Verification: Dynamic policy variants, plugin selection, credential placeholders,
multipart limits and policy ownership.

Exit criteria: Every supported configuration field is mapped; copying a policy does not
launch a scan.

Pentest stop: stop after Commit 29; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 30 - VM Scan Configuration And Organization

Goal: build exact scan definitions and folders.

Deliverables: Scan create/read/update/copy/delete, folder management, schedules, routing
and target encodings.

Verification: Missing required fields, omitted versus cleared settings, tag-based
targets, scanner selection, schedule bounds and accidental scope expansion.

Exit criteria: Configuration preserves every supported option and requires explicit
intent where creation can schedule execution.

Pentest stop: stop after Commit 30; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 31 - VM Scan Execution And Bulk Tasks

Goal: implement scan lifecycle with bounded remote-work authority.

Deliverables: Launch/pause/resume/stop/cancel and documented bulk tasks,
statuses/counts, remediation launch associations.

Verification: Unknown/partial states, active scan budgets, duplicate launch, timeout
after send, cancellation and target approvals.

Exit criteria: A scan is never launched, resumed or retried implicitly; remote
processing and local task completion remain distinct.

Pentest stop: stop after Commit 31; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 32 - VM Scan History And Results

Goal: inspect completed and partial scan evidence.

Deliverables: History, latest status, host/plugin results and supported scan detail
retrieval, with exact scan/history/host bindings.

Verification: Stale history IDs, truncated plugin output, HTML-like evidence, sensitive
host data, scope-limited responses and concurrent changes.

Exit criteria: All current result rows are available and partial results cannot be
mislabeled as complete.

Pentest stop: stop after Commit 32; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 33 - VM File Upload And Scan Import

Goal: support import workflows without running supplied content.

Deliverables: Bounded file upload and supported scan imports, import passwords,
scan/task ownership and format constraints.

Verification: Multipart filename injection, wrong uploaded handle, compressed limits,
malformed artifacts, duplicate import and uncertain delivery.

Exit criteria: Import contracts cover permitted formats; the SDK does not unpack or
execute scanner/plugin files.

Pentest stop: stop after Commit 33; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 34 - VM Asset Exports

Goal: provide complete incremental and full asset exports.

Deliverables: Supported v1/v2 request variants, status/jobs/chunks/cancel, filter
differences, resume state and deletion semantics.

Verification: Out-of-order/duplicate chunks, expired jobs, watermark overlap,
interrupted consumption and cancel failure.

Exit criteria: Exported data is complete only after verified job/chunk reconciliation,
not merely a successful first download.

Pentest stop: stop after Commit 34; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 35 - VM Vulnerability Exports

Goal: support bounded vulnerability extraction and refinement.

Deliverables: Current export filters/properties, jobs/status/chunks/cancel, plugin and
asset relationships, timestamps and risk models.

Verification: Missing or repeated chunks, filter association, sensitive plugin output,
resumed jobs and evolving documented properties.

Exit criteria: Every export field/filter is covered and partial history cannot
masquerade as a complete vulnerability inventory.

Pentest stop: stop after Commit 35; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 36 - VM Compliance Exports

Goal: preserve compliance-specific findings and evidence.

Deliverables: Compliance request/status/list/chunk/cancel operations, audit/result
models and bounded evidence.

Verification: Unknown check states, pass/fail/error distinctions, secret configuration
evidence, chunk mismatch and lifecycle races.

Exit criteria: Compliance data remains product-specific; SDK results do not certify
regulatory compliance.

Pentest stop: stop after Commit 36; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 37 - VM Scan Exports And Reports

Goal: complete scan/report artifact generation and retrieval.

Deliverables: Current scan export and report operations, formats/filters, status
tracking and streaming downloads.

Verification: Expiry, report/scan mismatch, CSV formula-like data, binary truncation,
content-type and filename ambiguity.

Exit criteria: Artifacts are safely delivered as data; reports are not rendered,
installed or treated as executable content.

Pentest stop: stop after Commit 37; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 38 - VM Vulnerabilities, Filters, And Workbenches

Goal: cover remaining current inspection and risk views.

Deliverables: Supported vulnerability/plugin-output, workbench and filter operations
with response variants and endpoint-specific limits.

Verification: Large detail responses, deprecated-route rejection, unstable pagination,
missing permissions and empty versus unavailable data.

Exit criteria: Current views are covered while retired imports/workbench export routes
are not resurrected.

Pentest stop: stop after Commit 38; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 39 - VM Remediation Goals, Projects, And Scans

Goal: support remediation management without implying automatic patching.

Deliverables: Goals/projects lifecycle and associated current remediation scan
operations.

Verification: Project/goal ownership, date arithmetic, target narrowing, deletion and
scan-launch consent.

Exit criteria: Management and active rescanning are distinct; completion cannot be
inferred from an accepted job.

Pentest stop: stop after Commit 39; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 40 - VM Exposure Response And Shared Collections

Goal: complete remaining collaborative VM workflows.

Deliverables: Exposure-response combinations/initiatives and shared-collection
operations with member/resource associations.

Verification: Cross-owner sharing, access expansion, duplicate associations, partial
updates and confidential data redaction.

Exit criteria: All remaining current VM tags reconcile to executable coverage, not a
generic unchecked payload endpoint.

Pentest stop: stop after Commit 40; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 41 - WAS Applications, Templates, And Configuration

Goal: cover web scan definitions independently of VM scan types.

Deliverables: Applications, configurations, templates and folders; authenticated
scanning settings, target scope and schedule controls.

Verification: Cookie/header secrets, URL injection, redirects as target configuration,
omitted settings and schedule-driven launches.

Exit criteria: All configuration rows are typed; VM scan credentials or IDs cannot
accidentally select a WAS operation.

Pentest stop: stop after Commit 41; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 42 - WAS Scan Lifecycle

Goal: execute web scans only with explicit target authorization.

Deliverables: Current scan CRUD/control/status and related histories/results
associations.

Verification: Duplicate launches, active budgets, cancellation, partial scan states and
cross-configuration identifiers.

Exit criteria: Every lifecycle row is executable without automatic crawling or scan
launch by the SDK itself.

Pentest stop: stop after Commit 42; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 43 - WAS Findings, Plugins, Filters, And Attachments

Goal: inspect web findings while protecting request/response evidence.

Deliverables: Supported finding/plugin/filter and attachment APIs, bounded bodies and
typed result context.

Verification: Embedded credentials, hostile HTML, attachment filenames, pagination and
untrusted evidence links.

Exit criteria: Evidence remains inert and protected; no finding link is automatically
fetched or rendered.

Pentest stop: stop after Commit 43; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 44 - WAS Exports

Goal: complete web finding export workflows.

Deliverables: Request/status/jobs/chunks/cancel and source-locked formats,
product-specific filters and stream handling.

Verification: Partial jobs, duplicate/expired chunks, tenant mismatch and cancellation
during writes.

Exit criteria: WAS export completeness is independently verified rather than assumed
from VM export behavior.

Pentest stop: stop after Commit 44; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 45 - Exposure Inventory, Tags, And Properties

Goal: cover Tenable One inventory search without conflating Cloud Exposure.

Deliverables: Asset/finding/software search, property catalogs, tag search and field
selection.

Verification: POST-as-read classification, query complexity, cross-type IDs, pagination
and unknown property variants.

Exit criteria: All assigned inventory rows have bounded typed queries and full
documented field coverage.

Pentest stop: stop after Commit 45; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 46 - Exposure View And Attack Paths

Goal: represent risk cards and attack graph results without causal overclaims.

Deliverables: Exposure View card operations, attack-path/technique searches and
connector sync logs.

Verification: Graph cycles/size, cross-card IDs, changing score versions, secret log
data and stale source timestamps.

Exit criteria: Risk scores and attack paths are evidence from Tenable, not SDK-generated
assurances.

Pentest stop: stop after Commit 46; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 47 - Exposure Inventory And Attack-Path Exports

Goal: cover both distinct export namespaces and their output contracts.

Deliverables: Inventory asset/finding jobs and chunks; attack path/technique/heatmap
exports and artifact status/download.

Verification: Wrong export family, reused IDs, partial graphs, media variants, retention
expiry and output cancellation.

Exit criteria: Every export row is executable with namespace-bound job handles and
accurate completion reporting.

Pentest stop: stop after Commit 47; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 48 - PCI ASV Attestations And Scans

Goal: implement the documented PCI inspection surface without inventing submission APIs.

Deliverables: Attestation list/details and PCI scan list, typed status and
account-scoped pagination.

Verification: Attestation/scan mismatch, unknown status, limited permission, empty
datasets and sensitive evidence.

Exit criteria: Current PCI read operations work and documentation clearly distinguishes
API coverage from ASV certification.

Pentest stop: stop after Commit 48; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 49 - PCI ASV Disputes, Failures, And Assets

Goal: complete the remaining PCI evidence views.

Deliverables: Dispute listing, undisputed failures and attestation assets; evidence
associations and bounded result models.

Verification: Duplicate/cross-attestation rows, missing evidence, unexpected dispute
states and incomplete pagination.

Exit criteria: All six surveyed PCI operations are covered; dispute creation, submission
and signing are not fabricated.

Pentest stop: stop after Commit 49; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 50 - MSSP Accounts, Quotes, And License Views

Goal: cover commercial/customer account operations with explicit intent.

Deliverables: Current evaluation-account creation, quote creation, child account
details/list, partner/license/dashboard/filter reads.

Verification: Duplicate provisioning, evaluation expiry, monetary/quantity bounds,
unavailable products and parent/child identity mismatch.

Exit criteria: Deprecated evaluation v1 stays excluded; quotes and provisioning never
run as read-only discovery.

Pentest stop: stop after Commit 50; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 51 - MSSP Child Credentials And Delegation

Goal: make tenant switching impossible to perform accidentally.

Deliverables: Child-container list/history and protected key generation; explicit
child-bound client construction, generation/rotation state and resource scope.

Verification: Concurrent children, credential-cache collisions, parent-to-child replay,
wrong child metadata, cancellation and error redaction.

Exit criteria: Every execution remains bound to one verified customer context; no
mutable global current-tenant selector.

Pentest stop: stop after Commit 51; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 52 - MSSP Account Groups And Domains

Goal: manage customer grouping and domain verification.

Deliverables: Group CRUD, domain operations and activation-code workflows.

Verification: Cross-group membership, ownership conflicts, IDN/host parsing,
activation-code leakage and uncertain verification delivery.

Exit criteria: Assignments preserve parent/child boundaries and verification is an
explicitly authorized workflow.

Pentest stop: stop after Commit 52; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 53 - MSSP Branding And Resource Links

Goal: complete logos, assignments and customer links safely.

Deliverables: Logo CRUD/PNG/base64 download, account assignment and single/bulk resource
links.

Verification: Malformed base64, oversized images, metadata injection, cross-account
assignments and unsafe link schemes.

Exit criteria: All remaining current MSSP rows are covered; images and resource URLs are
returned as inert data.

Pentest stop: stop after Commit 53; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 54 - Identity Deployment And Configuration

Goal: admit Identity's distinct endpoint and API-key model.

Deliverables: About/application settings, API keys, deployment profile and
version-specific wire conventions.

Verification: Customer-origin mismatch, product key confusion, unknown versions,
configuration overwrite and secret key generation.

Exit criteria: No Identity method relies on cloud X-ApiKeys or a guessed customer
hostname.

Pentest stop: stop after Commit 54; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 55 - Identity Users, Roles, Profiles, And Preferences

Goal: implement identity-product authorization and personalization.

Deliverables: Users, roles, profiles, preferences and lockout policy with precise write
semantics.

Verification: Cross-profile access, role escalation intent, immutable fields, lockout
edge cases and masked secret updates.

Exit criteria: All assigned authorization rows are typed and share no accidental
platform role enum.

Pentest stop: stop after Commit 55; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 56 - Identity Directories, Objects, And Topology

Goal: cover directory inventory and relationships.

Deliverables: Directory/infrastructure/AD object operations, topology and relays; exact
identifiers and bounded graph models.

Verification: Cyclic relationships, large attribute sets, path encoding, partial
directory visibility and operation-specific mutations.

Exit criteria: Documented directory operations are covered without initiating LDAP
connections from the SDK.

Pentest stop: stop after Commit 56; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 57 - Identity Checkers, Categories, And Options

Goal: represent exposure-detection configuration.

Deliverables: Checker/checker-option, category, attack-type configuration and option
operations.

Verification: Unsupported option variants, numeric bounds, profile association and
disabling a detection rule.

Exit criteria: Rule configuration carries explicit security-impacting intent and all
supported variants have fixtures.

Pentest stop: stop after Commit 57; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 58 - Identity Deviances, Reasons, Events, And Attacks

Goal: handle security findings and investigation workflows.

Deliverables: All current deviance/reason/event/attack/type operations, state changes
and evidence relationships.

Verification: Suppression intent, inconsistent finding state, large evidence, ordering
and case/identifier collisions.

Exit criteria: Investigation results preserve partial/error states and cannot silently
hide unresolved findings.

Pentest stop: stop after Commit 58; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 59 - Identity Notifications And Authentication Integration

Goal: cover notification and identity integration configuration.

Deliverables: Alerts, email notifiers, syslog, LDAP and SAML configuration with
secret-aware updates.

Verification: SSRF-like configuration values remain inert, XML bounds where parsed,
test-message side effects and credential echoes.

Exit criteria: No supplied notifier/LDAP/SAML URL is fetched automatically; all
supported integrations remain configurable.

Pentest stop: stop after Commit 59; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 60 - Identity Dashboards, Metrics, Reports, And License

Goal: complete visibility and entitlement APIs.

Deliverables: Dashboards/widgets, cloud statistics, metrics, scores, report access
tokens and licensing operations.

Verification: Cross-widget IDs, metric units, secret report tokens, expiry and
entitlement failures.

Exit criteria: All Identity tags reconcile to executable coverage; a token is not
treated as an unrestricted download URL.

Pentest stop: stop after Commit 60; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 61 - ASM Inventory, Assets, And Search

Goal: separate ASM inventories and query semantics from VM assets.

Deliverables: Inventory lifecycle, asset properties/global search, details,
dashboard/global/log/text-record views and supported asset actions.

Verification: Wrong inventory context, dynamic columns, nested filters, result bounds,
hostname/IP edge cases and unavailable fields.

Exit criteria: All assigned rows are covered without assuming one inventory per
credential.

Pentest stop: stop after Commit 61; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 62 - ASM Sources And Discovery Control

Goal: manage discovery sources as potentially active operations.

Deliverables: Documented DNS/IP/ASN/TLD/cloud sources, bulk add/delete, source
inspection and suggestions.

Verification: Unowned scope, mass-add limits, unexpected ASN expansion, duplicate
sources and partial discovery outcomes.

Exit criteria: Discovery changes require explicit scope approval; importing a source
cannot bypass the scan intent policy.

Pentest stop: stop after Commit 62; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 63 - ASM Cloud Integration Credentials

Goal: cover ASM-specific cloud account connectors.

Deliverables: AWS/Azure/Cloudflare/GCP key and keyless operations, external identifiers
and zones where documented.

Verification: Cross-business keys, provider-token leakage, confused external IDs, secret
replacement and returned URL handling.

Exit criteria: Credentials cannot be used as direct cloud-sdk provider credentials
without a separate caller decision.

Pentest stop: stop after Commit 63; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 64 - ASM Tags, Smart Folders, And Bulk Actions

Goal: complete grouping, subscriptions and asset manipulation.

Deliverables: Tags, memberships, filter-based assignments, smart-folder
lifecycle/history/recent views, alerts and bulk asset actions.

Verification: Filter scope expansion, cross-inventory assignments, subscriptions with
side effects and partial bulk failures.

Exit criteria: Every remaining grouping/action row has bounded filters and explicit
mutation semantics.

Pentest stop: stop after Commit 64; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 65 - ASM Exports And Downloads

Goal: support export tokens and formats without unsafe rendering.

Deliverables: CSV/XLSX/JSON asset/source exports and token-bound download, expiration
and streaming sinks.

Verification: CSV formula-like values preserved as data, opaque tokens, malformed
filenames, wrong inventory and truncation.

Exit criteria: All ASM exports are executable; spreadsheet parsing/execution is not part
of artifact retrieval.

Pentest stop: stop after Commit 65; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 66 - Enclave Platform Licensing

Goal: implement the separate appliance platform contract.

Deliverables: License upload/details/utilization, blade models and explicit trusted
Enclave origin with x-apikey credential type.

Verification: Malformed license files, expired capacity, wrong blade, upload bounds and
credential/header confusion.

Exit criteria: All three platform operations are covered without executing license
payloads or claiming regulatory accreditation.

Pentest stop: stop after Commit 66; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 67 - Enclave Container Inventory And Queries

Goal: cover current container findings and query catalogs.

Deliverables: Image/layer/package/vulnerability search/count, registry/source trees,
tags and supported filter/field operations.

Verification: Query/type mismatch, duplicate images, digest formats, large dependency
graphs and undocumented filters.

Exit criteria: Inventory coverage follows the Enclave specification, not retired cloud
container routes.

Pentest stop: stop after Commit 67; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 68 - Enclave Scanners And Scans

Goal: complete scanner deployment and container scan control.

Deliverables: Scanner lifecycle/download/jobs, scan CRUD/run/stop and asset-age-out
settings.

Verification: Binary versus cluster configuration, credentials in deployment files,
active scan approval, deletion and ambiguous run delivery.

Exit criteria: All scanner/control rows are executable; downloaded
scanners/configuration are not installed or launched.

Pentest stop: stop after Commit 68; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 69 - Enclave Container Exports And SBOMs

Goal: deliver documented container artifacts and evidence.

Deliverables: Image/vulnerability relation exports, SBOM downloads and bounded output
metadata.

Verification: Cross-image associations, archive limits, content-type mismatch, malformed
filenames and cancellation.

Exit criteria: All current export rows work with safe sinks and do not imply automatic
trust in scanned software.

Pentest stop: stop after Commit 69; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 70 - Product Downloads

Goal: support catalog and authenticated artifact retrieval.

Deliverables: Product pages, file lists and downloads; separate optional download bearer
credentials, platform/version selection and integrity metadata where provided.

Verification: Anonymous versus authenticated routes, download redirects, filename
traversal, missing integrity evidence and partial writes.

Exit criteria: All three download operations are executable; no automatic acceptance of
licenses, installation, extraction or code execution.

Pentest stop: stop after Commit 70; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 71 - Cloud Exposure Schema And Query Authorization

Goal: admit the customer-visible GraphQL contract, not example queries alone.

Deliverables: Verified tenant/region origin and auth, authorized schema snapshot,
documented root/field inventory, partner-validation conditions and query-specific
scheduling.

Verification: Wrong tenant, undocumented/introspected fields, zero results due to
missing authorization, schema revision mismatch and partial GraphQL errors.

Exit criteria: The full supported schema is obtained and reviewed; absent access blocks
this family instead of counting samples as complete.

Pentest stop: stop after Commit 71; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 72 - Cloud Exposure Inventory And Relationships

Goal: cover the admitted cloud resource query surface.

Deliverables: Typed documented inventory/resource variants, account/provider relations,
selected properties, cursor pagination and bounded traversal.

Verification: Unknown unions, nested relationship limits, missing pageInfo, looped
cursors and account filters.

Exit criteria: Every assigned inventory field is selectable without arbitrary query
strings or fetching resource URLs.

Pentest stop: stop after Commit 72; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 73 - Cloud Exposure Findings And Policies

Goal: represent posture, policy and risk results.

Deliverables: Source-locked findings/policy/remediation/compliance fields and documented
filter families.

Verification: Severity/status mismatches, unresolved findings, partial data, evidence
redaction and account scoping.

Exit criteria: All admitted fields are covered; remediation instructions are inert
content, not executed commands.

Pentest stop: stop after Commit 73; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 74 - Cloud Exposure Vulnerabilities And Software

Goal: cover vulnerabilities and affected-resource associations.

Deliverables: Documented vulnerability and instance/software query families,
exposure/risk fields and operation-specific page limits.

Verification: Numerical boundaries, cross-resource links, huge result sets and omitted
optional data.

Exit criteria: Page limits come from the specific query contract and every vulnerability
variant has typed decoding.

Pentest stop: stop after Commit 74; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 75 - Cloud Exposure Identities And Permissions

Goal: complete admitted identity/entitlement graph selections.

Deliverables: All documented identity, access and relationship fields assigned by the
admitted schema, with bounded traversal and explicit least-data selection.

Verification: Graph cycles, path explosion, cross-account identities, hidden permissions
and partial-error responses.

Exit criteria: Coverage is measured against authorized documented fields, not every
internal introspection result.

Pentest stop: stop after Commit 75; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 76 - Cloud Container Security GraphQL

Goal: cover current cloud container APIs separately from Enclave REST.

Deliverables: Documented image/package/layer/finding and related query fields from the
current schema, protected registry metadata and exact product entitlement.

Verification: Cloud/Enclave key mismatch, legacy route rejection, nested graph bounds
and unavailable entitlement.

Exit criteria: Current cloud container coverage has its own schema evidence; the retired
REST inventory remains excluded.

Pentest stop: stop after Commit 76; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 77 - Cloud Exposure Remaining Roots And Mutations

Goal: close every admitted GraphQL contract not covered by preceding query groups.

Deliverables: Explicit root-to-checkpoint reconciliation; typed supported mutations,
remaining query fields and subscription protocols only where actually documented. Split
additional root families before coding if needed.

Verification: Mutation intent, selection variants, operation-name binding, no automatic
replay, partial data and source drift.

Exit criteria: Zero admitted roots/fields remain without execution tests; unsupported
mutations/subscriptions are evidence-backed, never invented.

Pentest stop: stop after Commit 77; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 78 - OT Deployment And Schema Profile

Goal: establish OT's separate GraphQL and credential boundary.

Deliverables: Authorized versioned schema, exact /graphql route and X-APIKeys key=
format, trusted appliance TLS profile, documented supported operations and limits.

Verification: Cloud-pair confusion, private CA failures, wrong firmware/profile,
introspection-only private fields and malformed credentials.

Exit criteria: A verified OT schema/profile is available before product operations and
remote execution is disabled by default.

Pentest stop: stop after Commit 78; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 79 - OT Assets, Networks, And Groups

Goal: cover passive inventory and documented topology queries.

Deliverables: Asset details, networks, protocol/port/asset groups and supported
relationships; bounded GraphQL selection and pagination.

Verification: Large/cyclic graphs, stale assets, unfamiliar device classes,
cross-appliance IDs and unavailable fields.

Exit criteria: All assigned OT inventory roots/fields are covered without contacting
industrial devices directly.

Pentest stop: stop after Commit 79; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 80 - OT Events, Vulnerabilities, And Policies

Goal: cover monitoring evidence and policy inspection.

Deliverables: Event aggregations/details, system logs, plugins/vulnerability data and
admitted policy query variants.

Verification: Event ordering, source/destination identity, counts versus actual pages,
evidence secrecy and partial results.

Exit criteria: All assigned monitoring fields are executable and incomplete observations
do not become a clean security state.

Pentest stop: stop after Commit 80; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 81 - OT Authorized Administration And Active Operations

Goal: isolate safety-sensitive changes and remaining supported mutations.

Deliverables: Typed schema-supported configuration/policy/group administration and
active queries/actions, explicit operational authorization and capability matrix.

Verification: Read-versus-active misclassification, unsafe default scheduling,
cancellation, unknown outcomes and cross-device scope.

Exit criteria: Every admitted mutation has explicit intent and test evidence; no
production OT probing or automatic remediation runs during CI.

Pentest stop: stop after Commit 81; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 82 - OT Exports And Contract Closure

Goal: complete documented transfer/subscription surfaces and remaining schema coverage.

Deliverables: Supported export/download workflows and documented subscriptions if
present; root/field closure and version compatibility evidence.

Verification: Malformed streamed data, disconnect/reconnect, partial output, unknown
schema version and export authority mismatch.

Exit criteria: All OT contract rows are covered; absent protocols are documented from
evidence rather than guessed from GraphQL capabilities.

Pentest stop: stop after Commit 82; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 83 - Security Center Version And Authentication Profile

Goal: establish the appliance and Director REST contract.

Deliverables: Versioned resource inventory, /rest and documented management paths,
source-verified key/session auth, organizations, envelopes and field-expansion
semantics.

Verification: HTML/example parsing differences, error_code in HTTP success, wrong
edition/version and unsafe HTTP examples.

Exit criteria: Only trusted HTTPS profiles execute; every current resource has an
implementation owner and required edition.

Pentest stop: stop after Commit 83; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 84 - Security Center Users, Organizations, And Access

Goal: cover multi-organization authorization.

Deliverables: Current user/org, users/groups/roles, organization membership/security
managers, TES role/permission resources and supported LDAP/SAML configuration.

Verification: Cross-organization shares, restricted role transitions, authentication
configuration locks and masked passwords.

Exit criteria: Every access row is typed and no cloud/MSSP identifier can select an
appliance organization.

Pentest stop: stop after Commit 84; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 85 - Security Center Assets, Hosts, And Repositories

Goal: implement inventory and repository management.

Deliverables: Assets/templates/attribute sets, hosts, repositories and supported bulk
inventory operations.

Verification: Repository/source ownership, IP ranges, field expansion, destructive
changes and large result pagination.

Exit criteria: Every assigned inventory row is executable with organization-bound
identifiers.

Pentest stop: stop after Commit 85; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 86 - Security Center Policies, Credentials, And Audit Content

Goal: cover scan configuration and security content.

Deliverables: Policies/templates, credentials/SSH keys, audit files/templates,
plugins/families/custom plugin administration.

Verification: Credential redaction, multipart bounds, policy unions, untrusted
audit/plugin content and unsupported field injection.

Exit criteria: Configuration is complete without executing or interpreting downloaded
plugins and audit scripts.

Pentest stop: stop after Commit 86; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 87 - Security Center Sensors And Scan Zones

Goal: cover active/passive scanner infrastructure.

Deliverables: Scanners, zones, passive/NNM resources, WAS scanners, sensor proxies and
linking keys where supported by the locked version.

Verification: Scanner/zone association, delegated systems, version-only resources,
secret linking material and destructive unlink.

Exit criteria: All admitted sensor resources are covered; edition-specific differences
cannot silently select another endpoint.

Pentest stop: stop after Commit 87; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 88 - Security Center Scan And Agent Workflows

Goal: complete licensed scan orchestration.

Deliverables: Scans, agent scans/groups/result synchronization, WAS scans, freeze
windows and supported job control.

Verification: Active-operation approval, frozen windows, incorrect repositories,
duplicate launch and uncertain delivery.

Exit criteria: Every scan/control row is executable and launch permissions remain
separate from configuration reads.

Pentest stop: stop after Commit 88; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 89 - Security Center Results, Queries, And Analysis

Goal: provide typed analysis instead of a generic arbitrary query payload.

Deliverables: Scan results, saved queries, analysis tools/variants, filters, projections
and source types.

Verification: Inclusive/exclusive offsets, tool-specific schemas, incorrect history IDs,
truncated outputs and partial status.

Exit criteria: Each documented query/analysis variant has field-level request and
response evidence.

Pentest stop: stop after Commit 89; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 90 - Security Center Risk Rules And Alerts

Goal: cover risk treatment and notification controls.

Deliverables: Accept-risk/recast rules, solutions and alerts/notifications with exact
scope and lifecycle semantics.

Verification: Overbroad suppression, expiry boundaries, automated action side effects
and permission mismatches.

Exit criteria: Risk acceptance is explicit and never presented as fixing the underlying
vulnerability.

Pentest stop: stop after Commit 90; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 91 - Security Center Reports, Dashboards, And Assurance Cards

Goal: cover reporting and visualization resource APIs.

Deliverables: Reports/definitions/images/templates, dashboard components/tabs/templates,
ARC/templates, style/families and publishing sites.

Verification: Cross-org sharing, artifact injection, scheduling side effects,
renderer-free downloads and partial generation.

Exit criteria: Every current reporting resource is covered without automatically
publishing or rendering evidence.

Pentest stop: stop after Commit 91; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 92 - Security Center System, Feeds, And Integrations

Goal: complete appliance administration and remaining resource groups.

Deliverables: Configuration/sections, device/system/status, licenses, feeds, files/jobs,
MDM, LCE-related current resources, Lumin and documented instance integration.

Verification: Sensitive configuration, restart/update intent, malformed binary imports,
disabled product editions and source-version mismatch.

Exit criteria: No residual current resource is hidden in an untyped passthrough; retired
resources have dated disposition evidence.

Pentest stop: stop after Commit 92; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 93 - Security Center Director Federation

Goal: cover Director-only management without weakening tenant separation.

Deliverables: Director insights, organizations, repositories, scans, scanners, policies,
results, zones, systems and users.

Verification: Director-to-managed-instance binding, ID collisions, cross-org delegation,
partial fleet results and fan-out bounds.

Exit criteria: All documented Director rows are executable only through a
Director-qualified profile.

Pentest stop: stop after Commit 93; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 94 - Nessus Contract And License Qualification

Goal: admit only supported and permitted direct Nessus API use.

Deliverables: Version/edition-specific instance API snapshot, license restrictions,
allowed administrative routes, credential/session profile and enterprise scanning
prerequisites.

Verification: Wrong edition, unavailable routes, unsupported scanning permission, schema
drift and documented versus private endpoint classification.

Exit criteria: The allowed matrix is reviewed before implementation; possessing a key is
not treated as permission to bypass direct-scan restrictions.

Pentest stop: stop after Commit 94; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 95 - Nessus Administrative And Sensor APIs

Goal: cover the qualified appliance administration surface.

Deliverables: Admitted server/settings/update metadata, users/groups/permissions,
agents/groups/scanners, proxy/mail/token/session and related management operations.

Verification: Version-specific fields, token secrecy, disruptive updates, proxy
credentials, accidental key regeneration and restart intent.

Exit criteria: All licensed administrative rows are covered, with
destructive/system-changing actions explicit.

Pentest stop: stop after Commit 95; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 96 - Nessus Qualified Configuration And Scan Operations

Goal: support direct scanning only within documented permitted enterprise use.

Deliverables: Admitted editor/plugin/rule/policy/folder configuration and permitted scan
lifecycle; strong capability/intent binding and enterprise integration examples.

Verification: Standalone unqualified clients cannot invoke scan configuration/launch,
target widening, duplicate actions and uncertain outcomes.

Exit criteria: No generic endpoint escape bypasses the documented restriction;
unsupported license modes remain explicitly unavailable.

Pentest stop: stop after Commit 96; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 97 - Nessus Results And Artifact Workflows

Goal: complete supported evidence and data transfers.

Deliverables: Qualified scan result/history/import/export/file and remaining documented
read workflows; full versioned contract reconciliation.

Verification: Protected outputs, malformed scan archives, truncated exports, mismatched
history and version drift.

Exit criteria: Every permitted row has execution coverage; the SDK does not claim
unrestricted Nessus automation.

Pentest stop: stop after Commit 97; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 98 - Unified Clients And Execution Coverage

Goal: make each product's secure path straightforward without one ambiguous universal
client.

Deliverables: Named product clients and constructors, common provider request contract,
blocking/local-async/Send-async execution and qualified streaming/GraphQL adapters.

Verification: External-consumer examples for every family, request-to-decoder binding,
feature combinations and cross-client credential rejection.

Exit criteria: All admitted operations are executable through typed clients without
manual HTTP assembly.

Pentest stop: stop after Commit 98; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 99 - Cross-Product And MSSP Workflow Isolation

Goal: qualify integrations without flattening products or customers.

Deliverables: Explicit MSSP-to-child client workflow, scan-to-export examples, permitted
appliance/cloud integrations and reusable neutral testkit fixtures.

Verification: Concurrent customers with identical IDs, cache/rate/poll token isolation,
task cancellation, replay and accidental data/credential forwarding.

Exit criteria: Cross-product workflows require deliberate caller transitions and
preserve origin, tenant, product and permission bindings.

Pentest stop: stop after Commit 99; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 100 - Adversarial, Differential, And Fuzz Qualification

Goal: test shared parser/protocol boundaries beyond examples.

Deliverables: Fuzz inventory, deterministic exact-bound corpora, allocation/failure
injection, JSON/GraphQL/filter/chunk/multipart/auth and appliance-envelope targets.

Verification: Every target builds with warning-denied lint; bounded campaigns,
independent parser oracles where applicable and fail-closed fixture checks.

Exit criteria: All protocol families have regression evidence and no skipped fixture
silently counts as coverage.

Pentest stop: stop after Commit 100; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 101 - Opt-In Live And Appliance Evidence

Goal: verify real behavior without making CI scan customer systems.

Deliverables: Least-privilege read probes, licensed disposable appliance fixtures,
per-product credential loading and separately authorized mutation/scan budgets and
cleanup.

Verification: No credentials by default, synthetic targets only, approval checks,
account/edition identification, unavailable entitlement and redaction.

Exit criteria: Live limitations are recorded honestly; no public/customer/industrial
target is scanned or changed without specific authorization.

Pentest stop: stop after Commit 101; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 102 - Fresh Full-Scope Reconciliation

Goal: prove completion against current upstream contracts.

Deliverables: Repeat all source and lifecycle checks, operation/field/root/execution
matrix and source discrepancy closure; inventory new products and changes during the
train.

Verification: Remove one public operation, field, appliance version gate or GraphQL
branch and require failure; compare live sources without auto-acceptance.

Exit criteria: Zero unclassified or model-only admitted rows remain; new public
contracts require extra pentested checkpoints before release.

Pentest stop: stop after Commit 102; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 103 - Platforms, Features, Dependencies, And Packages

Goal: qualify the expanded provider without regressing the workspace.

Deliverables: Documented MSRV/stable and OS/portable targets, feature graph matrix,
dependency/license/advisory review, tool freshness, SBOMs and package verification.

Verification: Default no_std, alloc/Serde/product/transport combinations, all-target
dependency policies, workspace tests, Clippy, formatting and reproducible packaging.

Exit criteria: Support claims match evidence; no unneeded product or network dependency
enters defaults and FIPS stays separately deferred.

Pentest stop: stop after Commit 103; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 104 - Product Documentation And Migration Review

Goal: make product boundaries and practical support understandable.

Deliverables: Crate README with module/capability table, per-product examples and
credential/tenant setup; license/access limitations, lifecycle migration and
offline/live drift instructions.

Verification: Compile examples in correct feature sets, test local links, compare
documentation with ledger and confirm no invented PCI mutations or unrestricted Nessus
claims.

Exit criteria: Users can choose a product without studying unrelated APIs; no version or
current-support claim is premature.

Pentest stop: stop after Commit 104; pentest the complete delta from the preceding
accepted checkpoint, remediate and retest, record the accepted SHA, and wait for green
GitHub CI and CodeQL before proceeding.

## Commit 105 - Final Candidate And Release Decision

Goal: qualify the complete admitted Tenable surface before selecting a version.

Deliverables: Final support matrix, threat model, pentest report, release notes,
migration/provenance evidence and one candidate gate.

Verification: Run the preceding 104 checkpoint gates plus candidate checks without
recursion; full workspace/adversarial/platform suites, fresh upstream/advisory checks,
clean-clone reproduction and green CI/CodeQL.

Exit criteria: Full-provider review and retest are accepted; post-qualification changes
invalidate affected evidence. Versions, tag and publication require a separate approval.

Pentest stop: run the full-provider pentest and retest for the exact candidate, rerun
all release checks, and wait for green GitHub CI and CodeQL. Only then request a
separate release/version/tag/publication decision.

## Maintenance And Completeness Gate

Commit 3 implements documented commands for:

1. Offline verification of every committed source lock and generated artifact.
2. Live comparison of all public catalogs/specifications/guides and authorized
   GraphQL/appliance profiles, without changing locks automatically.
3. Explicit reviewed source refresh with a semantic diff and required changes.
4. Operation/field/client/test coverage reconciliation against the current locks.

Command names are selected during implementation; they do not exist merely
because this plan describes them. CI and the final release gate run the
accumulated offline checks. Authorized live checks distinguish network/access
failure from drift and from genuine unchanged data.

Commit 102 repeats full product reconciliation before release. New supported
products, operations or fields found during the train get a reviewed additional
checkpoint and pentest rather than automatic deferral. Conflicting documents,
missing entitled schemas or unresolved license restrictions remain blockers.
A digest refresh cannot stand in for new models, execution or verification.

The candidate gate runs predecessor gates and candidate-specific verification,
never recursively itself. Any code, dependency or contract change after final
qualification requires the affected checks and evidence to be renewed.

## Explicit Non-Goals

- Implementing a vulnerability scanner, exploit engine, packet sensor, AD/LDAP
  client, plugin/NASL interpreter, remote shell, patch engine or desktop client.
- Accessing undocumented/private console endpoints, internal GraphQL fields,
  retired APIs, unavailable license capabilities or bypassing permissions.
- Treating Tenable integration validation, PCI ASV output, encryption or a
  green pentest as certification, vendor endorsement or regulatory approval.
- Automatically executing installers, reports, scanner downloads, uploaded
  archives, remediation commands or arbitrary links found in API responses.
- Implementing third-party cloud/registry/identity APIs merely because Tenable
  accepts credentials or URLs for those integrations.
- Browser scraping/session theft, unrestricted raw-request passthrough,
  automatic production scans or unattended destructive remediation.

These exclusions do not allow a documented supported Tenable API to disappear
from the ledger. A product without an independently documented public contract
needs a recorded source decision; a gated but supported API needs authorized
documentation, not an exclusion based solely on access difficulty.

## Release Decision

This plan does not assign a version or reorder the existing provider roadmap.
After Commit 105 passes full review, qualification and GitHub CI/CodeQL, choose
appropriate SemVer versions under the workspace's independent crate policy.
Publish only the approved changed packages in dependency order, after explicit
release approval. The `cloud-sdk-tenable` README must describe the implemented
product/version/license matrix, not claim all future Tenable APIs automatically.
