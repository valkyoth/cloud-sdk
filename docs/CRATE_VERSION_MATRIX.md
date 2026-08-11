# Crate Version Matrix

Status: `v0.75.0` is the latest published checkpoint and `v0.76.0` is the
Robot credential and lockout-policy source milestone at its implementation
stop. Every pre-1.0 tag receives its own incremental pentest; v0.76.0 remains
blocked until its pentest, final release gate, and GitHub CI and CodeQL are
green. Cumulative crates.io publication is deferred to v0.80.0.

`cloud-sdk` is the provider-neutral entry point. Provider crates such as
`cloud-sdk-hetzner` own their endpoint models in internal modules. Shared
transport, test, and secret-handling boundaries remain provider-neutral so the
workspace normally adds only one primary crate for each provider.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | `cloud-sdk` always follows the release/tag version. At public checkpoints, provider and boundary crates use one independent minor bump for cumulative code changes. | Public checkpoint only |
| `dependency` | At public checkpoints, provider and boundary crates patch-bump when an internal dependency range must change. | Public checkpoint only |
| `metadata` | At public checkpoints, use the independently required metadata version; the facade follows the tag. | Public checkpoint only |
| `unchanged` | Keep the previous published version. | No |

Every pre-1.0 tag advances `cloud-sdk` and receives incremental pentest
evidence, but intermediate tags select no package for crates.io. Supporting
crates retain their published versions while their code and metadata changes
accumulate. At every fifth-minor or exceptional public checkpoint, the release
tool compares each package tree with the previous public tag, rejects any lost
delta, verifies every intervening tag's pentest report, and publishes changed
packages in dependency order. Unchanged crates are not republished.

All minor and patch tags after the public baseline must appear in
`cumulative_milestones`; a checkpoint cannot omit an intervening build from
publication planning or the pentest-evidence chain.

## v0.1.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | none | `0.1.0` | `code` | Yes | Initial no_std provider-neutral cloud SDK foundation. |
| `cloud-sdk-hetzner` | none | `0.1.0` | `code` | Yes | Initial no_std Hetzner provider crate with internal Cloud, DNS, security, and Storage Box modules. |
| `cloud-sdk-hetzner-reqwest` | none | `0.1.0` | `code` | Yes | Initial optional reqwest transport adapter boundary without admitting reqwest yet. |
| `cloud-sdk-hetzner-sanitization` | none | `0.1.0` | `code` | Yes | Initial optional secret-sanitization boundary without admitting third-party dependencies yet. |
| `cloud-sdk-hetzner-testkit` | none | `0.1.0` | `code` | Yes | Initial testkit boundary for mock transports and fixtures. |

## v0.2.0 Tracking Table

`v0.2.0` is documentation, release-gate, source-lock, and drift-evidence work.
All workspace packages publish as `0.2.0` so crates.io metadata and release
evidence stay aligned.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.1.0` | `0.2.0` | `metadata` | Yes | Source-lock documentation and release metadata for the provider-neutral workspace. |
| `cloud-sdk-hetzner` | `0.1.0` | `0.2.0` | `metadata` | Yes | Source-locked Hetzner Cloud/DNS and Storage Box API matrix. |
| `cloud-sdk-hetzner-reqwest` | `0.1.0` | `0.2.0` | `metadata` | Yes | Keep optional transport boundary aligned with workspace release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.1.0` | `0.2.0` | `metadata` | Yes | Keep sanitization boundary aligned with workspace release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.1.0` | `0.2.0` | `metadata` | Yes | Keep testkit boundary aligned with workspace release evidence. |

## v0.3.0 Tracking Table

`v0.3.0` adds the core request and response policy domains used by later
endpoint builders. The provider-neutral facade and Hetzner provider both
publish as `0.3.0`; the optional boundary crates also publish metadata updates
so their crate-local README and rustdoc pages are available on crates.io.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.2.0` | `0.3.0` | `code` | Yes | Provider-neutral method token helper, crate-local documentation, and release evidence. |
| `cloud-sdk-hetzner` | `0.2.0` | `0.3.0` | `code` | Yes | Core no_std request/response policy domains for endpoint paths, query encoding, labels, pagination, actions, errors, and rate-limit metadata. |
| `cloud-sdk-hetzner-reqwest` | `0.2.0` | `0.3.0` | `metadata` | Yes | Publish crate-local README and rustdoc metadata aligned with v0.3.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.2.0` | `0.3.0` | `metadata` | Yes | Publish crate-local README and rustdoc metadata aligned with v0.3.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.2.0` | `0.3.0` | `metadata` | Yes | Publish crate-local README and rustdoc metadata aligned with v0.3.0 release evidence. |

## v0.4.0 Tracking Table

`v0.4.0` adds read-only Hetzner catalog request domains. The provider-neutral
facade follows the release tag, the Hetzner provider publishes a code release,
and optional boundary crates publish metadata-aligned packages because the
workspace still uses a shared package version.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.3.0` | `0.4.0` | `code` | Yes | Release metadata, README updates, and v0.4.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.3.0` | `0.4.0` | `code` | Yes | Read-only no_std catalog request domains for locations, pricing, server types, load balancer types, ISOs, and public images. |
| `cloud-sdk-hetzner-reqwest` | `0.3.0` | `0.4.0` | `metadata` | Yes | Keep optional transport boundary metadata aligned with v0.4.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.3.0` | `0.4.0` | `metadata` | Yes | Keep sanitization boundary metadata aligned with v0.4.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.3.0` | `0.4.0` | `metadata` | Yes | Keep testkit boundary metadata aligned with v0.4.0 release evidence. |

## v0.5.0 Tracking Table

`v0.5.0` adds Hetzner security request domains. The provider-neutral facade
follows the release tag, the Hetzner provider publishes a code release, and
optional boundary crates publish metadata-aligned packages because the
workspace still uses a shared package version.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.4.0` | `0.5.0` | `code` | Yes | Release metadata, README updates, and v0.5.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.4.0` | `0.5.0` | `code` | Yes | No_std security request domains for SSH key CRUD, certificate CRUD, and certificate retry action endpoints. |
| `cloud-sdk-hetzner-reqwest` | `0.4.0` | `0.5.0` | `metadata` | Yes | Keep optional transport boundary metadata aligned with v0.5.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.4.0` | `0.5.0` | `metadata` | Yes | Keep sanitization boundary metadata aligned with v0.5.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.4.0` | `0.5.0` | `metadata` | Yes | Keep testkit boundary metadata aligned with v0.5.0 release evidence. |

## v0.6.0 Tracking Table

`v0.6.0` adds Hetzner server request domains and shared no_std fixed-buffer
helpers. The provider-neutral facade follows the release tag, the Hetzner
provider publishes a code release, and optional boundary crates publish
metadata-aligned packages because the workspace still uses a shared package
version.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.5.0` | `0.6.0` | `code` | Yes | Shared no_std buffer helpers, README updates, and v0.6.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.5.0` | `0.6.0` | `code` | Yes | No_std server request domains for server CRUD, metrics, and source-locked server action endpoint paths. |
| `cloud-sdk-hetzner-reqwest` | `0.5.0` | `0.6.0` | `metadata` | Yes | Keep optional transport boundary metadata aligned with v0.6.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.5.0` | `0.6.0` | `metadata` | Yes | Keep sanitization boundary metadata aligned with v0.6.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.5.0` | `0.6.0` | `metadata` | Yes | Keep testkit boundary metadata aligned with v0.6.0 release evidence. |

## v0.7.0 Tracking Table

`v0.7.0` adds Hetzner server-adjacent request domains. The
provider-neutral facade follows the release tag, the Hetzner provider
publishes a code release, and optional boundary crates publish
metadata-aligned packages because the workspace still uses a shared package
version.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.6.0` | `0.7.0` | `code` | Yes | Release metadata, README updates, and v0.7.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.6.0` | `0.7.0` | `code` | Yes | No_std server-adjacent request domains for images, placement groups, primary IPs, and source-locked action endpoint paths. |
| `cloud-sdk-hetzner-reqwest` | `0.6.0` | `0.7.0` | `metadata` | Yes | Keep optional transport boundary metadata aligned with v0.7.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.6.0` | `0.7.0` | `metadata` | Yes | Keep sanitization boundary metadata aligned with v0.7.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.6.0` | `0.7.0` | `metadata` | Yes | Keep testkit boundary metadata aligned with v0.7.0 release evidence. |

## v0.8.0 Tracking Table

