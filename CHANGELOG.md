# Changelog

## 0.23.0 - 2026-07-14

- Added the non-default `cloud-sdk-reqwest/blocking-rustls-fips` feature.
- Built each FIPS-mode client from an explicit rustls FIPS provider and
  rejected providers or complete TLS configurations that do not report FIPS.
- Kept process-global rustls providers out of the decision and made the FIPS
  path win when both blocking transport features are enabled.
- Added exact graph checks and runtime tests for rustls, AWS-LC-RS,
  AWS-LC-FIPS, alternate TLS/crypto exclusions, and additive features.
- Forced repository checks to compile Cargo-authenticated bundled
  AWS-LC-FIPS source instead of auto-discovering a system library.
- Documented native build trust, upstream certificate mismatch, platform
  scope, and the absence of an application or deployment compliance claim.
- Separated standard native transport CI from the dedicated Linux FIPS job.
- Fixed post-tag publishing so the signed approved commit does not rerun the
  network-sensitive release gate unless `--rerun-gate` is requested.
- Added the `v0.23.0` release gate and independent crate version plan.

## 0.22.0 - 2026-07-14

- Added six isolated libFuzzer targets for fixed-buffer writers, request
  targets, labels and DNS, pagination, action polling, and response envelopes.
- Added synthetic source-derived seed corpora and rejected generated corpora
  or crash artifacts from tracked release inputs.
- Pinned the fuzz nightly, cargo-fuzz, and libfuzzer-sys toolchain in an
  excluded non-published package with an independent lockfile.
- Added bounded CI seed replay and a documented long-run, crash replay,
  minimization, sanitization, and deterministic-regression workflow.
- Added exhaustive undersized-buffer JSON atomicity tests and adversarial
  Serde regressions for malformed, nested, oversized, duplicate, overflowing,
  and control-character upstream inputs.
- Added separate Cargo Deny, RustSec, and SPDX SBOM evidence for the fuzz graph.
- Completed all SPDX package inventories from locked Cargo metadata and added
  an independent fail-closed gate for omitted native build or development
  dependencies.
- Added the `v0.22.0` release gate and independent crate version plan.

## 0.21.0 - 2026-07-14

- Added a provider-neutral executable quickstart and six compile-checked
  Hetzner examples for read-only catalog requests, mutation request building,
  pagination, action polling, DNS Zones, and Storage Boxes.
- Added workflow-oriented quickstart, Hetzner, security, and release-runbook
  guides with explicit credential, logging, timeout, retry, live-smoke, and
  mutation-cost boundaries.
- Added docs.rs all-feature metadata and complete feature tables to every
  published crate.
- Added a dependency-free repository-local Markdown and HTML link validator,
  including regression tests for multiple links, missing targets, fenced
  examples, and repository traversal.
- Added an explicit all-feature workspace doctest gate to normal checks.
- Refreshed locked transitive dependencies to `simd_cesu8 1.2.0` and
  `socket2 0.6.5` while preserving the legacy Windows dependency boundary.
- Added the `v0.21.0` release gate and independent crate version plan.

## 0.20.0 - 2026-07-13

- Added an explicit allowlisted compile matrix for representative Linux,
  Windows, FreeBSD, macOS, Android, iOS, WASM, and embedded targets.
- Checked portable crates with default no_std features and with their
  allocation-bearing or Serde feature combinations.
- Added native full-workspace transport compilation on Linux, Windows, macOS
  ARM64, and macOS x86-64 GitHub-hosted runners.
- Added a default dependency-graph gate that rejects accidental activation of
  every package outside the explicit first-party and sanitization allowlist
  across default features and all target-specific dependency branches.
- Added adversarial regression tests for target allowlisting, missing target
  libraries, unavailable or broken rustup commands, argument validation,
  command construction, and dependency leaks.
- Removed transient release-candidate status from publishable READMEs and added
  a regression gate that rejects stale crates.io release-status wording.
- Documented support tiers and explicit reqwest limitations for FreeBSD,
  Android, iOS, WASM, embedded, and future Aesynx environments.
- Added the `v0.20.0` release gate and independent crate version plan.

## 0.19.0 - 2026-07-13

- Added an opt-in read-only Hetzner live smoke harness for locations, server
  types, load balancer types, ISOs, public system images, and pricing.
- Required an exact live-mode marker and a private regular token file; direct
  token environment variables and configurable provider origins are not used.
- Added bounded token and response reads, symlink and Unix permission checks,
  opened-file identity validation, and volatile cleanup of token and response
  source buffers.
- Kept the live network test ignored by default while compiling it in normal
  workspace checks and running twelve offline policy and adversarial tests.
- Separated credential-free Cargo staging, privileged root-owned installation,
  and authenticated open-descriptor execution; adjacent user-owned digests are
  never treated as authenticity evidence.
- Added static diagnostics that omit token values, token paths, response
  bodies, and provider resource IDs.
- Documented least-privilege read-only project setup and a separate future
  destructive-test plan whose mutation execution remains disabled.
