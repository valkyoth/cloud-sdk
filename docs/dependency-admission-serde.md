# Serde Dependency Admission

Status: admitted only behind the non-default `cloud-sdk-hetzner/serde` feature.

Checked: 2026-07-16.

## Packages

| Package | Version | Scope | License | Default features |
| --- | --- | --- | --- | --- |
| `serde` | `1.0.228` | optional normal dependency | MIT OR Apache-2.0 | disabled |
| `serde_json` | `1.0.150` | optional normal dependency and test parser | MIT OR Apache-2.0 | disabled; `alloc` only |

Serde and serde_json are sourced from crates.io and maintained by the Serde
project at <https://github.com/serde-rs/serde> and
<https://github.com/serde-rs/json>.

## Feature Policy

The provider's `serde` feature enables its existing `alloc` boundary, Serde's
`alloc` and `derive` features, and serde_json's `alloc` parser. Neither parser
enables `std`. The workspace's default graph remains the two local crates
`cloud-sdk-hetzner` and `cloud-sdk`, with no third-party normal dependency.

serde_json is admitted as the non-default checked-decoder parser in v0.31. Its
generic `Value` representation remains private. A bounded parser seed rejects
duplicate keys, excessive nesting, oversized strings, and oversized
containers before resource conversion. Only validated provider-owned models
cross the public boundary.

## Transitive Surface

The derive feature adds build-time proc-macro dependencies `serde_derive`,
`proc-macro2`, `quote`, `syn`, and `unicode-ident`. serde_json adds `itoa`,
`memchr`, and `zmij`. `cargo deny`, `cargo audit`, locked versions, and the
workspace MSRV matrix cover the complete optional graph.

## Security Decision

Blanket derives are not applied to validated request structs. Only a checked
`RrsetRequestBody` wrapper implements complete request-body serialization, so
path selectors are omitted and a conservative 1 MiB JSON upper bound is
checked before serialization. Response-only action and error envelopes use
private wire structs and validate nonzero IDs, action status, progress, and
control bytes after parsing. `ResponseBytes` caps raw parser input at 8 MiB,
and action resource arrays and interpreted text have independent model bounds.

Known and unknown response fields first pass the duplicate-rejecting bounded
JSON admission layer. The source-locked operation table then requires the
exact success status, envelope family, root key, and required top-level fields.
Unknown response fields are ignored only after admission and are never exposed
without model validation. Requests are emitted from closed SDK types and never
deserialize around validation constructors.

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
