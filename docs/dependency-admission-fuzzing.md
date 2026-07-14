# Fuzzing Dependency Admission

Status: admitted only in the excluded, non-published `fuzz/` package.

Checked: 2026-07-14.

## Packages And Tools

| Component | Version | Role | License |
| --- | --- | --- | --- |
| `cargo-fuzz` | `0.13.2` | installed test tool | MIT OR Apache-2.0 |
| `libfuzzer-sys` | `0.4.13` | fuzz runtime and build wrapper | MIT OR Apache-2.0, and NCSA |
| `arbitrary` | `1.4.2` | transitive byte-input support | MIT OR Apache-2.0 |
| Rust nightly | `nightly-2026-07-13` | sanitizer and libFuzzer compiler support | Rust toolchain licenses |

The SDK already admits `serde_json 1.0.150` for response-boundary tests. The
fuzz package reuses that exact locked version.

## Isolation Decision

`fuzz/` is excluded from the root workspace, has `publish = false`, and uses a
separate lockfile. No published crate depends on `libfuzzer-sys`, and no
default, alloc, Serde, transport, or all-feature SDK graph activates it.
Nightly Rust is required only for this harness; supported SDK compilers remain
stable Rust 1.90.0 through 1.97.0.

`libfuzzer-sys` compiles and links LLVM libFuzzer support through its build
dependencies, including `cc`. This native build surface is acceptable only in
the isolated test harness. It must never be moved into a published package,
build dependency, feature, example, or normal workspace member.

## Security Boundaries

- Fuzz targets call pure validation, writing, state-machine, and parsing APIs.
- Targets do not read credentials, environment variables, files, or sockets.
- Committed seeds are synthetic and source-derived.
- Generated corpora and artifacts are ignored and rejected if tracked.
- Smoke runs copy seeds into a temporary writable directory.
- Input length and per-input timeout are bounded.
- Exact versions are locked and checked with Cargo Deny and RustSec.
- SPDX generation is completed from locked Cargo metadata and independently
  checked to include every runtime, target, development, and native build
  package, including the complete `libfuzzer-sys` C build chain.
- The dedicated CI job compiles all targets with sanitizer instrumentation.

Fuzzing can demonstrate crashes and violated assertions for explored inputs; it
cannot prove absence of defects. Deterministic unit tests, source locks,
platform checks, dependency review, pentest, CodeQL, and release gates remain
required.

## Automated Enforcement

- `scripts/check_fuzz_harness.sh --metadata`
- `scripts/check_fuzz_harness.sh --build`
- `scripts/check_fuzz_harness.sh --smoke`
- `cargo deny --manifest-path fuzz/Cargo.toml --config deny.toml --locked
  check advisories licenses sources`
- `cargo audit --no-fetch --file fuzz/Cargo.lock`

The v0.22 release gate runs all of these checks.
