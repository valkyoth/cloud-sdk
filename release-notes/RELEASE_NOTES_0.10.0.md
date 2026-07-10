# cloud-sdk 0.10.0 Release Notes

Status: release candidate; pentest and retest complete.

## Scope

`0.10.0` adds no_std Hetzner Firewall and Network request primitives in
`cloud-sdk-hetzner`. It does not add HTTP transport, request-body
serialization, response models, token storage, live API tests, retry policy,
pagination iterators, or action polling.

## Added

- Firewall list/create/get/update/delete endpoint and request domains.
- Firewall action paths for global and resource action lists, action lookup,
  applying/removing resources, and replacing rules.
- Direction-specific Firewall selectors that prevent source/destination field
  conflicts by construction.
- Firewall protocol, port/range, CIDR-count, description, duplicate-selector,
  duplicate-rule, and 50-rule limit validation.
- Firewall server and label-selector application targets.
- Network list/create/get/update/delete endpoint and request domains.
- Network action paths and bodies for routes, subnets, IP range changes, and
  delete protection.
- Non-deprecated Cloud and vSwitch subnet models with vSwitch IDs required by
  construction.
- Allocation-free canonical IPv4/IPv6 Firewall CIDR parsing and canonical RFC
  1918 Network, subnet, route destination, and gateway validation.
- `scripts/release_0_10_gate.sh`.
- Current stable Rust `1.97.0` development pin and `cargo-deny 0.20.2` CI pin;
  cargo-deny's Rust `1.88.0` requirement remains below this workspace's Rust
  `1.90.0` MSRV.

## Security Notes

- The default graph remains no_std and transport-free.
- Endpoint paths validate the fully assembled path through `EndpointPath`.
- List queries use caller-owned fixed buffers and percent encoding.
- Network and Firewall IDs remain nonzero.
- Network ranges must be canonical RFC 1918 IPv4 networks of at least `/24`;
  subnets must be at least `/30`.
- Firewall CIDRs reject host bits as required by Hetzner since 10 December
  2025; individual hosts remain valid as `/32` or `/128`.
- Route destinations are private IPv4 CIDRs or `0.0.0.0/0`; gateways are
  private IPv4 addresses, and Hetzner's reserved `172.31.1.1` is rejected.
- Project-state checks such as overlap with existing routes/subnets, gateway
  membership, and server/label existence remain enforced by Hetzner because
  transport-free request values do not have project state.
- The deprecated resource-local action lookup endpoints remain intentionally
  deferred.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test -p cloud-sdk-hetzner --all-features networks_firewalls`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/checks.sh`
- `scripts/release_0_10_gate.sh`
- `git diff --check`

## Pentest

- PASS. Permanent report: `security/pentest/v0.10.0.md`.
- Retest is green. No blocking findings remain for tagging `v0.10.0`.

## Publishing Plan

- `cloud-sdk` publishes as `0.10.0`.
- `cloud-sdk-hetzner` publishes as `0.10.0`.
- `cloud-sdk-hetzner-reqwest` publishes as `0.10.0`.
- `cloud-sdk-hetzner-sanitization` publishes as `0.10.0`.
- `cloud-sdk-hetzner-testkit` publishes as `0.10.0`.
