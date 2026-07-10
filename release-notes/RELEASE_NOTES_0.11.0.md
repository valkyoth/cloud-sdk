# cloud-sdk 0.11.0 Release Notes

Status: release candidate; pentest remediation complete and retest pending.

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

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features load_balancers`
- `scripts/test-hetzner-api-drift.py`
- `scripts/test-pentest-binding.py`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_11_gate.sh`
- `git diff --check`

## Pentest

- Initial review reported release-attestation binding, pinned-spec integrity,
  secret-buffer failure cleanup, and remote-download resource limits.
- All four findings are remediated; retest is pending for the finalized
  release-sensitive commit.

## Publishing Plan

- `cloud-sdk` publishes as `0.11.0`.
- `cloud-sdk-hetzner` publishes as `0.11.0`.
- `cloud-sdk-hetzner-reqwest` publishes as `0.11.0`.
- `cloud-sdk-hetzner-sanitization` publishes as `0.11.0`.
- `cloud-sdk-hetzner-testkit` publishes as `0.11.0`.
