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

Release gates require `cargo deny check`, `cargo audit`, and an SBOM before
tagging. Standalone test/tooling workspaces compiled by release CI require
their own locked policy check, advisory audit, and SBOM.
`scripts/check_sbom_freshness.sh` regenerates both inventories and compares
canonical content with the committed evidence. It ignores only the generated
creation timestamp, random document namespace, and array ordering.

Serde `1.0.228` is the first admitted optional third-party normal dependency.
Its defaults are disabled and its `alloc` and `derive` features are enabled only
by `cloud-sdk-hetzner/serde`. serde_json `1.0.150` is dev-only. The full decision
and transitive surface are recorded in
[`dependency-admission-serde.md`](dependency-admission-serde.md), and
`scripts/check_serde_boundary.sh` enforces graph isolation.

The first-party `sanitization` `1.2.4` crate is admitted only through
`cloud-sdk-sanitization`, with default features disabled and no transitive
runtime dependencies. The decision and limits are recorded in
[`dependency-admission-sanitization.md`](dependency-admission-sanitization.md),
and `scripts/check_sanitization_boundary.sh` enforces graph isolation.

Reqwest `0.13.4` is admitted only through the non-default
`cloud-sdk-reqwest/blocking-rustls` and `async-rustls` features. Bytes `1.12.1`
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
