# Dependency Review 0.94.0

Status: implementation stop; incremental pentest required.

## Result

v0.94 adds no dependency edge, feature, unsafe code, native build, network
client, runtime, filesystem, clock, randomness, or secret-store edge. It
updates three already-admitted optional transport packages and refreshes
compatible transitive packages in the locked graphs.

Robot clients compose the existing provider-neutral client kernel, bounded
workspace leases, official endpoint and Basic-auth binding, credential-attempt
state, cleanup guards, prepared operations, permit families, and strict Robot
decoders. Provider crates remain transport-free and default features remain
empty.

The ordinary, non-FIPS AWS-LC graph advances to `aws-lc-rs 1.18.0` and
`aws-lc-sys 0.44.0`, backed by AWS-LC 5.5.0. The update includes upstream
hardening for short key-wrap ciphertexts and a read-only prebuilt-source build
fix. The workspace does not activate `aws-lc-rs/fips`, and no
`aws-lc-fips-sys` package is present in any locked graph. FIPS support remains
deferred under [`FIPS_DEFERMENT.md`](FIPS_DEFERMENT.md).

`http-body-util 0.1.5` is a documentation-only patch over 0.1.4. These three
packages retain MSRVs below the workspace's Rust 1.92 floor. Full freshness,
deny, RustSec, SBOM, feature-unification, package, and platform gates remain
required before tagging.

## Lockfile Changes

| Package | Previous | v0.94 | Review |
| --- | --- | --- | --- |
| `aws-lc-rs` | `1.17.3` | `1.18.0` | Exact optional cryptographic-provider update; FIPS remains disabled and absent. |
| `aws-lc-sys` | `0.43.0` | `0.44.0` | Exact optional native AWS-LC update from 5.2.0 to 5.5.0. |
| `cc` | `1.4.2` | `1.4.3` | Compatible native build-driver patch update. |
| `cloud-sdk` | `0.93.0` | `0.94.0` | Advance the internal facade for complete Robot clients. |
| `find-msvc-tools` | `0.1.10` | `0.1.11` | Compatible MSVC discovery patch update used by native builds. |
| `futures-channel` | `0.3.33` | `0.3.34` | Compatible async channel patch update. |
| `futures-core` | `0.3.33` | `0.3.34` | Compatible async core patch update. |
| `futures-io` | `0.3.33` | `0.3.34` | Compatible async I/O patch update. |
| `futures-sink` | `0.3.33` | `0.3.34` | Compatible async sink patch update. |
| `futures-task` | `0.3.33` | `0.3.34` | Compatible async task patch update. |
| `futures-util` | `0.3.33` | `0.3.34` | Compatible async utility patch update. |
| `http-body-util` | `0.1.4` | `0.1.5` | Admitted raw-body utility documentation patch. |
| `icu_collections` | `2.2.0` | `2.3.0` | Compatible URL/IDNA Unicode-data update. |
| `icu_locale_core` | `2.2.0` | `2.3.0` | Compatible URL/IDNA locale-core update. |
| `icu_normalizer` | `2.2.0` | `2.3.0` | Compatible URL/IDNA normalization update. |
| `icu_normalizer_data` | `2.2.0` | `2.3.0` | Data paired with the ICU normalizer update. |
| `icu_properties` | `2.2.0` | `2.3.0` | Compatible URL/IDNA Unicode-property update. |
| `icu_properties_data` | `2.2.0` | `2.3.0` | Data paired with the ICU properties update. |
| `icu_provider` | `2.2.0` | `2.3.0` | Compatible ICU data-provider update. |
| `litemap` | `0.8.2` | `0.8.3` | Compatible ICU map patch update. |
| `ovhcloud-v2-probe` | `0.93.0` | `0.94.0` | Advance the excluded workspace probe identity only. |
| `pkg-config` | `0.3.33` | `0.3.34` | Compatible native build-discovery patch update. |
| `potential_utf` | `0.1.5` | `0.1.6` | Compatible ICU UTF handling patch update. |
| `rustls-webpki` | `0.103.13` | `0.103.14` | Compatible rustls certificate-validation patch update. |
| `tinystr` | `0.8.3` | `0.8.4` | Compatible ICU small-string patch update. |
| `writeable` | `0.6.3` | `0.6.4` | Compatible ICU formatting patch update. |
| `zerotrie` | `0.2.4` | `0.2.5` | Compatible ICU trie patch update. |
| `zerovec` | `0.11.6` | `0.11.7` | Compatible ICU zero-copy vector patch update. |
| `zerovec-derive` | `0.11.3` | `0.11.4` | Derive implementation paired with `zerovec`. |

The exact newly admitted Cargo archive checksums are:

| Package | SHA-256 | Rust version |
| --- | --- | --- |
| `aws-lc-rs 1.18.0` | `ce2b2dcc879c3bae0d371e77c99f2238400ef24ec001394befa67b6e543add9e` | `1.71.0` |
| `aws-lc-sys 0.44.0` | `f09fae7be8bb3174e05c6afdb34199e6dc0c7c04ba9fa237b1967adfbde27483` | `1.71.0` |
| `http-body-util 0.1.5` | `23169fe34a5fbcdd3f3862e78fb9b6fccd5f02a6dc6f732547005d45631ce71c` | `1.61` |

Cargo registry metadata and upstream AWS-LC release notes were checked on
2026-08-17. Rust 1.97.1, `actions/checkout v7.0.1`, `cargo-deny 0.20.2`,
`cargo-audit 0.22.2`, `cargo-sbom 0.10.0`, and `cargo-fuzz 0.13.2` remain
current. The isolated fuzz toolchain advances from `nightly-2026-07-26` to
`nightly-2026-08-17`; it is not part of the published crate MSRV contract.

## Publication Selection

| Package | Published | v0.94 source | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.94.0` | code | no |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | code | no |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | code and dependency updates accumulated | no |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | code accumulated | no |

`scripts/release_crates.py` must select no package. Fuzz, tools, isolated
tests, and the OVHcloud probe remain excluded.
