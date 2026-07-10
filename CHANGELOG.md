# Changelog

## Unreleased

- Added `cloud-sdk-hetzner::cloud::load_balancers` no_std request primitives
  for Load Balancer CRUD, metrics, services, targets, networks, reverse DNS,
  protection, algorithms, type changes, and public-interface actions.
- Added protocol-safe HTTP/HTTPS service settings, bounded health checks,
  mutually exclusive target selection, public server-IP validation, explicit
  reverse-DNS set/reset intent, and deterministic multi-metric queries.
- Added the `v0.11.0` release gate script.

## 0.10.0 - 2026-07-10

- Added `cloud-sdk-hetzner::cloud::firewalls` no_std request primitives for
  Firewall CRUD, resource apply/remove actions, and rule replacement.
- Added `cloud-sdk-hetzner::cloud::networks` no_std request primitives for
  Network CRUD, routes, subnets, IP range changes, and protection actions.
- Added allocation-free canonical IPv4/IPv6 Firewall CIDR, RFC 1918 range,
  route, gateway, Firewall direction/protocol, port-range, and rule-conflict
  validation.
- Updated the development compiler to current stable Rust `1.97.0` and the CI
  `cargo-deny` pin to current `0.20.2`.
- Hardened Firewall CIDR validation after pentest review by rejecting IPv4 and
  IPv6 host bits while retaining `/32` and `/128` single-host selectors.
- Added the `v0.10.0` release gate script.

## 0.9.0 - 2026-07-09

- Added `cloud-sdk-hetzner::storage::storage_boxes` no_std request primitives
  for Storage Box CRUD, folder listing, type catalog endpoints, snapshots,
  subaccounts, Storage Box actions, and subaccount actions scheduled for
  `v0.9.0`.
- Added source-locked Storage Box query builders for box, type, action,
  snapshot, and subaccount list endpoints.
- Added redacted Storage Box password markers, bounded snapshot-plan markers,
  conservative home-directory validation, and explicit deferred policy for the
  deprecated resource-local action lookup endpoint.
- Hardened v0.9.0 Storage Box request validation after pentest review by
  rejecting trimmed `.` and `..` home-directory segments and documenting caller
  zeroization responsibility for password output buffers.
- Added the `v0.9.0` release gate script.

## 0.8.0 - 2026-07-09

- Added `cloud-sdk-hetzner::cloud::volumes` no_std request primitives for
  volume list/create/get/update/delete and volume action endpoints scheduled
  for `v0.8.0`.
- Added `cloud-sdk-hetzner::cloud::networks::floating_ips` no_std request
  primitives for floating IP list/create/get/update/delete and floating IP
  action endpoints.
- Added bounded Volume size markers, explicit volume server/location
  placement, explicit floating IP server/home-location placement, and explicit
  floating IP DNS pointer set/reset intent.
- Hardened v0.8.0 storage/IP tests after pentest review by making fallible
  fixture constructor failures explicit assertion failures.
- Added the `v0.8.0` release gate script.

## 0.7.0 - 2026-07-09

- Added `cloud-sdk-hetzner::cloud::images` no_std request primitives for
  image list/get/update/delete and image action endpoints scheduled for
  `v0.7.0`.
- Added `cloud-sdk-hetzner::cloud::servers::placement_groups` no_std request
  primitives for placement group list/create/get/update/delete.
- Added `cloud-sdk-hetzner::cloud::networks::primary_ips` no_std request
  primitives for primary IP list/create/get/update/delete and primary IP
  action endpoints.
- Added shared Cloud request helpers for nonzero IDs, bounded JSON-safe
  text/name values, ordered labels, fixed-buffer paths, and query construction.
- Hardened v0.7.0 request intent by requiring explicit primary IP DNS pointer
  set/reset and by omitting removed datacenter request fields from create and
  update builders.
- Removed unused public cloud request error variants after pentest review and
  documented that primary IP address and DNS pointer semantic validation must
  be added before future body serialization.
- Added the `v0.7.0` release gate script.

## 0.6.0 - 2026-07-08

- Added `cloud-sdk-hetzner::cloud::servers` no_std request primitives for
  server list/create/get/update/delete and metrics endpoints scheduled for
  `v0.6.0`.
- Added source-locked server action endpoint paths and request markers for
  power, reboot, reset, shutdown, rebuild, rescue, backup, ISO, network,
  placement group, DNS pointer, protection, type change, image creation,
  console, and password reset operations.
