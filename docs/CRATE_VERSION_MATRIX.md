# Crate Version Matrix

Status: `v0.31.0` is tagged. `v0.32.0` starts the pre-1.0
provider-neutral architecture-hardening track.

`cloud-sdk` is the provider-neutral entry point. Provider crates such as
`cloud-sdk-hetzner` own their endpoint models in internal modules. Shared
transport, test, and secret-handling boundaries remain provider-neutral so the
workspace normally adds only one primary crate for each provider.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | `cloud-sdk` always follows the release/tag version. Provider and boundary crates use independent minor bumps after their initial release. | Yes |
| `dependency` | Provider and boundary crates patch-bump the existing line when a manifest dependency range must change. `cloud-sdk` still follows the release/tag version. | Yes |
| `metadata` | Use the release/tag version when republishing corrected immutable package metadata or release evidence. | Yes |
| `unchanged` | Keep the previous published version. | No |

`cloud-sdk` is the facade crate and must publish on every release with the same
version as the tag. Other crates publish independently: real code changes move
to the next independent minor line, dependency-only related bumps stay on the
same minor line and increase only the patch number, metadata-only release
alignment uses the release/tag version, and unchanged crates are not published.

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

## Planned Milestone Ownership

Exact independent crate versions are assigned when each milestone starts.
This table prevents a future release from accidentally publishing unrelated
crates or placing provider-specific behavior in a neutral boundary.

| Releases | Primary code owners | Purpose |
| --- | --- | --- |
| `v0.32.0 - v0.35.0` | `cloud-sdk`, `cloud-sdk-hetzner`, `cloud-sdk-reqwest`, `cloud-sdk-testkit` as required | Extensible identities/endpoints, canonical HTTP metadata, raw execution/auth separation, and pagination/quota/idempotency strategies. |
| `v0.36.0 - v0.38.0` | `cloud-sdk`, provider decoder modules, adapters/testkit as required | Local async, streaming contracts, resource profiles, capacity reporting, automatic cleanup, and incremental decoding. |
| `v0.39.0 - v0.42.0` | `cloud-sdk`, `cloud-sdk-hetzner`, `cloud-sdk-testkit`, optional adapters | Typed operations, enforced permits, secure high-level client workflows, diagnostics, and workflow scenarios. |
| `v0.43.0` | provider-neutral drift tooling and documentation | Manifest-driven drift checks and canonical historical evidence. Published crates change only if a public contract is required. |
| `v0.44.0` | excluded unpublished OVHcloud API v2 conformance package plus neutral contracts that the probe proves incomplete | Exercise geographic authorities, OAuth2 bearer policy, versioned headers, cursor pagination, and asynchronous resources before the neutral API freeze. The probe must never enter the publish sequence. |
| `v0.45.0 - v0.46.0` | `cloud-sdk-hetzner`, with neutral fixes only when genuinely provider-independent | Complete Cloud, DNS, security, and Console Storage Box response models. |
| `v0.47.0 - v0.58.0` | `cloud-sdk-hetzner`; neutral auth/transport/testkit crates only for reusable behavior | Robot source lock, protocol, endpoint families, client integration, and complete Hetzner hardening. |
| `v0.59.0 - v0.60.0` | release tooling/docs, affected crates only for proven release-candidate fixes | Provenance/governance review, controlled mutation evidence, and final 1.0 release candidate. |
| `v1.0.0` | all changed publishable crates under independent version rules | Stable provider-neutral foundation and complete claimed Hetzner provider. |

Every milestone still follows the independent rules above: `cloud-sdk` matches
the release tag, code changes receive the crate's next minor version,
dependency-only changes receive a patch version, and unchanged crates are not
published.

After `v1.0.0`, published provider work proceeds as
`cloud-sdk-scaleway`, then `cloud-sdk-digitalocean`, then a separately planned
full `cloud-sdk-ovhcloud`. Exact crate and workspace versions are assigned when
each provider starts. The excluded `v0.44.0` OVHcloud API v2 probe is never
converted into a published package; a full OVHcloud crate requires its own
source lock, threat model, API matrix, release plan, and independent version
history.
