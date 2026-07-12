# Crate Version Matrix

Status: `v0.14.0` implementation candidate; pentest pending.

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
