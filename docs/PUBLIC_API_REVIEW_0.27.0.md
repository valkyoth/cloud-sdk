# v0.27 Public API Review

Review date: 2026-07-15

## Scope

This review covers every published first-party crate, its default and optional
features, request constructors, public error families, and credential-bearing
transport setup. It is a stabilization review, not a claim that typed response
models or an end-to-end Hetzner client are complete.

## Findings And Decisions

- Default features remain empty for every crate. The facade, provider,
  sanitization, and testkit default graphs remain `no_std`; transport, Serde,
  allocation, and standard-library integration remain opt-in.
- The one-crate-per-provider rule remains intact. Hetzner Cloud, DNS, Console
  Storage Box, and future Robot modules stay in `cloud-sdk-hetzner`.
- Required provider request inputs are direct validated constructor arguments.
  `Option` remains only for optional, nullable, resettable, or tri-state API
  semantics. Cross-field validation remains fallible.
- Public first-party error families implement payload-free `Display` and
  `core::error::Error`. Static messages do not interpolate credentials,
  endpoints, targets, bodies, provider messages, or customer data.
- A caller-selected HTTPS endpoint is explicitly constructed through
  `HttpsEndpoint::new_custom`. The endpoint is a bearer-token destination and
  must come only from trusted operator configuration.
- Capability documentation separates request models, path/query encoding, body
  serialization, response models, and end-to-end client coverage. Complete
  request-operation coverage does not imply complete execution coverage.
- `ServerQueryError` is retained for compatibility despite its historical name;
  it is a bounded query writer, not an error value. A future rename should wait
  for a broader provider request-preparation API rather than add churn now.

## Feature Review

| Crate | Default | Optional reviewed boundaries |
| --- | --- | --- |
| `cloud-sdk` | empty | `alloc`, `std` |
| `cloud-sdk-hetzner` | empty | `alloc`, `serde`, `std`; live smoke is a Serde-gated integration test |
| `cloud-sdk-reqwest` | empty | blocking rustls, deterministic roots, blocking FIPS, async rustls |
| `cloud-sdk-sanitization` | empty | `std` |
| `cloud-sdk-testkit` | empty | `std` |

Feature unification must not activate transport or operating-system
dependencies in any default graph. Existing dependency-boundary and platform
matrix checks enforce this rule.

## Deferred Work

- Shared request preparation and operation safety metadata: `v0.29.0`.
- Complete checked body serialization: `v0.30.0`.
- Typed success and error response decoding: `v0.31.0`.
- Safe official-endpoint Hetzner client facade: `v0.41.0`, after the
  provider-neutral architecture and typed-operation milestones.
- Shared concurrent transport and credential rotation: `v0.28.0`.

The exact constructor and endpoint migrations are documented in
[`MIGRATION_0.27.0.md`](MIGRATION_0.27.0.md).
