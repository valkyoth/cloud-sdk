# cloud-sdk 0.11.0 Release Notes

Status: release candidate; pentest and retest complete.

## Scope

`0.11.0` adds no_std Hetzner Load Balancer request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, request-body
serialization, response models, token storage, live API tests, retry policy,
pagination iterators, or action polling.

## Added

- Load Balancer list/create/get/update/delete and metrics request domains.
- Global and resource action lists plus global action lookup.
- Service add/update/delete models for TCP, HTTP, and HTTPS.
- Protocol-specific HTTP and HTTPS settings for sticky sessions, certificates,
  redirects, and idle timeouts.
- TCP and HTTP health-check models with source-locked port, interval, timeout,
  retry, path, response, status-code, and TLS boundaries.
- Server, label-selector, and direct-IP target models for add/remove actions.
- Network attach/detach, algorithm, reverse-DNS, protection, type-change, and
  public-interface action models.
- `scripts/release_0_11_gate.sh`.
- Atomic shared JSON-string writes that leave undersized caller buffers
  unchanged, including Storage Box passwords and cloud-init user data.
- Full-byte pinned SHA-256 verification before any fetched OpenAPI document is
  parsed, plus 32 MiB, connection-time, and total-time download ceilings.
- Content-bound pentest evidence for release-sensitive paths from `v0.11.0`,
  mandatory verifiable signed tags for publishing, and a normal publisher
  without bypass flags.

## Security Notes

- The default graph remains no_std, allocation-free, and transport-free.
- Endpoint paths validate the fully assembled path through `EndpointPath`.
- List and metrics queries use caller-owned fixed buffers and percent encoding.
- Resource, action, server, certificate, and network IDs remain nonzero.
- Service protocols carry only their compatible HTTP or HTTPS settings.
- Target selection is an enum, and private-IP mode is rejected for direct-IP
  targets and server targets that also select a public address.
- Server public-IP targets reject private, loopback, link-local, multicast,
  unspecified, and broadcast address space.
- Reverse-DNS actions require explicit set or JSON-null reset intent because
  omission is deprecated upstream.
- Metrics require a non-empty, duplicate-free metric selection and an
  increasing, calendar-valid UTC timestamp range.
- Resource ownership, target membership, address assignment, subnet
  containment, and current project state remain enforced by Hetzner.
- The deprecated resource-local action lookup endpoint remains intentionally
  deferred.
- Release-sensitive source, manifests, lockfiles, scripts, and workflow files
  may not change after the commit named by the pentest report.
- Public server targets conservatively admit only ordinary global-unicast IPv6
  allocations source-locked from IANA; translated, transition, benchmarking,
  documentation, deprecated, special-purpose, unallocated, and reserved ranges
  are rejected.
- Release metadata requires exactly one unambiguous `Reviewed-Commit:` field.
- Permanent pentest reports require an OpenSSH-signed attestation from the
  approved pentest key, which is distinct from the release-tag signing key.
- Pentest reports, signed attestation bundles, and signer policy are opened
  without following links and copied into private bounded snapshots. The
  signed metadata binds the report's immutable Git commit, path, and SHA-256.
- Source-lock files, release notes, changelog, and security documentation are
  included in post-review content binding.
- Local OpenAPI inputs are opened once without following links and validated
  from the resulting descriptor; symlinks, FIFOs, devices, and oversized files
  are rejected before hashing or parsing.
- Pentest signing requires a clean repository, reads the committed report as a
  Git blob, and publishes one complete signed bundle with a single
  non-overwriting hard link.
- The release gate fetches bounded copies of both IANA IPv6 registries and
  fails on digest, allocation, special-range, lock-file, or Rust-policy drift.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features load_balancers`
- `scripts/test-hetzner-api-drift.py`
- `scripts/test-pentest-binding.py`
- `scripts/test-iana-ipv6-registry.py`
- `scripts/check_iana_ipv6_registry.py --fetch`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_11_gate.sh`
- `git diff --check`

## Pentest

- Initial review reported release-attestation binding, pinned-spec integrity,
  secret-buffer failure cleanup, and remote-download resource limits.
- Subsequent review tightened IPv6 classification, independent pentest
  provenance, content-binding scope, and local OpenAPI file handling.
- Final remediation source-locked IANA IPv6 allocations, removed signed-report
  and local-spec pathname races, and hardened signature publication.
- Final signing hardening binds evidence to a committed Git blob, makes future
  unsupported IPv6 prefix lengths fail closed, and authenticates IANA bytes
  before CSV parsing.
- Signed pentest evidence is published as one bounded transactional bundle, so
  failed publication cannot leave a partial attestation/signature pair.
- All findings are remediated; retest passed for the finalized
  release-sensitive commit.

## Publishing Plan

- `cloud-sdk` publishes as `0.11.0`.
- `cloud-sdk-hetzner` publishes as `0.11.0`.
- `cloud-sdk-hetzner-reqwest` publishes as `0.11.0`.
- `cloud-sdk-hetzner-sanitization` publishes as `0.11.0`.
- `cloud-sdk-hetzner-testkit` publishes as `0.11.0`.