`v0.8.0` adds Hetzner storage/IP request domains. The provider-neutral facade
follows the release tag, the Hetzner provider publishes a code release, and
optional boundary crates publish metadata-aligned packages because the
workspace still uses a shared package version.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.7.0` | `0.8.0` | `code` | Yes | Release metadata, README updates, and v0.8.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.7.0` | `0.8.0` | `code` | Yes | No_std storage/IP request domains for volumes, floating IPs, and source-locked action endpoint paths. |
| `cloud-sdk-hetzner-reqwest` | `0.7.0` | `0.8.0` | `metadata` | Yes | Keep optional transport boundary metadata aligned with v0.8.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.7.0` | `0.8.0` | `metadata` | Yes | Keep sanitization boundary metadata aligned with v0.8.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.7.0` | `0.8.0` | `metadata` | Yes | Keep testkit boundary metadata aligned with v0.8.0 release evidence. |

## v0.9.0 Tracking Table

`v0.9.0` adds Hetzner Storage Box request domains. The provider-neutral facade
follows the release tag, the Hetzner provider publishes a code release, and
optional boundary crates publish metadata-aligned packages because the
workspace still uses a shared package version.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.8.0` | `0.9.0` | `code` | Yes | Release metadata, README updates, and v0.9.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.8.0` | `0.9.0` | `code` | Yes | No_std Storage Box request domains for boxes, types, snapshots, subaccounts, and source-locked action endpoint paths. |
| `cloud-sdk-hetzner-reqwest` | `0.8.0` | `0.9.0` | `metadata` | Yes | Keep optional transport boundary metadata aligned with v0.9.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.8.0` | `0.9.0` | `metadata` | Yes | Keep sanitization boundary metadata aligned with v0.9.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.8.0` | `0.9.0` | `metadata` | Yes | Keep testkit boundary metadata aligned with v0.9.0 release evidence. |

## v0.10.0 Tracking Table

`v0.10.0` adds Hetzner Firewall and Network request domains. The
provider-neutral facade follows the release tag, the Hetzner provider
publishes a code release, and optional boundary crates publish
metadata-aligned packages because the workspace still uses a shared package
version.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.9.0` | `0.10.0` | `metadata` | Yes | Release metadata, current Rust tooling, README updates, and v0.10.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.9.0` | `0.10.0` | `code` | Yes | No_std Firewall and Network request domains with canonical CIDR, route, subnet, and rule validation. |
| `cloud-sdk-hetzner-reqwest` | `0.9.0` | `0.10.0` | `metadata` | Yes | Keep optional transport boundary metadata aligned with v0.10.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.9.0` | `0.10.0` | `metadata` | Yes | Keep sanitization boundary metadata aligned with v0.10.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.9.0` | `0.10.0` | `metadata` | Yes | Keep testkit boundary metadata aligned with v0.10.0 release evidence. |

## v0.11.0 Tracking Table

`v0.11.0` adds Hetzner Load Balancer request domains and hardens shared JSON
writers, source-lock downloads, and release attestation. Release-sensitive
metadata is finalized before retest so the reviewed commit covers it.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.10.0` | `0.11.0` | `code` | Yes | Atomic no_std JSON string writes, release-attestation hardening, and v0.11.0 release evidence. |
| `cloud-sdk-hetzner` | `0.10.0` | `0.11.0` | `code` | Yes | No_std Load Balancer request domains plus atomic secret writers and source-lock download hardening. |
| `cloud-sdk-hetzner-reqwest` | `0.10.0` | `0.11.0` | `metadata` | Yes | Keep optional transport boundary metadata aligned with v0.11.0 release evidence. |
| `cloud-sdk-hetzner-sanitization` | `0.10.0` | `0.11.0` | `metadata` | Yes | Keep sanitization boundary metadata aligned with v0.11.0 release evidence. |
| `cloud-sdk-hetzner-testkit` | `0.10.0` | `0.11.0` | `metadata` | Yes | Keep testkit boundary metadata aligned with v0.11.0 release evidence. |

## v0.12.0 Tracking Table

`v0.12.0` adds Hetzner DNS Zone request domains. The provider-neutral facade
follows the release tag, the Hetzner provider publishes a code release, and
optional boundary crates publish metadata-aligned packages because the
workspace still uses a shared package version.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.11.0` | `0.12.0` | `metadata` | Yes | README updates and v0.12.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.11.0` | `0.12.0` | `code` | Yes | No_std DNS Zone CRUD, zonefile, nameserver, TTL, protection, and action request domains. |
| `cloud-sdk-hetzner-reqwest` | `0.11.0` | `0.11.0` | `unchanged` | No | Retired from the workspace before adoption and not part of the v0.12.0 publish plan. |
| `cloud-sdk-reqwest` | none | `0.12.0` | `code` | Yes | Initial provider-neutral no_std reqwest transport boundary for all cloud providers. |
| `cloud-sdk-hetzner-sanitization` | `0.11.0` | `0.11.0` | `unchanged` | No | Retired from the workspace before adoption and not part of the v0.12.0 publish plan. |
| `cloud-sdk-sanitization` | none | `0.12.0` | `code` | Yes | Initial provider-neutral no_std sanitization boundary for reusable cloud SDK secret helpers. |
| `cloud-sdk-hetzner-testkit` | `0.11.0` | `0.11.0` | `unchanged` | No | Retired from the workspace before adoption and not part of the v0.12.0 publish plan. |
| `cloud-sdk-testkit` | none | `0.12.0` | `code` | Yes | Initial provider-neutral no_std mock transport and fixture boundary for all cloud providers. |

## v0.13.0 Tracking Table

`v0.13.0` adds Hetzner DNS RRSet request domains. The facade follows the tag,
the Hetzner provider receives its next code minor, and the provider-neutral
boundary crates receive dependency-only patch releases for the facade's new
`0.13` line.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.12.0` | `0.13.0` | `metadata` | Yes | README updates and v0.13.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.12.0` | `0.13.0` | `code` | Yes | No_std DNS RRSet CRUD, record mutation, TTL, and protection request domains. |
| `cloud-sdk-reqwest` | `0.12.0` | `0.12.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.13 facade line. |
| `cloud-sdk-sanitization` | `0.12.0` | `0.12.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.13 facade line. |
| `cloud-sdk-testkit` | `0.12.0` | `0.12.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.13 facade line. |

## v0.14.0 Tracking Table

`v0.14.0` admits the optional Hetzner Serde boundary and provider-neutral
caller-buffer sanitization. The facade follows the tag, the provider and
sanitization boundary receive code minors, and the remaining neutral boundary
crates receive dependency-only patches for the facade's `0.14` line.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.13.0` | `0.14.0` | `metadata` | Yes | README updates and v0.14.0 release evidence for the provider-neutral facade. |
| `cloud-sdk-hetzner` | `0.13.0` | `0.14.0` | `code` | Yes | Optional no_std Serde boundary for RRSet bodies and validated shared response envelopes. |
| `cloud-sdk-reqwest` | `0.12.1` | `0.12.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.14 facade line. |
| `cloud-sdk-sanitization` | `0.12.1` | `0.13.0` | `code` | Yes | Admit reviewed volatile caller-buffer cleanup and an early-return-safe guard. |
| `cloud-sdk-testkit` | `0.12.1` | `0.12.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.14 facade line. |

## v0.15.0 Tracking Table

`v0.15.0` adds provider-neutral blocking transport contracts and the first
usable no_std testkit. The facade follows the tag, testkit receives a code
minor, and crates whose manifests only follow the new facade or dev boundary
receive dependency-only patches.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.14.0` | `0.15.0` | `code` | Yes | Provider-neutral blocking transport contracts for reusable mock and future real adapters. |
| `cloud-sdk-hetzner` | `0.14.0` | `0.15.0` | `code` | Yes | Integrate the provider-neutral adversarial corpus into Hetzner Serde tests. |
| `cloud-sdk-reqwest` | `0.12.2` | `0.12.3` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.15 facade line. |
| `cloud-sdk-sanitization` | `0.13.0` | `0.13.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.15 facade line. |
| `cloud-sdk-testkit` | `0.12.2` | `0.13.0` | `code` | Yes | Ordered no_std mock transport, response metadata fixtures, and adversarial response corpus. |

## v0.16.0 Tracking Table

`v0.16.0` admits the first provider-neutral blocking reqwest adapter behind an
explicit non-default rustls feature. The facade gains explicit content-type
metadata, reqwest receives a code minor, and crates whose manifests only
follow the facade receive dependency-only patches.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.15.0` | `0.16.0` | `code` | Yes | Explicit content-type support for the provider-neutral transport contract. |
| `cloud-sdk-hetzner` | `0.15.0` | `0.15.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.16 facade line. |
| `cloud-sdk-reqwest` | `0.12.3` | `0.13.0` | `code` | Yes | First blocking adapter with explicit rustls, timeout, redirect, retry, proxy, and redaction policy. |
| `cloud-sdk-sanitization` | `0.13.1` | `0.13.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.16 facade line. |
| `cloud-sdk-testkit` | `0.13.0` | `0.13.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.16 facade line. |

## v0.17.0 Tracking Table

