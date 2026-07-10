# cloud-sdk 0.11.0 Release Notes

Status: implementation stop reached; pentest pending.

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

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features load_balancers`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_11_gate.sh`
- `git diff --check`

## Pentest

- Pending for the exact implementation-stop commit.

## Publishing Plan

The crate version and independent publish set will be finalized only after the
pentest and retest are green.
