# cloud-sdk Release Plan To 1.0

Status: planning document

This plan is intentionally granular. The SDK manages infrastructure APIs, so
each milestone must be small enough to review, test, pentest, and stop cleanly
before tagging.

Tags use:

```text
v0.N.0      milestone release
v0.N.P      patch/fix release for milestone N
v1.0.0      first serious production-ready cloud-sdk foundation and Hetzner provider
v1.1.0+     post-1.0 Robot Webservice support track
```

## Release Principles

Every release must have:

- a clear definition of done;
- a local verification command;
- security review notes;
- known limitations;
- release notes;
- dependency-policy evidence;
- source-lock evidence for API behavior;
- completed pentest evidence for the exact implementation commit being
  reviewed;
- no hidden dependency on one developer machine.

## Pentest Before Tags

Every version must pass a security review and pentest before it is tagged. This
applies to `v0.N.P` patch tags as well as milestone tags.

A version is not tag-ready until:

- `scripts/checks.sh` passes;
- `cargo deny check` passes;
- `cargo audit` passes;
- `scripts/generate-sbom.sh` succeeds;
- release notes exist at `release-notes/RELEASE_NOTES_X.Y.Z.md`;
- a pentest report exists at `security/pentest/vX.Y.Z.md`;
- the pentest report names the exact full 40-character `Reviewed-Commit:`;
- the pentest report has `Status: PASS`;
- the pentest report has non-blank `Tester:` and `Scope:` fields;
- the pentest report has a `Date: YYYY-MM-DD` field;
- `sbom/cloud-sdk.spdx.json` exists and is non-empty.

## Crate Versioning And Publish Order

Provider-neutral domains live in `cloud-sdk`. Hetzner endpoint models live in
`cloud-sdk-hetzner`. Extra provider-specific crates are versioned only for real
optional boundaries: reqwest transport, testkit fixtures, and secret
sanitization.

Track every release in `release-crates.toml` and
`docs/CRATE_VERSION_MATRIX.md`.

## Milestones

### v0.1.0 - Repository Foundation

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.96.1` pinned.
- Rust `1.90.0` through `1.96.1` compatibility policy.
- One provider-neutral no_std crate, one focused Hetzner provider crate, and
  three optional Hetzner boundary crates.
- CI, dependency policy, security policy, release notes.
- Implementation, release, API, threat-model, modularity, toolchain, unsafe,
  and supply-chain docs.

Verification:

- `scripts/checks.sh`
- `scripts/release_0_1_gate.sh`

### v0.2.0 - Official API Source Lock

Goal: pin the authoritative current Hetzner API source before endpoint models.

Deliverables:

- Official OpenAPI/spec-source discovery.
- `docs/SPEC_LOCK.md` with retrieval date, source URL, and revision/hash when
  available.
- Complete `docs/API_MATRIX.md` endpoint table with method, path, resource
  owner module, pagination, sorting, action behavior, and implementation status.
- Explicit Storage Boxes review because the current changelog shows active
  Storage Box API changes that were not in the original prompt endpoint list.
- Drift-check script skeleton.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_upstream.sh --local-only`

### v0.3.0 - Core Request And Response Policy

Goal: no_std request path, query, label, sorting, pagination, error, and
rate-limit domains.

Deliverables:

- HTTP method and path domains.
- Bounded query parameter builder.
- Label and label-selector validation.
- Pagination and sorting policy.
- Error envelope and rate-limit metadata.
- Action status model.

### v0.4.0 - Read-Only Catalog Resources

Goal: implement low-risk read-only resources first.

Deliverables:

- Locations.
- Pricing.
- Server types.
- Load balancer types.
- ISOs.
- Image list/get.

### v0.5.0 - Security Resources

Goal: model SSH keys and certificates safely.

Deliverables:

- SSH key list/create/get/update/delete domains.
- Certificate list/create/get/update/delete domains.
- Certificate retry action domain.
- Redacted debug output for secret-adjacent values.

### v0.6.0 - Server Resource Models

Goal: server CRUD, metrics request domains, and server action request domains.

### v0.7.0 - Image, Placement Group, Primary IP Models

Goal: complete remaining server-adjacent resource models.