`v0.17.0` adds a runtime-neutral async core contract and an optional hardened
async reqwest adapter. The facade, reqwest boundary, and testkit receive code
releases; crates whose manifests only follow the facade receive dependency
patches.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.16.0` | `0.17.0` | `code` | Yes | Runtime-neutral async transport contract for provider-neutral adapters and testkits. |
| `cloud-sdk-hetzner` | `0.15.1` | `0.15.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.17 facade line. |
| `cloud-sdk-reqwest` | `0.13.0` | `0.14.0` | `code` | Yes | Add the hardened provider-neutral async reqwest adapter. |
| `cloud-sdk-sanitization` | `0.13.2` | `0.13.3` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.17 facade line. |
| `cloud-sdk-testkit` | `0.13.1` | `0.14.0` | `code` | Yes | Implement the runtime-neutral async mock transport contract. |

## v0.18.0 Tracking Table

`v0.18.0` adds explicit provider-neutral pagination and action polling state,
strict transport rate-limit metadata, and the first Hetzner shared pagination
response parser. Every crate whose code implements the boundary receives an
independent minor; sanitization follows the facade dependency with a patch.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.17.0` | `0.18.0` | `code` | Yes | Provider-neutral pagination, action polling, and rate-limit metadata contracts. |
| `cloud-sdk-hetzner` | `0.15.2` | `0.16.0` | `code` | Yes | Strict Hetzner pagination metadata parsing and action polling conversion. |
| `cloud-sdk-reqwest` | `0.14.0` | `0.15.0` | `code` | Yes | Validate and propagate provider rate-limit response headers. |
| `cloud-sdk-sanitization` | `0.13.3` | `0.13.4` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.18 facade line. |
| `cloud-sdk-testkit` | `0.14.0` | `0.15.0` | `code` | Yes | Propagate validated rate-limit metadata through deterministic fixtures. |

## v0.19.0 Tracking Table

`v0.19.0` adds an opt-in, read-only Hetzner catalog smoke harness without
placing filesystem, HTTP, TLS, or runtime dependencies in any provider default
graph. The provider test code receives an independent minor; provider-neutral
boundaries follow the facade dependency with patch releases.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.18.0` | `0.19.0` | `metadata` | Yes | Live smoke documentation, release evidence, and v0.19 facade metadata. |
| `cloud-sdk-hetzner` | `0.16.0` | `0.17.0` | `code` | Yes | Opt-in read-only Hetzner catalog smoke harness and adversarial credential-file tests. |
| `cloud-sdk-reqwest` | `0.15.0` | `0.15.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.19 facade line. |
| `cloud-sdk-sanitization` | `0.13.4` | `0.13.5` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.19 facade line. |
| `cloud-sdk-testkit` | `0.15.0` | `0.15.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.19 facade line. |

## v0.20.0 Tracking Table

`v0.20.0` adds cross-target no_std/alloc compile evidence, native desktop
transport checks, and a default dependency-graph boundary. No crate API code
changes; the facade carries release metadata and dependents patch-bump their
facade requirement.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.19.0` | `0.20.0` | `metadata` | Yes | Platform support documentation, release evidence, and v0.20 facade metadata. |
| `cloud-sdk-hetzner` | `0.17.0` | `0.17.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.20 facade line. |
| `cloud-sdk-reqwest` | `0.15.1` | `0.15.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.20 facade line. |
| `cloud-sdk-sanitization` | `0.13.5` | `0.13.6` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.20 facade line. |
| `cloud-sdk-testkit` | `0.15.1` | `0.15.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.20 facade line. |

## v0.21.0 Tracking Table

`v0.21.0` adds compile-checked SDK workflows, complete crate feature
documentation, security recipes, a release runbook, doctest enforcement, and
tested local-link validation. The facade follows the tag, the Hetzner provider
receives an independent code minor for its shipped examples, and the neutral
boundary crates patch-bump for the facade dependency.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.20.0` | `0.21.0` | `code` | Yes | Compile-checked quickstart, documentation gates, and v0.21 release evidence. |
| `cloud-sdk-hetzner` | `0.17.1` | `0.18.0` | `code` | Yes | Compile-checked read-only, mutation, pagination, action, DNS, and Storage Box examples. |
| `cloud-sdk-reqwest` | `0.15.2` | `0.15.3` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.21 facade line. |
| `cloud-sdk-sanitization` | `0.13.6` | `0.13.7` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.21 facade line. |
| `cloud-sdk-testkit` | `0.15.2` | `0.15.3` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.21 facade line. |

## v0.22.0 Tracking Table

`v0.22.0` adds an isolated fuzz package, six libFuzzer targets, source-derived
seed inputs, deterministic adversarial regressions, and separately audited
fuzz supply-chain evidence. The facade follows the tag, the Hetzner provider
receives an independent code minor for response-boundary tests, and the neutral
boundary crates patch-bump for the facade dependency.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.21.0` | `0.22.0` | `code` | Yes | Isolated fuzz harness, deterministic adversarial tests, and v0.22 release evidence. |
| `cloud-sdk-hetzner` | `0.18.0` | `0.19.0` | `code` | Yes | Adversarial Serde response tests for malformed and oversized upstream inputs. |
| `cloud-sdk-reqwest` | `0.15.3` | `0.15.4` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.22 facade line. |
| `cloud-sdk-sanitization` | `0.13.7` | `0.13.8` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.22 facade line. |
| `cloud-sdk-testkit` | `0.15.3` | `0.15.4` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.22 facade line. |

## v0.23.0 Tracking Table

`v0.23.0` adds a fail-closed optional blocking FIPS-mode transport, explicit
provider and complete-client runtime checks, bundled-source build evidence,
and a documented current validation-status limitation. The facade follows the
tag, reqwest receives an independent code minor, and crates following the
facade dependency receive patch releases.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.22.0` | `0.23.0` | `metadata` | Yes | Optional blocking FIPS transport documentation and v0.23 release evidence. |
| `cloud-sdk-hetzner` | `0.19.0` | `0.19.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.23 facade line. |
| `cloud-sdk-reqwest` | `0.15.4` | `0.16.0` | `code` | Yes | Explicitly selected and runtime-verified blocking rustls FIPS-mode transport. |
| `cloud-sdk-sanitization` | `0.13.8` | `0.13.9` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.23 facade line. |
| `cloud-sdk-testkit` | `0.15.4` | `0.15.5` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.23 facade line. |

## v0.24.0 Tracking Table

`v0.24.0` adds an optional deterministic Mozilla-root blocking transport and
refreshes dependency, tool, native-build, audit, and SBOM evidence. The facade
follows the tag, reqwest receives an independent code minor, and crates
following the facade dependency receive patch releases.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.23.0` | `0.24.0` | `metadata` | Yes | Dependency, tooling, deterministic-root documentation, and v0.24 release evidence. |
| `cloud-sdk-hetzner` | `0.19.1` | `0.19.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.24 facade line. |
| `cloud-sdk-reqwest` | `0.16.0` | `0.17.0` | `code` | Yes | Deterministic Mozilla-root transport and hardened dependency/tooling evidence. |
| `cloud-sdk-sanitization` | `0.13.9` | `0.13.10` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.24 facade line. |
| `cloud-sdk-testkit` | `0.15.5` | `0.15.6` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.24 facade line. |

## v0.25.0 Tracking Table

`v0.25.0` turns source-spec drift detection into an actionable maintenance
process with grouped reports, checked-in fixture specifications, a read-only
scheduled check, and explicit source-lock decision documentation. Published
crate APIs are unchanged; the facade follows the tag and dependents receive
patch-only manifest updates.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.24.0` | `0.25.0` | `metadata` | Yes | API drift maintenance automation, documentation, and v0.25 release evidence. |
| `cloud-sdk-hetzner` | `0.19.2` | `0.19.3` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.25 facade line. |
| `cloud-sdk-reqwest` | `0.17.0` | `0.17.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.25 facade line. |
| `cloud-sdk-sanitization` | `0.13.10` | `0.13.11` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.25 facade line. |
| `cloud-sdk-testkit` | `0.15.6` | `0.15.7` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.25 facade line. |

## v0.26.0 Tracking Table

`v0.26.0` implements the five remaining non-deprecated action request
operations and adds a fail-closed matrix gate for complete source-locked
coverage. The facade follows the tag, the Hetzner provider receives an
independent code minor, and neutral crates patch-bump for the facade
dependency.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.25.0` | `0.26.0` | `metadata` | Yes | Complete non-deprecated Hetzner endpoint coverage and v0.26 release evidence. |
| `cloud-sdk-hetzner` | `0.19.3` | `0.20.0` | `code` | Yes | Add the five remaining non-deprecated action query operations. |
| `cloud-sdk-reqwest` | `0.17.1` | `0.17.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.26 facade line. |
| `cloud-sdk-sanitization` | `0.13.11` | `0.13.12` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.26 facade line. |
| `cloud-sdk-testkit` | `0.15.7` | `0.15.8` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.26 facade line. |

