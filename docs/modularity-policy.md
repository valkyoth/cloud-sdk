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

The local gate is:

```bash
scripts/validate-modularity-policy.sh check
```
