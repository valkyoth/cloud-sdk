# Prepared Coverage Checker Dependency Admission

Status: admitted only in the excluded, non-published
`tools/prepared-coverage-check` package.

Checked: 2026-07-15.

## Packages

| Component | Version | Role | License | Rust |
| --- | --- | --- | --- | --- |
| `syn` | `2.0.119` | Rust source and macro-input parser | MIT OR Apache-2.0 | 1.71 |
| `proc-macro2` | `1.0.106` | transitive token representation | MIT OR Apache-2.0 | 1.68 |
| `unicode-ident` | `1.0.24` | transitive identifier tables | Unicode-3.0 | 1.71 |

`cargo search syn --limit 1` and `cargo info syn@2.0.119` confirmed the
current release, license, and compiler floor on 2026-07-15. The checker pins
the exact `syn` version and disables default features, enabling only `full`
and `parsing`. It does not admit `quote`, a procedural macro, network access,
or native code.

## Isolation

The checker has `publish = false`, an independent lockfile, and its own empty
workspace. The root workspace explicitly excludes it. No published crate,
default feature, provider, transport, example, or build script depends on the
checker or its dependencies.

The release script invokes it only through
`scripts/check_prepared_operation_coverage.py`. It parses bounded local Rust
sources and emits operation identifiers; it does not read credentials,
environment configuration, sockets, or provider responses.

## Security Decision

The previous Python scanner did not implement nested Rust comments,
`cfg_attr`, raw strings, or complete expression semantics. `syn` is admitted
because release-integrity evidence must follow the same lexical and AST rules
as Rust source.

The checker accepts operation evidence only from top-level, unqualified
`endpoint_wire!`, `body_wire!`, and `body_component!` item macros or explicit
implementations using the canonical `crate::prepared::EndpointWire` and
`crate::prepared::BodyWire` paths. It requires exactly one of each applicable
reviewed macro definition in the separately enumerated root files and compares
their parsed delimiter and token structure with locks under
`tools/prepared-coverage-check/locks`. Duplicate, missing, modified, or no-op
definitions fail closed.

Inline modules cannot provide evidence, while file-level or item-level `cfg`
and `cfg_attr`, imports, aliases, glob imports, `macro_use`, local adapter
definitions, and namespaced adapter calls fail closed.

Endpoint mappings must be match arms returning string literals. Conditional
items, helper expressions, discarded literals, unknown operations, and
ambiguous mappings also fail closed.

## Verification

- `cargo clippy --manifest-path tools/prepared-coverage-check/Cargo.toml --locked --all-targets -- -D warnings`
- `cargo test --manifest-path tools/prepared-coverage-check/Cargo.toml --locked`
- `scripts/test-prepared-operation-coverage.py`
- `cargo deny --manifest-path tools/prepared-coverage-check/Cargo.toml --config deny.toml --locked check advisories licenses sources`
- `cargo audit --no-fetch --file tools/prepared-coverage-check/Cargo.lock`
- `scripts/check_sbom_freshness.sh`