## v0.27.0 Tracking Table

`v0.27.0` stabilizes existing public APIs before Robot implementation. Required
request fields become direct arguments, public errors gain payload-free standard
traits, custom credential destinations become explicit, and capability claims
are checked against the reviewed documentation contract.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.26.0` | `0.27.0` | `code` | Yes | Add payload-free public error traits and v0.27 stabilization evidence. |
| `cloud-sdk-hetzner` | `0.20.0` | `0.21.0` | `code` | Yes | Make required request inputs type-safe and stabilize public errors. |
| `cloud-sdk-reqwest` | `0.17.2` | `0.18.0` | `code` | Yes | Make custom credential destinations explicit and stabilize public errors. |
| `cloud-sdk-sanitization` | `0.13.12` | `0.13.13` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.27 facade line. |
| `cloud-sdk-testkit` | `0.15.8` | `0.16.0` | `code` | Yes | Add payload-free public error traits and update the facade dependency. |

## v0.28.0 Tracking Table

`v0.28.0` moves blocking and async sends to shared references, adds immutable
endpoint identity, and gives the optional reqwest clients a reviewed concurrent
credential lifecycle without changing any default dependency graph.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.27.0` | `0.28.0` | `code` | Yes | Add shared-reference transport and immutable endpoint identity contracts. |
| `cloud-sdk-hetzner` | `0.21.0` | `0.22.0` | `code` | Yes | Add exact official Cloud and Storage endpoint verification. |
| `cloud-sdk-reqwest` | `0.18.0` | `0.19.0` | `code` | Yes | Add shareable clients, endpoint identity, mutable token ingestion, and atomic rotation. |
| `cloud-sdk-sanitization` | `0.13.13` | `0.13.14` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.28 facade line. |
| `cloud-sdk-testkit` | `0.16.0` | `0.17.0` | `code` | Yes | Adapt the ordered mock to shared-reference blocking and async transports. |

## v0.29.0 Tracking Table

`v0.29.0` adds allocation-free operation preparation, explicit safety/retry
metadata, immutable service endpoint binding, checked response policy, response
content-type capture, and prepared-request testkit evidence.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.28.0` | `0.29.0` | `code` | Yes | Add prepared request metadata, bounded execution, and checked response policy. |
| `cloud-sdk-hetzner` | `0.22.0` | `0.22.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.29 facade line. |
| `cloud-sdk-reqwest` | `0.19.0` | `0.20.0` | `code` | Yes | Capture validated response content types and reject malformed or duplicate values. |
| `cloud-sdk-sanitization` | `0.13.14` | `0.13.15` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.29 facade line. |
| `cloud-sdk-testkit` | `0.17.0` | `0.18.0` | `code` | Yes | Add prepared-request records, bound endpoints, and response content-type fixtures. |

## v0.30.0 Tracking Table

`v0.30.0` completes allocation-free prepared requests for all 208 active
Hetzner Cloud, DNS, and Console Storage operations and source-locks all 91
operations with request bodies.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.29.0` | `0.30.0` | `metadata` | Yes | Publish the v0.30 facade metadata and Hetzner preparation documentation. |
| `cloud-sdk-hetzner` | `0.22.1` | `0.23.0` | `code` | Yes | Prepare every active Hetzner operation with bounded target, body, metadata, and response policy. |
| `cloud-sdk-reqwest` | `0.20.0` | `0.20.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.30 facade line. |
| `cloud-sdk-sanitization` | `0.13.15` | `0.13.16` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.30 facade line. |
| `cloud-sdk-testkit` | `0.18.0` | `0.18.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.30 facade line. |

## v0.31.0 Tracking Table

`v0.31.0` binds every prepared operation to its source-locked success schema
and adds checked typed success and API error decoding behind the optional
`serde`/`alloc` boundary.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.30.0` | `0.31.0` | `code` | Yes | Bind validated provider operation identifiers and expose prepared response-policy validation. |
| `cloud-sdk-hetzner` | `0.23.0` | `0.24.0` | `code` | Yes | Decode all 208 active operations through source-locked success families and typed API errors. |
| `cloud-sdk-reqwest` | `0.20.1` | `0.20.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.31 facade line. |
| `cloud-sdk-sanitization` | `0.13.16` | `0.14.0` | `code` | Yes | Add alloc-backed owned UTF-8 secret storage with volatile cleanup for checked provider responses. |
| `cloud-sdk-testkit` | `0.18.1` | `0.18.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.31 facade line. |

## v0.32.0 Tracking Table

`v0.32.0` replaces closed core provider and API-family enums with bounded
canonical IDs and provider-owned marker traits. Hetzner owns its service
identities; an external integration fixture proves another provider needs no
central registry edit.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.31.0` | `0.32.0` | `code` | Yes | Add bounded provider/service IDs, open ownership markers, and marker-derived prepared-service binding. |
| `cloud-sdk-hetzner` | `0.24.0` | `0.25.0` | `code` | Yes | Own Hetzner Cloud, DNS, security, and Console Storage identities and migrate preparation and decoding. |
| `cloud-sdk-reqwest` | `0.20.2` | `0.20.3` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.32 facade line. |
| `cloud-sdk-sanitization` | `0.14.0` | `0.15.0` | `code` | Yes | Adapt the stable wrapper API to sanitization 2.0.3 and update the cloud-sdk dependency. |
| `cloud-sdk-testkit` | `0.18.2` | `0.18.3` | `dependency` | Yes | Update the `cloud-sdk` dependency and identity assertions to the v0.32 facade line. |

## v0.33.0 Tracking Table

`v0.33.0` replaces the closed four-variant HTTP method enum with a bounded,
allocation-free method value. Known methods have canonical constants,
provider extensions are validated, and operation safety remains provider-owned.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.32.0` | `0.33.0` | `code` | Yes | Add GET, POST, PUT, DELETE, PATCH, HEAD, origin-form OPTIONS, and bounded canonical extension methods while denying CONNECT and TRACE. |
| `cloud-sdk-hetzner` | `0.25.0` | `0.26.0` | `code` | Yes | Replace method-derived impact, semantics, and retry inference with explicit provider-owned operation classes. |
| `cloud-sdk-reqwest` | `0.20.3` | `0.21.0` | `code` | Yes | Map every admitted known and extension method through blocking and async adapters with exact wire tests. |
| `cloud-sdk-sanitization` | `0.15.0` | `0.15.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.33 facade line. |
| `cloud-sdk-testkit` | `0.18.3` | `0.18.4` | `dependency` | Yes | Update the `cloud-sdk` dependency and prove exact extension-method matching. |

## v0.34.0 Tracking Table

`v0.34.0` replaces one static expected endpoint with provider-owned endpoint
policy algebra. Authority input is canonical and custom credential
destinations require an explicit trusted-operator acknowledgement.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.33.0` | `0.34.0` | `code` | Yes | Add fixed, bounded official-set, region-derived, and acknowledged-custom endpoint policies with non-static identities. |
| `cloud-sdk-hetzner` | `0.26.0` | `0.27.0` | `code` | Yes | Own exact fixed Cloud/Storage policies and a bounded provider-wide official endpoint set. |
| `cloud-sdk-reqwest` | `0.21.0` | `0.22.0` | `code` | Yes | Require provider policy or explicit custom acknowledgement and reject ambiguous raw authorities before URL normalization. |
| `cloud-sdk-sanitization` | `0.15.1` | `0.15.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.34 facade line. |
| `cloud-sdk-testkit` | `0.18.4` | `0.19.0` | `code` | Yes | Preserve non-static endpoint policy lifetimes in prepared-request records and policy matching tests. |

## v0.35.0 Tracking Table

`v0.35.0` makes core the single request-target validation authority and
separates canonical paths from explicit absent, canonical, and form query
states without allocation.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.34.0` | `0.35.0` | `code` | Yes | Add canonical path/query types, exact query views, pair iteration, and transactional assembly. |
| `cloud-sdk-hetzner` | `0.27.0` | `0.28.0` | `code` | Yes | Align provider paths and prepared query evidence with the canonical core contract. |
| `cloud-sdk-reqwest` | `0.22.0` | `0.23.0` | `code` | Yes | Remove divergent target validation and preserve exact core-admitted bytes. |
| `cloud-sdk-sanitization` | `0.15.2` | `0.15.3` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.35 facade line. |
| `cloud-sdk-testkit` | `0.19.0` | `0.20.0` | `code` | Yes | Match exact query presence and encoding dialect in deterministic mocks. |

## v0.36.0 Tracking Table