### v0.8.0 - Volume And Floating IP Models

Goal: volume and floating IP resources plus actions.

### v0.9.0 - Storage Box Models

Goal: Storage Boxes, Storage Box actions, subaccounts, and source-locked
storage-specific constraints.

### v0.10.0 - Firewall And Network Models

Goal: firewall, firewall actions, network, and network actions.

### v0.11.0 - Load Balancer Models

Goal: load balancers, metrics, services, targets, algorithms, network attach,
public interface actions, and type changes.

### v0.12.0 - DNS Zone Models

Goal: zones, zonefile import/export, zone actions, TTL and nameserver policy.

### v0.13.0 - DNS RRSet Models

Goal: RRSets, RRSet actions, record set mutation helpers, and validation.

### v0.14.0 - Optional Serde Boundary

Goal: admit optional `serde` derives and parser tests outside default no_std.

### v0.15.0 - Mock Transport And Testkit

Goal: deterministic mock transport, pagination/action fixtures, and adversarial
response corpus.

### v0.16.0 - Optional Blocking Transport Adapter

Goal: first reviewed transport adapter outside the default graph.

### v0.17.0 - Optional Async Transport Adapter

Goal: async transport adapter with explicit runtime neutrality.

### v0.18.0 - Pagination And Action Polling Helpers

Goal: ergonomic optional helpers over transport traits without hiding rate-limit
or timeout policy.

### v0.19.0 - Live Smoke Harness

Goal: opt-in live tests gated by environment variables and least-privilege test
project guidance.

### v0.20.0 - Platform Matrix

Goal: Linux, Windows, BSD, macOS, Android, iOS, WASM, and embedded/no_std
compatibility evidence for claimed crates.

### v0.21.0 - Documentation And Examples Hardening

Goal: docs.rs examples, transport examples, security recipes, and release
runbook completeness.

### v0.22.0 - Fuzzing And Adversarial Tests

Goal: fuzz parsers/builders and add malformed API response corpus.

### v0.23.0 - Dependency And Tooling Hardening

Goal: current dependency/tool review, SBOM, cargo-deny/cargo-audit evidence,
and supply-chain docs.

### v0.24.0 - API Drift Automation

Goal: automated upstream drift signal for Hetzner API docs/spec changes.

### v0.25.0 - Release Candidate Cleanup

Goal: final public API review, deprecation policy, examples, docs, and
semver-readiness audit.

### v1.0.0 - Production SDK

Goal: first serious production-ready `cloud-sdk` foundation and Hetzner
Cloud/DNS provider.

Exit criteria:

- Complete claimed endpoint coverage.
- Default graph remains no_std and transport-free.
- Optional transport adapters have security and dependency evidence.
- API drift process is documented.
- Live and mock tests cover critical workflows.
- Pentest, SBOM, cargo-deny, cargo-audit, and CI evidence pass.
- Provider-neutral naming and module patterns are documented for later crates
  such as `cloud-sdk-cloudflare`.

### v1.1.0 - Robot Webservice Source Lock

Goal: start Robot Webservice support without changing the 1.0 Cloud/DNS
contract.

Deliverables:

- Source-lock <https://robot.hetzner.com/doc/webservice/en.html>.
- Document Robot as a distinct API surface with HTTP Basic Auth, HTTPS-only
  transport, form-encoded POST parameters, JSON/YAML response modes, and
  Robot-specific errors/rate limits.
- Add a `robot` module plan for server, IP, subnet, reset, failover, wake-on-LAN,
  boot configuration, reverse DNS, traffic, SSH keys, server ordering, Robot
  storage box, firewall, and vSwitch operations.
- Decide whether Robot implementation lives in the main SDK crate modules only
  or also gets an optional adapter/helper crate.

Verification:

- `scripts/checks.sh`
- Robot source-lock drift check.

### v1.2.0+ - Robot Webservice Implementation

Goal: implement Robot Webservice operations in small reviewed passes and expose
them through `cloud-sdk-hetzner`.

Exit criteria:

- Robot support has separate auth, encoding, error, rate-limit, fixture, and
  live-test evidence.
- Cloud/DNS default behavior remains unchanged.
- Robot docs clearly separate Cloud API tokens from Robot webservice users.
