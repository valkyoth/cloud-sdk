# cloud-sdk Modularity Policy

`cloud-sdk` must not become a monolithic source tree.

Rules:

- `cloud-sdk` is provider-neutral and stays small.
- `cloud-sdk-hetzner` is the Hetzner endpoint implementation home.
- Hetzner Cloud resources, DNS resources, security resources, and Storage Box
  resources live in separate modules under the Hetzner provider crate.
- Reusable transport adapters, mock transports, test infrastructure, and
  secret-sanitization helpers belong in provider-neutral boundary crates.
- Provider-specific authentication, models, errors, and fixtures stay in the
  provider crate; do not create transport or testkit crates per provider.
- Keep `lib.rs` as module wiring and public API shape.
- Non-generated Rust source files must stay under 500 lines.
- A file approaching 300 lines should be reviewed for splitting.
- Adapter crates may depend inward on the stable SDK crate; the SDK crate must
  not depend outward on adapters.
- Each provider should use one primary provider crate unless a future boundary
  has a documented technical reason that cannot be shared.
- Feature flags must not silently enable networking, TLS, filesystem, clocks,
  token storage, or async runtimes.
- `std` paths are prohibited except under explicitly reviewed optional
  `cloud-sdk-reqwest` blocking, async, shared-policy, and test modules, plus the
  ignored `cloud-sdk-hetzner` live-smoke integration test. The live harness may
  use provider-neutral adapters only through dev dependencies and must never
  enter the provider's normal graph. Public blocking and async modules remain
  guarded by their reviewed non-default `blocking-rustls`,
  `blocking-rustls-webpki-roots`, `blocking-rustls-fips`, and `async-rustls`
  features.

The local gate is:

```bash
scripts/validate-modularity-policy.sh check
```

## Provider Crate Cardinality

- Each provider has exactly one primary package named `cloud-sdk-{provider}`.
- Provider identifiers use one package-name segment, such as `hetzner`, `ovh`,
  or `scaleway`.
- Provider API families, authentication, models, errors, and fixtures use
  modules or narrowly reviewed features inside that provider package.
- Cross-provider capabilities use one neutral package named for the capability,
  such as `cloud-sdk-reqwest`, `cloud-sdk-testkit`, or
  `cloud-sdk-sanitization`.
- Nested package names such as `cloud-sdk-ovh-dns`,
  `cloud-sdk-scaleway-reqwest`, or `cloud-sdk-hetzner-testkit` are prohibited.

`scripts/release_crates.py` rejects nested `cloud-sdk` package names in its
publish order, release plan, Cargo workspace metadata, and final publish call.
An exception requires a documented architectural constraint, an explicit
policy and validator change, release notes, and a new pentest-reviewed commit.