`v0.36.0` adds bounded ordered request headers, fixed-capacity response
metadata, explicit provider header policy, and exact adapter/testkit
conformance.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.35.0` | `0.36.0` | `code` | Yes | Add bounded names, values, blocks, sensitivity, atomic encoding, and response metadata. |
| `cloud-sdk-hetzner` | `0.28.0` | `0.29.0` | `code` | Yes | Put JSON Accept and Content-Type policy into prepared requests. |
| `cloud-sdk-reqwest` | `0.23.0` | `0.24.0` | `code` | Yes | Forward exact request headers and capture bounded response metadata. |
| `cloud-sdk-sanitization` | `0.15.3` | `0.15.4` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.36 facade line. |
| `cloud-sdk-testkit` | `0.20.0` | `0.21.0` | `code` | Yes | Match exact request headers and model bounded response metadata. |

## v0.37.0 Tracking Table

`v0.37.0` makes response provenance structural through a cleanup-owning
caller buffer, sealed transport writer, core-only response views, and
guard-scoped checked decoding.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.36.0` | `0.37.0` | `code` | Yes | Add sealed response admission, commitment, cleanup ownership, and checked guard lifetimes. |
| `cloud-sdk-hetzner` | `0.29.0` | `0.30.0` | `code` | Yes | Consume cleanup-owning response buffers in all checked success and provider-error decoding. |
| `cloud-sdk-reqwest` | `0.24.0` | `0.25.0` | `code` | Yes | Commit blocking and async responses only through the admitted sealed writer. |
| `cloud-sdk-sanitization` | `0.15.4` | `0.15.5` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.37 facade line. |
| `cloud-sdk-testkit` | `0.21.0` | `0.22.0` | `code` | Yes | Produce deterministic fixtures through the same sealed writer and commitment contract. |

## v0.38.0 Tracking Table

`v0.38.0` makes response cleanup mandatory through one audited
provider-neutral primitive, owns the complete checked decode workspace, and
adds explicit request-identifier retention policy.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk-sanitization` | `0.15.5` | `0.16.0` | `code` | Yes | Invert the core dependency and expose byte and scalar cleanup through the admitted primitive. |
| `cloud-sdk` | `0.37.0` | `0.38.0` | `code` | Yes | Mandate full response/workspace cleanup and add protected retained metadata. |
| `cloud-sdk-hetzner` | `0.30.0` | `0.31.0` | `code` | Yes | Decode with guard-owned scratch and protected request identifiers. |
| `cloud-sdk-reqwest` | `0.25.0` | `0.26.0` | `code` | Yes | Transfer non-Copy metadata into core cleanup owners and retain additive hooks. |
| `cloud-sdk-testkit` | `0.22.0` | `0.23.0` | `code` | Yes | Exercise non-Copy fixtures and mandatory cleanup paths. |

## v0.39.0 Tracking Table

`v0.39.0` makes fixed-buffer request encoding transactional and adds bounded,
cleanup-owning request preparation storage profiles.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.38.0` | `0.39.0` | `code` | Yes | Add immutable snapshot encoding, exact replay verification, cleanup guards, and named capacity profiles. |
| `cloud-sdk-hetzner` | `0.31.0` | `0.32.0` | `code` | Yes | Migrate complete paths, queries, and JSON bodies to transactional encoding. |
| `cloud-sdk-reqwest` | `0.26.0` | `0.26.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.39 facade line. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.23.0` | `0.23.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.39 facade line. |

## v0.40.0 Tracking Table

`v0.40.0` adds credential-free bounded raw HTTP execution, conservative
delivery phases, and deterministic send-phase fault injection.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.39.0` | `0.40.0` | `code` | Yes | Add delivery-phased raw executor traits and bounded response-wire policy. |
| `cloud-sdk-hetzner` | `0.32.0` | `0.32.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.40 facade line. |
| `cloud-sdk-reqwest` | `0.26.1` | `0.27.0` | `code` | Yes | Add one shared bounded Hyper engine for raw blocking, async, deterministic-root, and FIPS execution. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.23.1` | `0.24.0` | `code` | Yes | Add deterministic raw delivery-phase fault injection. |

## v0.41.0 Tracking Table

`v0.41.0` adds mandatory provider or operation-owned bearer scope policy,
generation-safe rotation and refresh, and cleanup-owned adapter headers.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.40.0` | `0.41.0` | `code` | Yes | Add provider-neutral authentication scope, generation, refresh-handoff, and authenticated transport contracts. |
| `cloud-sdk-hetzner` | `0.32.1` | `0.32.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.41 facade line; live smoke adopts mandatory scope policy only in tests. |
| `cloud-sdk-reqwest` | `0.27.0` | `0.28.0` | `code` | Yes | Add immutable bearer scope, policy-required sends, atomic rotation, CAS refresh, and cleanup-owned authorization values. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.24.0` | `0.24.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.41 facade line. |

## v0.42.0 Tracking Table

`v0.42.0` adds type-separated Basic authentication transports, bounded
canonical signing inputs, and a narrow credential-free Robot wire source lock.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.41.0` | `0.42.0` | `code` | Yes | Add context-bound v2 canonical signing with tagged canonical hosts and distinct digest/signature algorithms, internally coupled body hashing, retained requests, and validated signed output. |
| `cloud-sdk-hetzner` | `0.32.2` | `0.32.3` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.42 facade line; the Robot fixture remains excluded repository evidence. |
| `cloud-sdk-reqwest` | `0.28.0` | `0.29.0` | `code` | Yes | Add bounded Basic credentials and scoped blocking/async authenticated clients. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.24.1` | `0.24.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.42 facade line. |

## v0.43.0 Tracking Table

`v0.43.0` removes the compatibility gap between provider preparation,
authentication, and the bounded raw HTTP executor for all active Hetzner
operations.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.42.0` | `0.43.0` | `code` | Yes | Require complete authentication-scope and raw-response policies in every prepared request and authenticated execution. |
| `cloud-sdk-hetzner` | `0.32.3` | `0.33.0` | `code` | Yes | Bind all 208 active operations to exact service, official endpoint, authentication, and raw response policy with one zero-fallback assembly path. |
| `cloud-sdk-reqwest` | `0.29.0` | `0.30.0` | `code` | Yes | Route bearer and Basic clients through the bounded raw Hyper engine while preserving conservative delivery phase. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.24.2` | `0.25.0` | `code` | Yes | Execute authenticated prepared requests and expose complete authentication and raw-response policy records. |

## v0.44.0 Tracking Table

`v0.44.0` separates provider-neutral pagination strategies and prevents raw
provider links from entering structured request-target assembly.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.43.0` | `0.44.0` | `code` | Yes | Add bounded numbered, offset, cursor, marker, and operation-bound raw-link pagination strategies. |
| `cloud-sdk-hetzner` | `0.33.0` | `0.34.0` | `code` | Yes | Migrate Hetzner pagination metadata to the numbered strategy contract. |
| `cloud-sdk-reqwest` | `0.30.0` | `0.30.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.44 facade line. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.25.0` | `0.25.1` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.44 facade line. |

## v0.45.0 Tracking Table

`v0.45.0` moves quota interpretation into provider crates and adds bounded
multi-bucket state plus pure caller-controlled delay decisions.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.44.0` | `0.45.0` | `code` | Yes | Add bounded quota buckets, exact Retry-After parsing, and pure maximum-bound delay policy. |
| `cloud-sdk-hetzner` | `0.34.0` | `0.35.0` | `code` | Yes | Decode complete Hetzner quota metadata and retain it on checked success and provider-error models. |
| `cloud-sdk-reqwest` | `0.30.1` | `0.31.0` | `code` | Yes | Remove the obsolete provider-specific parser and transport error variant. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.25.1` | `0.25.2` | `dependency` | Yes | Update the `cloud-sdk` dependency to the v0.45 facade line. |

## v0.46.0 Tracking Table

`v0.46.0` binds exact replay identity to provider-owned operation metadata and
adds one bounded retry owner with fresh local mutation intent.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.45.0` | `0.46.0` | `code` | Yes | Add canonical request fingerprints, fresh intent binding, monotonic budgets, and single-owner retry decisions. |
| `cloud-sdk-hetzner` | `0.35.0` | `0.36.0` | `code` | Yes | Mark transactionally prepared immutable target/body snapshots as byte-for-byte replayable. |
| `cloud-sdk-reqwest` | `0.31.0` | `0.31.1` | `dependency` | Yes | Update `cloud-sdk` and the reviewed transitive `ipnet` patch without introducing transport-owned retry. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.25.2` | `0.26.0` | `code` | Yes | Record explicit body replayability in redacted prepared-request policy evidence. |

## v0.47.0 Tracking Table

`v0.47.0` adds local asynchronous contracts for `!Send` browser-WASM,
embedded, and single-threaded executors and gives every async transport the
same non-committing response boundary.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.46.0` | `0.47.0` | `code` | Yes | Add local basic, authenticated, raw, prepared, pagination, and retry execution contracts with explicit cancellation policy. |
| `cloud-sdk-hetzner` | `0.36.0` | `0.36.1` | `dependency` | Yes | Update the provider's `cloud-sdk` dependency without changing Hetzner behavior. |
| `cloud-sdk-reqwest` | `0.31.1` | `0.32.0` | `code` | Yes | Migrate Send async transports to non-committing staging and SDK-owned response commitment; retain automatic local compatibility. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.26.0` | `0.27.0` | `code` | Yes | Add a no-allocation deliberately local-only mock and local async conformance fixtures. |