- Added the `v0.19.0` release gate and independent crate version plan.
- Bound the facade's planned previous version to the latest earlier semantic
  release tag, with regression coverage for stale release metadata.

## 0.18.0 - 2026-07-13

- Added no_std provider-neutral pagination and action polling state machines
  without hidden requests, sleeps, clocks, executors, or retry policy.
- Added hard page limits, exact transition checks, empty/repeated page
  rejection, progress-regression checks, zero-delay rejection, explicit
  cancellation/timeout decisions, and terminal provider-error propagation.
- Added validated provider-neutral rate-limit metadata to transport responses.
- Added strict all-or-none rate-limit header parsing to blocking and async
  reqwest transports and deterministic metadata propagation in the testkit.
- Added strict reusable Hetzner `meta.pagination` parsing and action response
  conversion for the provider-neutral helpers.
- Corrected the source-locked Hetzner page default and maximum to 25 and 50.
- Required advertised previous and next page numbers to be exactly adjacent.
- Rejected premature known-last termination, entries above `per_page`, and
  page counts or continuation state inconsistent with supplied totals.
- Bound cursor page size and first-response total/last metadata across each
  traversal so snapshot changes fail before advancing.
- Preserved terminal action failures ahead of non-authoritative progress
  regression checks.
- Rejected identical or conflicting duplicate rate-limit headers in blocking
  and async transports.
- Hardened live release drift fetches with explicit validating TLS contexts,
  exact non-redirecting HTTPS URLs, and regression tests for downgrade and
  redirect rejection before reading response data.
- Prepared independent v0.18 crate versions, release notes, and release gate.

## 0.17.0 - 2026-07-13

- Added the runtime-neutral no_std `AsyncTransport` contract and an
  allocation-free async mock implementation in `cloud-sdk-testkit`.
- Added the optional hardened `cloud-sdk-reqwest/async-rustls` adapter with
  explicit caller-provided Tokio execution.
- Added cancellation-safe caller-bounded response buffering and sanitized
  adapter-owned async request/response storage.
- Added deterministic loopback tests for exact requests, redirects, timeouts,
  cancellation, overflow, redaction, and downstream feature unification.
- Extended dependency, modularity, package, and default-graph gates for async
  reqwest, bytes, and Tokio without admitting them to provider defaults.
- Bound release-gate verification to one clean unchanged commit at entry and
  exit, including tracked and untracked worktree regression tests.
- Prepared independent v0.17 crate versions, release notes, and release gate.

## 0.16.0 - 2026-07-13

- Added explicit bounded content-type metadata to provider-neutral transport
  requests.
- Added the first hardened provider-neutral blocking reqwest adapter behind
  the non-default `blocking-rustls` feature.
- Required HTTPS, rustls with TLS 1.2 minimum, explicit timeouts and user
  agent, HTTP/1, system DNS, no redirects, no retries, no proxies, and no
  response decompression.
- Redacted content-type parameters and added an isolated downstream reqwest
  HTTP/2/Hickory feature-unification regression fixture.
- Added independent policy, advisory, and SPDX SBOM gates for the standalone
  feature-unification fixture lockfile.
- Added canonical CI and release-gate comparisons that reject stale committed
  SBOM evidence while ignoring only generator metadata and array ordering.
- Added redacted bearer-token ownership, sanitized adapter-owned request
  bodies, bounded response reads, payload-free failures, and failure cleanup.
- Added deterministic loopback security tests, dependency admission evidence,
  a fail-closed reqwest graph boundary, and the `v0.16.0` release gate.

## 0.15.0 - 2026-07-12

- Added provider-neutral no_std blocking transport contracts with validated
  origin-form request targets, bounded status codes, caller-owned response
  buffers, and redacted request debug output.
- Added the first usable `cloud-sdk-testkit` ordered mock transport, bounded
  fixture builders, compact oversized bodies, and six-case adversarial corpus.
- Reused the adversarial corpus in the Hetzner Serde response tests without
  adding testkit or transport dependencies to the provider's normal graph.
- Added the `v0.15.0` release gate and independent crate version plan.
- Rejected scheme-relative and backslash-containing request targets before
  future adapters can compose authenticated provider URLs.
- Bound transport responses to initialized caller-buffer slices so safe
  implementations cannot report an out-of-bounds response length.

## 0.14.0 - 2026-07-12

- Added an opt-in no_std Serde boundary for size-checked DNS RRSet request
  bodies and validated shared action/error response envelopes.
- Added a 1 MiB aggregate RRSet JSON policy, borrowed-or-owned escaped response
  text, adversarial fixtures, dependency admission evidence, and automated
  default-graph enforcement.
- Added provider-neutral volatile caller-buffer cleanup through the reviewed
  `sanitization` crate, including an early-return-safe `SecretBuffer` guard.
- Redacted borrowed API error messages and removed ordinary equality from
  Storage Box passwords, private keys, and containing request types.
- Added atomic escaped private-key JSON output without restoring raw access,
  with guarded cleanup and unchanged-buffer failure tests.
