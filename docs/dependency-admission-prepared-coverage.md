# Prepared Coverage Checker Dependency Admission

Status: admitted only in the excluded, non-published
`tools/prepared-coverage-check` package.

Checked: 2026-09-24.

## Packages

| Component | Version | Role | License | Rust |
| --- | --- | --- | --- | --- |
| `syn` | `3.0.6` | Rust source and macro-input parser | MIT OR Apache-2.0 | 1.71 |
| `saphyr` | `0.1.0` | bounded YAML document parser | MIT OR Apache-2.0 | 1.85.0 |
| `saphyr-parser` | `0.1.0` | pre-DOM YAML event validation | MIT OR Apache-2.0 | 1.85.0 |
| `proc-macro2` | `1.0.107` | transitive token representation | MIT OR Apache-2.0 | 1.68 |
| `unicode-ident` | `1.0.26` | transitive identifier tables | Unicode-3.0 | 1.71 |

The direct pins disable default features. `syn` explicitly enables `full`,
`parsing`, and `visit`; the YAML dependencies enable no direct features.
The resolved tool graph also includes `thiserror`/`thiserror-impl`, `quote`,
and additional unified Syn derive/printing features. These execute as build
tooling, never as a published SDK dependency. No network or native-code
capability is intentionally provided by this tool.

| Direct package | Version | Registry checksum |
| --- | --- | --- |
| `syn` | `3.0.6` | `8593e8e72159ed2257d083c7a454a85cbf854f37a0966d8d483aff8c8a3ebcee` |
| `saphyr` | `0.1.0` | `79830a82cc4eeea33aa46346f910d7cc1e576249a6fbe2fefab6ba89eba8a203` |
| `saphyr-parser` | `0.1.0` | `7429158804c36e705d9423b7848018e535b3e92d439a763e364ee12670ef473b` |

`python3 scripts/check_admission_evidence.py` verifies these exact direct
pins, feature selections and checksums against the manifest and isolated lock.

## Isolation

The checker has `publish = false`, an independent lockfile, and its own empty
workspace. The root workspace explicitly excludes it. No published crate,
default feature, provider, transport, example, or build script depends on the
checker or its dependencies.

Repository gates invoke its prepared-operation, fail-closed test, and workflow
policy checkers. They parse bounded local Rust and YAML sources, not credentials
or provider responses. YAML event validation rejects anchors and aliases before
building a document and enforces resource limits; workflow policy regressions
remain mandatory. No published SDK runtime uses this parser.

Scaleway inventory tooling also uses the isolated `source-yaml-json` binary to
parse public documentation snapshots offline. It consumes at most 10 MiB,
500,000 parser events, and 64 nesting levels, rejects duplicate/merge keys,
anchors, aliases, explicit tags, non-string mapping keys and multiple documents,
and emits JSON only after successful parsing. The caller imposes a subprocess
deadline; the parser never fetches references. No dependencies or features were
added for this use. Raw upstream sources remain separate digest-bound evidence.
Numeric values retain their raw JSON-compatible spelling without machine-number
rounding; YAML-only numeric syntax is rejected. The inventory caller retains
decimal precision with Python's standard-library `Decimal` decoder.

## Security Decision

The previous Python scanner did not implement nested Rust comments,
`cfg_attr`, raw strings, or complete expression semantics. `syn` is admitted
because release-integrity evidence must follow the same lexical and AST rules
as Rust source.

The checker anchors evidence to canonical, unattributed module edges from
`cloud-sdk-hetzner/src/lib.rs` through `prepared.rs` to `endpoints.rs` and
`bodies.rs`. The public `prepared` edge and private endpoint/body edges must be
external declarations with their exact expected visibility. Redirected,
conditional, inline, duplicate, missing, or substituted parent edges fail
closed.

Before source inspection, locked and offline Cargo metadata must bind the exact
`cloud-sdk-hetzner/Cargo.toml` package to one library target whose source is the
same canonical `src/lib.rs`. Missing or ambiguous packages and library targets,
disabled automatic libraries, malformed metadata, and `[lib] path` redirects
fail closed.

The checker accepts operation evidence only from top-level, unqualified
`endpoint_wire!`, `body_wire!`, and `body_component!` item macros or explicit
implementations using the canonical `crate::prepared::EndpointWire` and
`crate::prepared::BodyWire` paths. A source file is inspected only when its
root has one unconditional external `mod name;` declaration and the directory
contains exactly the corresponding regular `name.rs` file. Missing, orphaned,
duplicate, attributed, redirected, inline, public, and noncanonical module
declarations fail closed.

Manual `QueryWire` implementations are also restricted to the canonical
`crate::prepared::QueryWire` path. Any `accepts_operation` override on a manual
query or body implementation must have the exact
`fn(self, operation_key: &str) -> bool` signature and one explicit match whose
scrutinee is that parameter. The first arm must map one or more string literals
to `true`, and the final wildcard arm must map to `false`. Literal, member-call,
constant, renamed, or otherwise substituted scrutinees fail closed.

Reserved endpoint, query, and body trait implementations may contain only
unattributed methods. Associated-item macros, verbatim or unsupported syntax,
associated constants and types, and attributes on any associated item fail
closed. Parent-defined macros, built-in expansion such as `include!`, and
procedural attributes therefore cannot generate methods after the checker has
extracted compatibility or operation evidence.

The checker requires exactly the reviewed module-scope macro definitions in
the two roots and compares their parsed delimiter and token structure with
locks under `tools/prepared-coverage-check/locks`. Duplicate, missing,
modified, or no-op definitions fail closed.

Inline modules cannot provide evidence, while file-level or item-level `cfg`
and `cfg_attr`, imports, aliases, glob imports, `macro_use`, local adapter
definitions, and namespaced adapter calls fail closed.

Only inner documentation attributes are allowed on prepared source files, and
every module-scope item must be unattributed. Each top-level item is traversed
recursively. Every type, expression, and writer path parsed from the five
admitted adapter macros is retained and inspected, including anonymous
constants and generic arguments. `impl_endpoint_prepare!` accepts only a
nonempty, recursively inspected type list. Nested items and statement-position
macros in functions, constant blocks, wire methods, adapter types, writer
paths, or adapter expressions fail closed.

Expression, type, and pattern macros are opaque to `syn` and therefore all fail
closed. Compatibility and policy expressions use explicit Rust matches instead
of macro expansion. Parent-defined macros, imported aliases, procedural
attributes, derive macros, and local implementations therefore cannot change or
hide global wire behavior outside the inspected syntax. Unparsed module items
are also forbidden.

The provider crate narrowly allows Clippy's `match_like_matches_macro` lint
because applying that style suggestion would violate this security boundary.
The syntax-aware release gate, rather than the style lint, remains authoritative
for every prepared evidence file.

Every module-scope macro invocation is also allowlisted. Endpoint and body
adapters plus the two reviewed endpoint helper macros are accepted; an
unreviewed macro cannot expand into a shadowing adapter definition.

Adapter macro invocations, canonical trait implementations, operation-key
methods, accepted-operation methods, and counted match arms must have no
attributes. This prevents procedural attributes from erasing or replacing
evidence after the syntax-aware checker has counted it.

Each evidence method must consist of exactly one tail expression. Attributes
are rejected recursively throughout accepted operation expressions, including
macro-provided mappings. Earlier returns or statements and attributed tail
expressions therefore cannot make compiled behavior differ from counted
evidence.

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