## v0.48.0 Tracking Table

`v0.48.0` adds bounded streaming policy, actual-byte accounting, deterministic
backpressure, replay identity, cancellation cleanup, and blocking, Send-async,
and local-async drivers.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.47.0` | `0.48.0` | `code` | Yes | Add bounded streaming policy, accounting, replay, cleanup, and drivers for all execution modes. |
| `cloud-sdk-hetzner` | `0.36.1` | `0.36.2` | `dependency` | Yes | Update the provider's `cloud-sdk` dependency without changing Hetzner behavior. |
| `cloud-sdk-reqwest` | `0.32.0` | `0.32.1` | `dependency` | Yes | Update the core dependency without claiming a reqwest streaming adapter. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.16.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.27.0` | `0.28.0` | `code` | Yes | Add deterministic borrowed stream sources and caller-buffered short-write sinks. |

## v0.49.0 Tracking Table

`v0.49.0` adds provider-owned bounded incremental JSON decoding without
changing the buffered checked decoder or default dependency graph.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.48.0` | `0.49.0` | `metadata` | Yes | Publish facade documentation and align the provider release. |
| `cloud-sdk-hetzner` | `0.36.2` | `0.37.0` | `code` | Yes | Add bounded incremental visitor decoding across arbitrary JSON chunks. |
| `cloud-sdk-reqwest` | `0.32.1` | `0.32.2` | `dependency` | Yes | Update the core dependency without changing adapter behavior. |
| `cloud-sdk-sanitization` | `0.16.0` | `0.17.0` | `code` | Yes | Add bounded fallible growth for protected parser staging. |
| `cloud-sdk-testkit` | `0.28.0` | `0.28.1` | `dependency` | Yes | Update the core dependency without changing testkit behavior. |

## v0.50.0 Tracking Table

`v0.50.0` adds exhaustive compile-time associations for every active Hetzner
operation without changing the default dependency or feature graph.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.49.0` | `0.50.0` | `code` | Yes | Add const operation IDs and compile-time validated operation literals. |
| `cloud-sdk-hetzner` | `0.37.0` | `0.38.0` | `code` | Yes | Add all 208 sealed operation markers, typed components, prepared requests, and source-lock generation. |
| `cloud-sdk-reqwest` | `0.32.2` | `0.32.3` | `dependency` | Yes | Update the core dependency without changing adapter behavior. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.17.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.28.1` | `0.28.2` | `dependency` | Yes | Update the core dependency without changing testkit behavior. |

## v0.51.0 Tracking Table

`v0.51.0` is the first internal milestone accumulated after the public
v0.50 baseline. It adds plan-confirm execution authority without publishing a
crate; all changed package trees remain part of the v0.55 cumulative release.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.51.0` | `code` | No | Add exact plan confirmation, direct/shared permits, delivery classification, and state-changing execution enforcement. |
| `cloud-sdk-hetzner` | `0.38.0` | `0.38.0` | `code` | No | Gate typed state-changing execution on the provider-neutral permit lifecycle. |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.3` | `dependency` | No | Accumulate the source core dependency for v0.55 without publication. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.17.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.28.2` | `0.28.2` | `code` | No | Align transport-policy fixtures with read-only direct execution. |

## v0.52.0 Tracking Table

`v0.52.0` is the second internal milestone accumulated after the public v0.50
baseline. It adds the provider-generic client kernel without publishing a
crate; all changed package trees remain part of the v0.55 cumulative release.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.52.0` | `code` | No | Add typed blocking/Send-async/local-async client execution and bounded caller-owned workspace leases. |
| `cloud-sdk-hetzner` | `0.38.0` | `0.38.0` | `code` | No | Retain cumulative v0.51 typed permit integration for v0.55. |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.3` | `dependency` | No | Accumulate the source core dependency for v0.55 without publication. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.17.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.28.2` | `0.28.2` | `code` | No | Retain cumulative v0.51 fixture changes for v0.55. |

## v0.53.0 Tracking Table

`v0.53.0` is the third internal milestone accumulated after the public v0.50
baseline. It adds bounded provider-neutral workflow drivers without publishing
a crate; all changed package trees remain part of the v0.55 cumulative release.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.53.0` | `code` | No | Add sequenced pager and bounded action polling with separate cancellation, backoff, progress, and time policy. |
| `cloud-sdk-hetzner` | `0.38.0` | `0.38.0` | `code` | No | Retain cumulative typed execution changes for v0.55. |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.3` | `dependency` | No | Accumulate the source core dependency for v0.55 without publication. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.17.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.28.2` | `0.28.2` | `code` | No | Retain cumulative v0.51 fixture changes for v0.55. |

## v0.54.0 Tracking Table

`v0.54.0` is the fourth internal milestone accumulated after the public v0.50
baseline. It adds provider-neutral payload-free diagnostics without publishing
a crate; all changed package trees remain part of the v0.55 cumulative release.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.54.0` | `code` | No | Add finite diagnostic categories and opt-in lifecycle observers across all client modes. |
| `cloud-sdk-hetzner` | `0.38.0` | `0.38.0` | `code` | No | Retain cumulative typed execution changes for v0.55. |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.3` | `dependency` | No | Accumulate the source core dependency for v0.55 without publication. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.17.0` | `unchanged` | No | No code, dependency, or package metadata changes. |
| `cloud-sdk-testkit` | `0.28.2` | `0.28.2` | `code` | No | Retain cumulative v0.51 fixture changes for v0.55. |

## v0.55.0 Tracking Table

`v0.55.0` is the cumulative public checkpoint after the published v0.50
baseline. Independent versions cover every package-tree change accumulated in
v0.51 through v0.55, including the final cross-workspace test-assurance
remediation.

| Crate | Previous | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.50.0` | `0.55.0` | `code` | Yes | Publish cumulative permits, client execution, workflow drivers, and diagnostics. |
| `cloud-sdk-hetzner` | `0.38.0` | `0.39.0` | `code` | Yes | Publish cumulative typed permit and direct-execution integration. |
| `cloud-sdk-reqwest` | `0.32.3` | `0.32.4` | `dependency` | Yes | Update the core dependency without changing adapter behavior. |
| `cloud-sdk-sanitization` | `0.17.0` | `0.18.0` | `code` | Yes | Make cleanup fixtures fail closed under the workspace-wide security test policy. |
| `cloud-sdk-testkit` | `0.28.2` | `0.29.0` | `code` | Yes | Publish cumulative policy fixtures plus bounded dynamic responders, records, scripts, and stream faults. |

## v0.56.0 Tracking Table

`v0.56.0` adds provider-neutral drift evidence and restores incremental
pentesting for every tag. It is an internal milestone, so all supporting crates
retain their published versions and no package is selected for crates.io.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.56.0` | `metadata` | No | Record provider-generic drift tooling and per-tag pentest release metadata. |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.0` | `unchanged` | No | No provider crate source or metadata changes. |
| `cloud-sdk-reqwest` | `0.32.4` | `0.32.4` | `unchanged` | No | No transport adapter source, dependency, or metadata changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.0` | `unchanged` | No | No testkit source or metadata changes. |

## v0.57.0 Tracking Table

`v0.57.0` source-locks an excluded OVHcloud API v2 architecture probe. The
probe is evidence for later neutral-contract validation, not a supported
provider crate, and no workspace package is selected for crates.io.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.57.0` | `metadata` | No | Record the source-locked unpublished OVHcloud probe and release evidence. |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.0` | `unchanged` | No | No provider crate source or metadata changes. |
| `cloud-sdk-reqwest` | `0.32.4` | `0.32.4` | `unchanged` | No | No transport adapter source, dependency, or metadata changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.0` | `unchanged` | No | No testkit source or metadata changes. |

## v0.58.0 Tracking Table

`v0.58.0` adds exact regional API/token authority pairs and explicit-time
expiring bearer lifecycles. It remains an internal milestone, so cumulative
package changes retain their published version numbers until v0.60.0. Its
incremental pentest and final retest passed.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.58.0` | `code` | No | Add bounded endpoint-pair and credential-lifetime contracts. |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.0` | `dependency` | No | Accumulate the source core dependency for v0.60.0. |
| `cloud-sdk-reqwest` | `0.32.4` | `0.32.4` | `code` | No | Add expiry-qualified atomic bearer rotation and refresh. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.0` | `dependency` | No | Accumulate the source core dependency for v0.60.0. |

## v0.59.0 Tracking Table

`v0.59.0` adds prepared-request-bound sensitive header-cursor execution and
reviewed schema-version validation. It remains an internal milestone, so
cumulative package changes retain their published version numbers until
v0.60.0.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.59.0` | `code` | No | Add prepared-bound header-cursor and reviewed schema-version contracts. |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.0` | `dependency` | No | Accumulate the source core dependency for v0.60.0. |
| `cloud-sdk-reqwest` | `0.32.4` | `0.32.4` | `dependency` | No | Accumulate the source core dependency for v0.60.0. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.0` | `dependency` | No | Accumulate the source core dependency for v0.60.0. |

