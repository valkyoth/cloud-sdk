# cloud-sdk 1.1.1

Status: maintenance candidate; pentest and GitHub acceptance remain required.
No tag or crates.io publication is authorized by this document.

## Security and Dependencies

- Update webpki-roots to 1.0.10 and rustls-webpki to 0.103.16 across
  the applicable workspace and isolated lockfiles. These address
  [RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381.html)
  and [RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382.html).
- Thanks to @Manishearth and @manishearth-agent for reporting the root-store
  update in [PR #14](https://github.com/valkyoth/cloud-sdk/pull/14).
- Refresh the admitted transport, serialization, sanitization and tooling
  dependencies. Exact lockfile changes are recorded in
  [Dependency Review](../docs/DEPENDENCY_REVIEW.md).
- Develop on Rust 1.99.1; retain MSRV 1.92 and add 1.99.0/1.99.1 to CI.
  Refresh the isolated fuzz nightly to 2026-10-22. GitHub checkout and pinned
  Cargo tools were checked for current releases.

## Hetzner

- Bind typed load-balancer deletion responses to the requested load-balancer
  ID and resource type. Missing or unrelated action resources fail closed.
- Restrict `CloudResourceId` (and its aliases, including Primary IP IDs) to
  `1..=9_007_199_254_740_991`, matching the provider schema. This validation
  correction rejects previously accepted invalid inputs; valid identifiers
  and public signatures are unchanged. Robot identifiers are unaffected.
- Load-balancer creation can select existing IPv4 and IPv6 Primary IPs with
  `with_public_net`. Null selections retain provider-assigned addresses.
  Supplying this object with a disabled public interface fails preparation.
- Adapt load-balancer deletion to Hetzner's 201 action response, replacing the
  former 204 empty response. Callers with explicitly named deletion result
  types must use the action response and poll completion where necessary.
- Refresh the bounded response schema for public-network IP IDs and blocked
  flags. Source locks record explicit pagination bounds already enforced by
  the SDK. No new Cloud operations were added; Robot is unchanged.
- Record server subnet-type and Link-header deprecations. Existing server
  subnets remain accepted during upstream's transition; pagination uses JSON
  metadata, not Link headers. Deprecated datacenter operations remain excluded.
- Refresh IANA IPv6 registry digests after reviewing unchanged allocated
  prefixes and the existing special-purpose exclusions; no address admission
  policy is broadened.

## crates.io

- Review the current Cargo registry protocol, public OpenAPI and access policy.
  The 51-operation inventory is unchanged.
- Refresh crate-settings schema validation for the equivalent plain-object
  request schema and Version nullable-union ordering. No new mutation or
  authentication capability is introduced.
- Correct generated enum fixture values when a non-null union branch is first;
  retain strict response validation and cover both nullable alternatives.
- Preserve historical pinned controller evidence; this refresh does not claim
  that every upstream backend commit has been independently audited.

All six packages are synchronized to 1.1.1 as candidates. Scaleway development
remains on its separate branch. FIPS remains deferred.

## Local Verification

Completed on 2026-10-22:

- `scripts/checks.sh`: workspace default/all-feature tests, doctests,
  warning-denied Clippy, generator regressions, boundaries and package checks.
- `scripts/check_rust_version_matrix.sh`: all 14 supported Rust versions,
  from 1.92.0 through 1.99.1.
- `scripts/check_platform_matrix.sh --all`: ten portable target configurations,
  unsupported-transport rejection and Linux native transport checks. Other
  native operating systems still require their GitHub runner jobs.
- `scripts/check_packaged_feature_graphs.sh`: all six published-package graphs.
- `scripts/check_fuzz_harness.sh --smoke`: 39 targets; bounded smoke coverage,
  not exhaustive fuzzing.
- Four-lockfile RustSec scan, `cargo deny check`, complete SBOM freshness,
  warning-denied rustdoc, documentation links and file-length policy.
- Exact dependency/tool freshness, live Hetzner/Robot/changelog and IANA checks,
  crates.io source/semantic/endpoint/controller checks and generated projections.
- Candidate publication rejection and release-policy/readiness regression tests.

The host's distro-managed rustup 1.29.1 cannot self-update; upstream 1.29.2
requires a system-package update. The pinned Rust 1.99.1 compiler is installed
and verified. No live credentialed mutations were performed.

These are implementation qualification results, not a pentest report or
authorization to publish. Review range: `v1.1.0..HEAD` on `main`.
