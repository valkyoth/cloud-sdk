# Supply Chain Security

Dependencies are denied by default until reviewed. New dependencies require:

- latest-version check;
- license review against `deny.toml`;
- maintenance and ownership review;
- default-feature review;
- no hidden `std`, transport, TLS, filesystem, clock, process, native-code, or
  secret-storage expansion in the main SDK default graph;
- tests for the behavior being admitted;
- documentation under `docs/dependency-admission-*.md`.

Every tag requires `cargo deny check`, `cargo audit`, and current SBOM evidence.
Every direct third-party workspace dependency uses an exact `=X.Y.Z`
requirement. `scripts/check_exact_dependency_pins.py` rejects broader or
unversioned direct requirements before lockfile and package evidence can pass.
Intermediate pre-1.0 stages prohibit crates.io publication. At every
fifth-minor or exceptional public checkpoint, the publisher compares package
trees cumulatively with the previous public tag, publishes changed
dependencies first, and publishes the facade last. Standalone test/tooling
workspaces compiled by release CI require their own locked policy check,
advisory audit, and SBOM.
`scripts/check_sbom_freshness.sh` regenerates both inventories and compares
canonical content with the committed evidence. It ignores only the generated
creation timestamp, random document namespace, and array ordering.

Serde `1.0.229` is the first admitted optional third-party normal dependency.
Its defaults are disabled and its `alloc` and `derive` features are enabled only
by `cloud-sdk-hetzner/serde`. serde_json `1.0.151` is an optional normal
dependency enabled only by `cloud-sdk-hetzner/serde`; its defaults and `std`
feature remain disabled. The full decision and transitive surface are recorded in
[`dependency-admission-serde.md`](dependency-admission-serde.md), and
`scripts/check_serde_boundary.sh` enforces graph isolation.

Exact `base64-ng 2.0.1`, `md-5 0.11.0`, and `sha2 0.11.0` are admitted only
through the non-default provider Serde boundary. A bounded first-party parser
uses them to structurally validate OpenSSH/RFC 4253 public keys, bind Hetzner's
legacy response fingerprint, and derive SHA-256 identity without creating a
second owned key model. Defaults are disabled, and MD5 is not admitted for new
security decisions. The complete graph and limitations are recorded in
[`dependency-admission-ssh-public-key.md`](dependency-admission-ssh-public-key.md),
while `scripts/check_security_response_models.sh` enforces graph isolation.

The first-party `sanitization` `2.0.3` crate is admitted only through
`cloud-sdk-sanitization`, with default features disabled and no transitive
runtime dependencies. The decision and limits are recorded in
[`dependency-admission-sanitization.md`](dependency-admission-sanitization.md),
and `scripts/check_sanitization_boundary.sh` enforces graph isolation.
Since v0.38, `cloud-sdk-sanitization` is published before and is a mandatory
dependency of `cloud-sdk`; this puts all first-party core cleanup behind the
single audited primitive while preserving a `no_std`, allocation-free default
graph. Provider, testkit, and transport boundaries receive it through core;
features still gate owned secret storage and transport implementations.
The v0.32 major-version review is recorded in
[`DEPENDENCY_REVIEW.md#v0320`](DEPENDENCY_REVIEW.md#v0320).
The v0.33 no-external-change freshness review is recorded in
[`DEPENDENCY_REVIEW.md#v0330`](DEPENDENCY_REVIEW.md#v0330).
From v0.45 onward, `scripts/check_dependency_review.py` compares the root
`Cargo.lock` with the previous release tag and fails unless every changed
package version has an exact row in the explicitly selected current-release
section of the versioned dependency review. Historical rows cannot satisfy the
current gate.
The v0.34 endpoint-policy no-external-change review is recorded in
[`DEPENDENCY_REVIEW.md#v0340`](DEPENDENCY_REVIEW.md#v0340).
The v0.35 canonical request-target no-external-change review is recorded in
[`DEPENDENCY_REVIEW.md#v0360`](DEPENDENCY_REVIEW.md#v0360).

Reqwest `0.13.4` is admitted only through the non-default
`cloud-sdk-reqwest/blocking-rustls`, `blocking-rustls-webpki-roots`,
and `async-rustls` features. Bytes `1.12.1`
is a direct optional dependency only for sanitized async request-body ownership.
Reqwest defaults are disabled;
native TLS, response decompression, proxies, redirects, and retries are not
admitted by policy. The full HTTP, Tokio, URL, rustls, platform-verifier,
aws-lc, license, duplicate-version, and transitive-zeroize review is recorded
in [`dependency-admission-reqwest.md`](dependency-admission-reqwest.md).
`scripts/check_reqwest_boundary.sh` keeps reqwest, bytes, and Tokio outside
every default/provider graph, validates the blocking and async feature graphs
separately, and rejects direct first-party `zeroize` dependencies.
The locked downstream feature-unification fixture is audited independently and
has its own SPDX SBOM; its exact target-specific duplicate dependency is
documented in the reqwest admission record.

The deterministic blocking feature admits `webpki-roots 1.0.9` with defaults
disabled and supplies that compiled Mozilla snapshot to a complete custom
rustls client configuration. Its trust-store and update tradeoffs, exact
archive checksums, direct dependency freshness review, and native AWS-LC build
review are recorded in the original
[`DEPENDENCY_REVIEW.md#v0240`](DEPENDENCY_REVIEW.md#v0240) and current
[`DEPENDENCY_REVIEW.md#2026-07-20`](DEPENDENCY_REVIEW.md#2026-07-20).

FIPS dependencies and transport APIs are excluded from the active workspace.
The previous AWS-LC experiment remains recorded as historical evidence in
[`dependency-admission-reqwest-fips.md`](dependency-admission-reqwest-fips.md),
while [`FIPS_DEFERMENT.md`](FIPS_DEFERMENT.md) defines the conditions for any
future Brynja integration. Repository checks reject accidental reintroduction.

The opt-in Hetzner live harness separates build, privileged sealing, and
credential phases. Cargo and all build-time dependencies run only while the
token is absent or unmounted and no token-file variable is exported. Build
output remains explicitly untrusted until an administrator installs the staged
bundle into root-owned non-writable system paths with trusted absolute tools.
The root-owned authenticated runtime validates the installation and hashes and
executes the same open descriptor under a minimal environment; it never invokes
Cargo. Root ownership, not an attacker-replaceable adjacent digest, is the local
authenticity trust anchor. Detached signature provenance is not claimed.