## v0.60.0 Tracking Table

`v0.60.0` is the cumulative public checkpoint after v0.55. It publishes every
package-tree change from v0.56 through v0.60, including bounded asynchronous
resources and source-locked production OVHcloud task conformance. The excluded
probe remains unpublished.

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.55.0` | `0.60.0` | `code` | Yes | Publish cumulative drift, authority, cursor, schema, and asynchronous-resource contracts. |
| `cloud-sdk-hetzner` | `0.39.0` | `0.39.1` | `dependency` | Yes | Update the core dependency for the public checkpoint. |
| `cloud-sdk-reqwest` | `0.32.4` | `0.33.0` | `code` | Yes | Publish expiry-qualified atomic bearer rotation and refresh. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.29.0` | `0.29.1` | `dependency` | Yes | Update the core dependency for the public checkpoint. |

## v0.61.0 Tracking Table

`v0.61.0` adds the nonpublishable OVHcloud end-to-end execution harness. It
remains an internal milestone, so no package is selected for crates.io and
supporting crates retain their published versions until v0.65.0.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.61.0` | `metadata` | No | Record the end-to-end probe harness and release evidence. |
| `cloud-sdk-hetzner` | `0.39.1` | `0.39.1` | `dependency` | No | Accumulate the source core dependency for v0.65.0. |
| `cloud-sdk-reqwest` | `0.33.0` | `0.33.0` | `code` | No | Accumulate the exact `base64-ng 2.0.1` Basic-auth encoder update for v0.65.0. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.29.1` | `0.29.1` | `code` | No | Retain the cumulative v0.62 fixture changes for v0.65.0. |

## v0.64.0 Tracking Table

`v0.64.0` completes Cloud action, metrics, composite, exact numeric, and
canonical timestamp responses. It remains internal, so no package is selected
for crates.io and cumulative provider/adapter/testkit changes remain deferred
to v0.65.0.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.64.0` | `metadata` | No | Record the Cloud special-model milestone and exact release evidence. |
| `cloud-sdk-hetzner` | `0.39.1` | `0.39.1` | `code` | No | Accumulate complete action, metrics, composite, exact scalar, and protected-error models for v0.65.0. |
| `cloud-sdk-reqwest` | `0.33.0` | `0.33.0` | `code` | No | Retain the cumulative exact `base64-ng 2.0.1` update for v0.65.0. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Existing protected owned strings satisfy nullable secret-output handling. |
| `cloud-sdk-testkit` | `0.29.1` | `0.29.1` | `code` | No | Retain cumulative neutral fixture changes for v0.65.0. |

## v0.62.0 Tracking Table

`v0.62.0` freezes the neutral contracts after materially different probes and
source-complete primary-provider slices. It remains internal; supporting crate
versions accumulate code until the v0.65.0 public checkpoint.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.62.0` | `code` | No | Record the reviewed neutral API freeze and exact milestone evidence. |
| `cloud-sdk-hetzner` | `0.39.1` | `0.39.1` | `code` | No | Accumulate source-complete locations, certificate, and Storage Box slices plus exact DNS/security response identities. |
| `cloud-sdk-reqwest` | `0.33.0` | `0.33.0` | `dependency` | No | Accumulate the source core dependency for v0.65.0. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Existing protected owned strings satisfy the secret-output boundary. |
| `cloud-sdk-testkit` | `0.29.1` | `0.29.1` | `code` | No | Accumulate exact successful statuses and delivery-classified permit fixtures. |

## v0.63.0 Tracking Table

`v0.63.0` completes source-derived ordinary Hetzner Cloud resource models. It
remains internal, so no package is selected for crates.io and provider code
continues to accumulate under the published package version until v0.65.0.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.63.0` | `metadata` | No | Record the complete Cloud model milestone and release evidence. |
| `cloud-sdk-hetzner` | `0.39.1` | `0.39.1` | `code` | No | Accumulate source-complete ordinary Cloud resources, pricing, generated schema validation, and dedicated checked results. |
| `cloud-sdk-reqwest` | `0.33.0` | `0.33.0` | `code` | No | Accumulate the exact `base64-ng 2.0.1` Basic-auth encoder update for v0.65.0. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.29.1` | `0.29.1` | `code` | No | Retain the cumulative v0.62 fixture changes for v0.65.0. |

## v0.65.0 Tracking Table

`v0.65.0` completes source-derived DNS response models and publishes all
reviewed code accumulated since v0.60.0. Independent code changes receive one
minor bump, the provider-neutral facade matches the tag, and unchanged
sanitization code is not republished.

| Crate | Previous published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.65.0` | `code` | Yes | Publish the cumulative provider-neutral contract and release checkpoint. |
| `cloud-sdk-hetzner` | `0.39.1` | `0.40.0` | `code` | Yes | Publish cumulative source-complete Cloud and DNS response models. |
| `cloud-sdk-reqwest` | `0.33.0` | `0.34.0` | `code` | Yes | Publish cumulative transport hardening and exact `base64-ng 2.0.1` integration. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No source or dependency change occurred after v0.60.0. |
| `cloud-sdk-testkit` | `0.29.1` | `0.30.0` | `code` | Yes | Publish cumulative exact-response and permit fixture improvements. |

Publish order is `cloud-sdk 0.65.0`, `cloud-sdk-reqwest 0.34.0`,
`cloud-sdk-testkit 0.30.0`, then `cloud-sdk-hetzner 0.40.0`. The unchanged
sanitization crate is skipped and the unpublished OVHcloud probe is outside the
publishable package set.

## v0.66.0 Tracking Table

`v0.66.0` completes source-derived certificate and SSH-key response models.
It remains internal, so no package is selected for crates.io and provider code
accumulates under the versions published at v0.65.0 until v0.70.0.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.66.0` | `metadata` | No | Record the complete security response-model milestone and release evidence. |
| `cloud-sdk-hetzner` | `0.40.0` | `0.40.0` | `code` | No | Accumulate complete certificate and SSH-key response models for v0.70.0. |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.0` | `unchanged` | No | No transport boundary changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Existing protected owned strings satisfy the secret-output boundary. |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.0` | `unchanged` | No | No testkit boundary changes. |

## v0.67.0 Tracking Table

`v0.67.0` completes source-derived Console Storage Box, type, snapshot, and
subaccount response models. It remains internal, so no package is selected for
crates.io and provider code continues to accumulate under the versions
published at v0.65.0 until v0.70.0.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.67.0` | `metadata` | No | Record the complete Console response-model milestone and release evidence. |
| `cloud-sdk-hetzner` | `0.40.0` | `0.40.0` | `code` | No | Accumulate complete Console Storage Box response models for v0.70.0. |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.0` | `unchanged` | No | No transport boundary changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Existing cleanup-owning strings satisfy the response-text boundary. |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.0` | `unchanged` | No | No testkit boundary changes. |

## v0.68.0 Tracking Table

`v0.68.0` proves one complete source-derived typed binding for every active
pre-Robot operation and explicit exclusion of every deprecated operation. It
remains internal, so no package is selected for crates.io and provider code
continues to accumulate under versions published at v0.65.0 until v0.70.0.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.68.0` | `metadata` | No | Record the complete typed-operation binding milestone and release evidence. |
| `cloud-sdk-hetzner` | `0.40.0` | `0.40.0` | `code` | No | Accumulate the 208-operation request, response, error, and policy proof for v0.70.0. |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.0` | `unchanged` | No | No transport boundary changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.0` | `unchanged` | No | No testkit boundary changes. |

## v0.69.0 Tracking Table

`v0.69.0` adds service-typed Hetzner client construction, endpoint trust,
complete workspace profiles, and generic checked read-only execution. It
remains internal, so no package is selected for crates.io and code continues
to accumulate under versions published at v0.65.0 until v0.70.0.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.69.0` | `code` | No | Add named complete-client capacity profiles and optional owned storage. |
| `cloud-sdk-hetzner` | `0.40.0` | `0.40.0` | `code` | No | Accumulate service-typed official/custom construction and checked read-only execution for v0.70.0. |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.0` | `unchanged` | No | No transport boundary changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Existing complete-buffer cleanup satisfies the new workspace owner. |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.0` | `unchanged` | No | Existing deterministic transport supports the vertical client evidence. |

## v0.70.0 Tracking Table

`v0.70.0` completes named typed workflows for all 139 active Cloud operations
and publishes the cumulative v0.66-v0.70 release train.

| Crate | Previous published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.65.0` | `0.70.0` | `code` | Yes | Publish the cumulative provider-neutral checkpoint and facade version. |
| `cloud-sdk-hetzner` | `0.40.0` | `0.41.0` | `code` | Yes | Publish cumulative response models, operation bindings, client foundation, and complete Cloud client methods. |
| `cloud-sdk-reqwest` | `0.34.0` | `0.34.1` | `dependency` | Yes | Require the v0.70 core without changing transport code. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No package content or dependency changed. |
| `cloud-sdk-testkit` | `0.30.0` | `0.30.1` | `dependency` | Yes | Require the v0.70 core without changing testkit code. |

