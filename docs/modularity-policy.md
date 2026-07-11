# cloud-sdk Modularity Policy

`cloud-sdk` must not become a monolithic source tree.

Rules:

- `cloud-sdk` is provider-neutral and stays small.
- `cloud-sdk-hetzner` is the Hetzner endpoint implementation home.
- Hetzner Cloud resources, DNS resources, security resources, and Storage Box
  resources live in separate modules under the Hetzner provider crate.
- Provider-specific transport adapters and test fixtures may live in separate
  crates when they admit optional dependencies. Reusable secret-sanitization
  helpers belong in a provider-neutral boundary crate.
- Keep `lib.rs` as module wiring and public API shape.
- Non-generated Rust source files must stay under 500 lines.
- A file approaching 300 lines should be reviewed for splitting.
- Adapter crates may depend inward on the stable SDK crate; the SDK crate must
  not depend outward on adapters.
- Feature flags must not silently enable networking, TLS, filesystem, clocks,
  token storage, or async runtimes.

The local gate is:

```bash
scripts/validate-modularity-policy.sh check
```