- Added explicit DNS pointer set/reset intent and validation for server create
  required fields, public network mutual exclusions, and metrics time ranges.
- Hardened v0.6.0 server request validation after pentest review by redacting
  cloud-init user data in `Debug`, fixing zero numeric query serialization,
  rejecting JSON-significant and bidi-control bytes in server text values, and
  requiring fixed-width digit-only metrics timestamps.
- Added `cloud_sdk::buffer`, a shared no_std fixed-buffer writer used by
  server and security request domains for string, decimal, query, and percent
  encoding output.
- Added shared JSON-string escaping for future body serializers and a
  `UserData` body-writing path that avoids raw JSON interpolation without
  exposing a raw string accessor.
- Added the `v0.6.0` release gate script.

## 0.5.0 - 2026-07-08

- Added `cloud-sdk-hetzner::security::ssh_keys` no_std request primitives for
  SSH key list/create/get/update/delete endpoints scheduled for `v0.5.0`.
- Added `cloud-sdk-hetzner::security::certificates` no_std request primitives
  for certificate list/create/get/update/delete and retry action endpoints
  scheduled for `v0.5.0`.
- Added conservative validation and redacted `Debug` output for
  secret-adjacent SSH public key and certificate PEM request values.
- Hardened v0.5.0 security request validation after pentest review by checking
  PEM marker order, capping SSH fingerprint filters, rejecting duplicate label
  keys, and clarifying uploaded certificate mode guarantees.
- Added the `v0.5.0` release gate script.

## 0.4.0 - 2026-07-08

- Added `cloud-sdk-hetzner::cloud::catalog` no_std request primitives for
  read-only catalog endpoints scheduled for `v0.4.0`.
- Added source-locked path construction, pagination, and sorting tests for
  locations, pricing, server types, load balancer types, ISOs, and public image
  catalog requests.
- Added the `v0.4.0` release gate script.

## 0.3.0 - 2026-07-08

- Added crate-local README and rustdoc entry-point documentation for every
  workspace crate.
- Clarified that `cloud-sdk-hetzner` is the main Hetzner-specific documentation
  surface while `cloud-sdk` remains the provider-neutral foundation.
- Added the first `v0.3.0` core request/response policy domains in
  `cloud-sdk-hetzner`: endpoint paths, base URL selection, bounded query
  parameters, labels, pagination, sorting, action status, API errors, and
  rate-limit metadata.
- Added fixed-buffer query percent encoding, endpoint group base URL mapping,
  and stricter label selector structure checks for `v0.3.0`.
- Hardened endpoint path validation against authority overrides, query or
  fragment injection, and parent directory segments after pentest review.
- Added the `v0.3.0` release gate script.

## 0.2.0 - 2026-07-08

- Source-locked the official Hetzner Cloud/DNS and Storage Box OpenAPI specs.
- Replaced the initial group-level API plan with a complete 221-operation
  endpoint matrix, including pagination, sorting, action, deprecation, and owner
  module tracking.
- Added local Hetzner upstream lock validation and a `v0.2.0` release gate.
- Added Hetzner API drift fingerprints and a drift detector for upstream
  operation and schema changes.
- Expanded the release plan so every milestone has concrete deliverables,
  verification commands, and an explicit pentest stop gate.
- Hardened the API drift lock refresh path to require explicit acknowledgement
  and pinned SHA-256 verification before overwriting fingerprint locks.
- Documented current Hetzner changelog items that affect future SDK models,
  including deprecated omitted DNS pointer and RRSet TTL fields, deprecated
  datacenter endpoints, and deprecated resource-local action lookups.

## 0.1.0 - 2026-07-08

- Initialized the `cloud-sdk` Rust workspace.
- Added `cloud-sdk` as the provider-neutral crate.
- Added `cloud-sdk-hetzner` as the first provider crate.
- Added one no_std SDK crate with internal Cloud, DNS, security, and Storage Box
  modules.
- Added placeholder crates for future reqwest transport, testkit, and
  sanitization boundaries.
- Added MIT OR Apache-2.0 licensing, security policy, dependency policy, CI
  metadata, and release planning.
- Added local checks for formatting, linting, tests, no_std policy, modularity,
  shell syntax, security policy, and file length.
- Hardened release gates for pentest evidence, no_std policy validation, and
  required dependency security tools.
- Configured CI checkout with full history so pentest reviewed-commit ancestry
  checks work on GitHub Actions.
