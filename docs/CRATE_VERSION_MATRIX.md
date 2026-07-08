# Crate Version Matrix

Status: `v0.1.0` repository foundation.

`cloud-sdk` is the provider-neutral entry point. Provider crates such as
`cloud-sdk-hetzner` own their endpoint models in internal modules. Extra
provider-specific crates are reserved for real optional boundaries: transport
adapters, test utilities, and secret-handling helpers.

## Version Rules

| Change kind | Version rule | Publish? |
| --- | --- | --- |
| `code` | `cloud-sdk` follows the milestone release version. Provider and boundary crates use independent minor bumps after their initial release. | Yes |
| `dependency` | Patch-bump the existing line when a manifest dependency range must change. | Yes |
| `metadata` | Use the release version when republishing corrected immutable package metadata. | Yes |
| `unchanged` | Keep the previous published version. | No |

## v0.1.0 Tracking Table

| Crate | Published | Planned | Change | Publish | Reason |
| --- | --- | --- | --- | --- | --- |
| `cloud-sdk` | none | `0.1.0` | `code` | Yes | Initial no_std provider-neutral cloud SDK foundation. |
| `cloud-sdk-hetzner` | none | `0.1.0` | `code` | Yes | Initial no_std Hetzner provider crate with internal Cloud, DNS, security, and Storage Box modules. |
| `cloud-sdk-hetzner-reqwest` | none | `0.1.0` | `code` | Yes | Initial optional reqwest transport adapter boundary without admitting reqwest yet. |
| `cloud-sdk-hetzner-sanitization` | none | `0.1.0` | `code` | Yes | Initial optional secret-sanitization boundary without admitting third-party dependencies yet. |
| `cloud-sdk-hetzner-testkit` | none | `0.1.0` | `code` | Yes | Initial testkit boundary for mock transports and fixtures. |