Publish order is `cloud-sdk 0.70.0`, `cloud-sdk-reqwest 0.34.1`,
`cloud-sdk-testkit 0.30.1`, then `cloud-sdk-hetzner 0.41.0`. The unchanged
sanitization crate and unpublished OVHcloud probe are excluded.

## v0.71.0 Tracking Table

`v0.71.0` completes named typed workflows for all 24 active DNS operations,
retires the experimental FIPS transport, and begins the v0.71-v0.75 cumulative
train. No crate is published at this internal milestone.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.71.0` | `metadata` | No | Record the DNS client and FIPS-deferment milestone. |
| `cloud-sdk-hetzner` | `0.41.0` | `0.41.0` | `code` | No | Accumulate complete generated DNS client methods for v0.75.0. |
| `cloud-sdk-reqwest` | `0.34.1` | `0.34.1` | `code` | No | Accumulate removal of the experimental FIPS transport for v0.75.0. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.1` | `unchanged` | No | No testkit boundary changes. |

## v0.72.0 Tracking Table

`v0.72.0` completes named typed workflows for all 14 active certificate and
SSH-key operations. It remains an internal milestone in the v0.71-v0.75
cumulative train, so no crate is published.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.72.0` | `metadata` | No | Record the Security client milestone. |
| `cloud-sdk-hetzner` | `0.41.0` | `0.41.0` | `code` | No | Accumulate complete DNS and Security client methods for v0.75.0. |
| `cloud-sdk-reqwest` | `0.34.1` | `0.34.1` | `unchanged` | No | No transport boundary changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Existing complete-buffer cleanup satisfies private-key preparation. |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.1` | `unchanged` | No | Existing deterministic transports provide Security executor evidence. |

## v0.73.0 Tracking Table

`v0.73.0` completes named typed workflows for all 31 active Console Storage
Box operations. It remains an internal milestone in the v0.71-v0.75
cumulative train, so no crate is published.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.73.0` | `metadata` | No | Record the Console Storage client milestone. |
| `cloud-sdk-hetzner` | `0.41.0` | `0.41.0` | `code` | No | Accumulate complete DNS, Security, and Storage client methods for v0.75.0. |
| `cloud-sdk-reqwest` | `0.34.1` | `0.34.1` | `unchanged` | No | No transport boundary changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Existing complete-buffer cleanup satisfies Storage password preparation. |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.1` | `unchanged` | No | Existing deterministic transports provide Storage executor evidence. |

## v0.74.0 Tracking Table

`v0.74.0` source-locks all 105 official Robot operation headings, assigns the
89 active operations to implementation milestones, and excludes 16 deprecated
legacy Storage Box operations. It remains an internal milestone in the
v0.71-v0.75 cumulative train, so no crate is published.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.74.0` | `metadata` | No | Record the complete Robot source-lock milestone. |
| `cloud-sdk-hetzner` | `0.41.0` | `0.41.0` | `code` | No | Accumulate pre-Robot clients and repository-only Robot source evidence for v0.75.0. |
| `cloud-sdk-reqwest` | `0.34.1` | `0.34.1` | `unchanged` | No | No transport boundary changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | No sanitization boundary changes. |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.1` | `unchanged` | No | No testkit boundary changes. |

## v0.75.0 Tracking Table

`v0.75.0` adds the bounded Robot form codec and publishes the cumulative
v0.71-v0.75 train. Only crates with cumulative code or dependency changes are
selected.

| Crate | Previous published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.75.0` | `code` | Yes | Publish the cumulative provider-neutral checkpoint and standard transactional form primitive. |
| `cloud-sdk-hetzner` | `0.41.0` | `0.42.0` | `code` | Yes | Publish DNS, Security, Storage clients and the bounded Robot form codec. |
| `cloud-sdk-reqwest` | `0.34.1` | `0.35.0` | `code` | Yes | Publish the accumulated experimental FIPS transport removal. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Existing volatile cleanup primitive is reused without package changes. |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.2` | `dependency` | Yes | Require the v0.75 core without changing testkit behavior. |

Publish order is `cloud-sdk 0.75.0`, `cloud-sdk-reqwest 0.35.0`,
`cloud-sdk-testkit 0.30.2`, then `cloud-sdk-hetzner 0.42.0`. Sanitization and
the unpublished OVHcloud probe are excluded.

## v0.76.0 Tracking Table

`v0.76.0` adds protected Robot credentials and a reusable lockout-aware
attempt lifecycle. It begins the v0.76-v0.80 cumulative train. No crate is
published at this internal milestone.

| Crate | Published | Source | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.76.0` | `code` | No | Add the allocation-free shared credential-attempt generation and rejection lifecycle. |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | `code` | No | Accumulate Robot service identity, exact endpoint scope, protected credentials, rotation, and lockout policy for v0.80.0. |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | `unchanged` | No | No transport boundary changes. |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | `unchanged` | No | Reuse the existing admitted protected-string and cleanup boundary. |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | `unchanged` | No | No testkit boundary changes. |

## Planned Milestone Ownership

Exact independent crate versions are assigned when each milestone starts.
This table prevents a future release from accidentally publishing unrelated
crates or placing provider-specific behavior in a neutral boundary.

| Releases | Primary code owners | Purpose |
| --- | --- | --- |
| `v0.32.0 - v0.43.0` | `cloud-sdk`, neutral boundary crates, and `cloud-sdk-hetzner` only for migration and excluded fixtures | Neutral wire/isolation kernel, response provenance, atomic cleanup-owning state transfer, snapshot/version-locked encoding, policy-required auth scopes, narrow pre-freeze Robot wire conformance, and migration of all existing Hetzner operations. |
| `v0.44.0 - v0.56.0` | `cloud-sdk`, provider policy/decoder modules, adapters, and testkit as required | Structured and raw-link pagination, quota/retry strategies, cryptographic fingerprint domains, local async, progress-enforcing byte/chunk/observation-bounded streaming, typed operations, shared-state permits, client kernel, workflows, diagnostics, testkit, and drift tooling. |
| `v0.57.0 - v0.61.0` | excluded unpublished OVHcloud API v2 conformance package plus neutral contracts only when the probe proves them incomplete | Source lock, authority/OAuth, cursor/header, task/event, and end-to-end conformance. The probe must never enter the publish sequence. |
| `v0.62.0` | provider-neutral contracts, `cloud-sdk-hetzner` vertical slices, adapters, testkit, and documentation | Neutral API/semver freeze only after OVHcloud and Robot conformance plus full-fidelity Hetzner Cloud/DNS/security/Storage Box/error/no-content slices across every execution mode. |
| `v0.63.0 - v0.73.0` | `cloud-sdk-hetzner`, with neutral fixes only when genuinely provider-independent | Complete pre-Robot models, exact operation bindings, and Cloud/DNS/security/Console Storage Box clients. |
| `v0.74.0 - v0.95.0` | `cloud-sdk-hetzner`; neutral boundaries only for reusable behavior | Robot source lock, protocol, lockout-safe non-retryable authentication rejection, every active endpoint family, client integration, and live evidence. |
| `v0.96.0 - v0.99.0` | release tooling/docs and affected crates only for proven qualification fixes | Whole-platform adversarial, platform/MSRV, provenance, and controlled-mutation release-candidate evidence; FIPS remains deferred to Brynja. |
| `v1.0.0` | all changed publishable crates under independent version rules | Stable provider-neutral foundation and complete claimed Hetzner provider. |

Every milestone still follows the independent rules above: `cloud-sdk` matches
the release tag, code changes receive the crate's next minor version,
dependency-only changes receive a patch version, and unchanged crates are not
published.

After `v1.0.0`, published provider work proceeds as
`cloud-sdk-scaleway`, then `cloud-sdk-digitalocean`, then a separately planned
full `cloud-sdk-ovhcloud`. Workspace milestones `v1.1.0-v1.6.0` are assigned
to Scaleway and `v1.7.0-v1.12.0` to DigitalOcean; each provider crate receives
its independent package version when its first milestone starts. The excluded
`v0.57.0-v0.61.0` OVHcloud API v2 probe is never converted into a published
package; a full OVHcloud crate requires its own source lock, threat model, API
matrix, release plan, and independent version history.
