# Serde Dependency Admission

Status: admitted only behind the non-default `cloud-sdk-hetzner/serde` feature.

Checked: 2026-07-12.

## Packages

| Package | Version | Scope | License | Default features |
| --- | --- | --- | --- | --- |
| `serde` | `1.0.228` | optional normal dependency | MIT OR Apache-2.0 | disabled |
| `serde_json` | `1.0.150` | test and fixture development only | MIT OR Apache-2.0 | enabled |

Serde is sourced from crates.io and maintained at
<https://github.com/serde-rs/serde>. serde_json is sourced from crates.io and
maintained at <https://github.com/serde-rs/json>.

## Feature Policy

The provider's `serde` feature enables its existing `alloc` boundary and
Serde's `alloc` and `derive` features. It does not enable Serde `std`. The
workspace's default graph remains the two local crates `cloud-sdk-hetzner` and
`cloud-sdk`, with no third-party normal dependency.

serde_json is not a runtime dependency. It is used only to test source-locked
JSON fixtures and adversarial duplicate, missing, unknown, and invalid fields.
Future transports must perform their own parser admission review rather than
assuming the dev dependency is approved for production.

## Transitive Surface

The derive feature adds build-time proc-macro dependencies `serde_derive`,
`proc-macro2`, `quote`, `syn`, and `unicode-ident`. serde_json's test-only graph
adds `itoa`, `memchr`, and `zmij`. `cargo deny`, `cargo audit`, locked versions,
and the workspace MSRV matrix cover the complete graph.

## Security Decision

Blanket derives are not applied to validated request structs. Only a checked
`RrsetRequestBody` wrapper implements complete request-body serialization, so
path selectors are omitted and a conservative 1 MiB JSON upper bound is
checked before serialization. Response-only action and error envelopes use
private wire structs and validate nonzero IDs, action status, progress, and
control bytes after parsing. `ResponseBytes` caps raw parser input at 8 MiB,
and action resource arrays and interpreted text have independent model bounds.

Known response fields reject duplicates and missing required fields through
Serde's generated map visitors. Unknown response fields are ignored for
forward compatibility with additive provider changes. Requests are emitted
from closed SDK types and never deserialize around validation constructors.

## Alternatives Considered

- Hand-written JSON for every body would duplicate a mature escaping engine
  and increase parser and serializer review surface.
- Blanket `Serialize` and `Deserialize` derives would produce incorrect body
  shapes and permit deserialization around validation constructors.
- A separate provider-specific Serde crate would violate the one-crate-per-
  provider policy and multiply packages for future providers.

## Automated Enforcement

`scripts/check_serde_boundary.sh` verifies the empty default graph, optional
no_std feature graph, absence of Serde `std`, and focused fixtures. It runs from
`scripts/checks.sh` and the v0.14 release gate. Provider package verification
also compiles the packaged tarball with `serde` enabled.
