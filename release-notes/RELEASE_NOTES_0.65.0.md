# cloud-sdk 0.65.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-08

Security-Review: PASS
Pentest: PASS
Publication: PENDING

## Overview

v0.65 completes source-derived Hetzner DNS response models and publishes the
cumulative v0.61-v0.65 work. The default graph remains transport-free,
runtime-free, and `no_std`; response decoding remains behind the provider's
optional `serde` feature.

## DNS Responses

- Added dedicated typed zone and RRSet singleton, page, and create-composite
  results instead of generic resource identities.
- Preserved every current zone field: primary/secondary mode, status, creation
  time, TTL, record count, labels, deletion protection, registrar,
  authoritative/delegated nameservers, delegation state, and transfer primaries.
- Preserved RRSet IDs, owner names, nullable inherited TTLs, labels, change
  protection, records/comments, owning zones, and bounded additive future RR
  types.
- Moved returned TSIG keys directly into protected owned storage, redacted DNS
  diagnostics, checked canonical Base64, and kept legacy response observations
  separate from the HMAC-SHA256-only outbound policy.
- Enforced nonempty unique secondary-zone primaries, atomic TSIG key/algorithm
  state, a conservative record-count envelope, no ordinary equality on
  TSIG-bearing aggregates, and clear-on-drop DNS operational strings.
- Added immediate cleanup ownership for temporary and partially parsed DNS
  strings so later validation errors clear topology, labels, record values, and
  comments before releasing their allocations.
- Replaced quadratic RRSet duplicate detection with bounded borrowed-value
  sorting and exact 4,096-record regression coverage.
- Incrementally prevalidate zone pages, RRSet pages, and bounded zonefiles
  before duplicate-rejecting protected model decoding.

## Cumulative Checkpoint

- Includes the neutral OVHcloud/Robot conformance-driven API freeze from
  v0.61-v0.62 and source-complete ordinary Cloud, action, metrics, composite,
  pricing, location, certificate, and Storage Box response work from v0.63-v0.64.
- Includes exact `base64-ng 2.0.1` integration in the optional reqwest Basic-auth
  graph and cumulative exact-response/testkit improvements.
- Reviews the newer AWS-LC `1.18.0/0.44.0/0.14.1` set but retains the previous
  exact pins because the FIPS crate fails a clean read-only Cargo source build;
  runtime FIPS verification remains distinct from deployment accreditation.
- Pins and mechanically verifies those retained AWS-LC versions in the fuzz
  tooling graph as well as the production and feature-unification lockfiles;
  colored Cargo output and repeated tree rows are normalized before exact-set
  comparison without admitting missing, additional, or changed versions.
- Adds deterministic DNS schema generation, adversarial tests, named fuzz seeds,
  incremental coverage, and an ignored credential-gated typed zone live probe.

## Versions

| Crate | Previous published | v0.65 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.60.0` | `0.65.0` | yes |
| `cloud-sdk-hetzner` | `0.39.1` | `0.40.0` | yes |
| `cloud-sdk-reqwest` | `0.33.0` | `0.34.0` | yes |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | no, unchanged |
| `cloud-sdk-testkit` | `0.29.1` | `0.30.0` | yes |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0650`](../docs/PUBLIC_API_REVIEW.md#v0650)
- [`docs/DEPENDENCY_REVIEW.md#v0650`](../docs/DEPENDENCY_REVIEW.md#v0650)
- [`docs/THREAT_MODEL_DELTA.md#v0650`](../docs/THREAT_MODEL_DELTA.md#v0650)
- [`docs/REJECTED_ABSTRACTIONS.md#v0650`](../docs/REJECTED_ABSTRACTIONS.md#v0650)
- [`docs/MIGRATION.md#v0650`](../docs/MIGRATION.md#v0650)
- [`security/pentest/v0.65.0.md`](../security/pentest/v0.65.0.md)

## Release Gate

The incremental pentest and final retest passed. Run
`scripts/release_0_65_gate.sh` on the clean evidence commit, then require green
GitHub CI and CodeQL on that unchanged commit before the signed public tag and
crates.io publication.