- Added the `v0.14.0` release gate, including sanitization graph isolation, and
  an independent `cloud-sdk-sanitization` `0.13.0` code release.

## 0.13.0 - 2026-07-12

- Added `cloud-sdk-hetzner::dns::rrsets` no_std request primitives for RRSet
  CRUD, repeated-type list filters, protection, TTL, and all record mutation
  actions.
- Added all 16 source-locked RR types, relative/apex/wildcard name validation,
  explicit JSON-null TTL inheritance, bounded unique record lists, and atomic
  JSON-string writers for values and comments.
- Added the `v0.13.0` release gate and independent dependency-only `0.12.1`
  boundary-crate version plan.
- Increased the bounded endpoint-path policy to cover maximum valid DNS RRSet
  action paths and added long-name regression coverage.

## 0.12.0 - 2026-07-11

- Added `cloud-sdk-hetzner::dns::zones` no_std request primitives for Zone
  CRUD, zonefile export/import, global and per-Zone action lists, global action
  lookup, primary nameserver replacement, deletion protection, and TTL changes.
- Added bounded lowercase Zone names, default TTLs, zonefiles, public primary
  nameservers, strict Base64 TSIG keys with redacted debug output, deterministic
  list queries, and fixed-buffer endpoint paths.
- Required explicit TTL values for Zone change-TTL actions while preserving the
  source-locked optional default TTL on Zone creation; the separate RRSet TTL
  deprecation remains assigned to `v0.13.0`.
- Reused the source-locked public IP policy for Load Balancer targets and DNS
  primary nameservers.
- Added the `v0.12.0` release gate script.
- Enforced canonical TSIG Base64 padding bits and documented secure erasure of
  caller-owned TSIG and zonefile output buffers after transport use.
- Updated the IANA IPv6 source lock for its shared Load Balancer and DNS
  primary-nameserver policy location.
- Replaced the unused `cloud-sdk-hetzner-sanitization` workspace placeholder
  with the provider-neutral `cloud-sdk-sanitization` crate before adoption.
- Added a fail-closed retired-package denylist so release planning, workspace
  verification, and direct publishing reject former package names.
- Replaced the unused Hetzner-specific reqwest and testkit placeholders with
  provider-neutral `cloud-sdk-reqwest` and `cloud-sdk-testkit` crates, keeping
  the architecture to one primary crate per provider.
- Enforced the future package-cardinality rule by rejecting nested names such
  as `cloud-sdk-{provider}-{boundary}` throughout release automation.
- Hardened DNS TSIG credentials to HMAC-SHA256 only, required at least 32
  decoded secret bytes, and removed ordinary equality from secret-bearing
  values and request containers.
- Replaced pentest key/signature attestations with the `eth` workspace's
  report-only release model: the final report commit must directly follow the
  reviewed commit and may change no other path.

## 0.11.0 - 2026-07-11

- Added `cloud-sdk-hetzner::cloud::load_balancers` no_std request primitives
  for Load Balancer CRUD, metrics, services, targets, networks, reverse DNS,
  protection, algorithms, type changes, and public-interface actions.
- Added protocol-safe HTTP/HTTPS service settings, bounded health checks,
  mutually exclusive target selection, public server-IP validation, explicit
  reverse-DNS set/reset intent, and deterministic multi-metric queries.
- Added the `v0.11.0` release gate script.
- Made JSON-string writes atomic so undersized buffers retain no password or
  cloud-init prefix.
- Enforced complete pinned OpenAPI hashes before parsing, bounded remote spec
  size and download time, and added regression tests for both controls.
- Bound `v0.11.0+` pentest evidence to release-sensitive content, made release
  tags mandatory for publishing, and removed normal publisher bypass flags.
- Tightened public IPv6 server targets to ordinary global-unicast ranges and
  rejected translated, transition, benchmarking, documentation, deprecated,
  and IETF-reserved address space.
- Required detached OpenSSH pentest signatures from an approved key distinct
  from the release signer and expanded post-review binding to all
  release-consumed source-lock and security documentation.
- Applied regular-file and size checks to local as well as fetched OpenAPI
  inputs before hashing and parsing.
- Source-locked public IPv6 server targets to IANA allocations and rejected
  reserved gaps such as `3000::/5` and `3ffe::/16`.
- Removed pentest-evidence and local OpenAPI pathname races with no-follow,
  bounded descriptor reads and authenticated private snapshots, and made
  pentest signature publication atomic and no-overwrite.
- Added a fail-closed IANA IPv6 registry drift checker that keeps the
  machine-readable allocation lock and Rust policy synchronized and runs live
  from the release gate.
- Bound pentest signatures to an immutable committed report blob through
  signed commit/path/SHA-256 metadata, rejected unrepresentable IPv6 prefixes,
  and authenticated IANA registry bytes before parsing.
- Published pentest attestation metadata and its OpenSSH signature as one
  bounded, atomically installed evidence bundle.

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
  sanitization responsibility for password output buffers.
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
