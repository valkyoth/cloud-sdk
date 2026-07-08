# Crate Version Matrix

Status: `v0.2.0` release candidate.

`cloud-sdk` is the provider-neutral entry point. Provider crates such as
`cloud-sdk-hetzner` own their endpoint models in internal modules. Extra
provider-specific crates are reserved for real optional boundaries: transport
adapters, test utilities, and secret-handling helpers.

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
