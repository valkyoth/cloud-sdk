# Toolchain Policy

The workspace develops on stable Rust `1.97.1` and declares MSRV `1.92`.

Compatibility must be maintained for:

| Rust | Requirement |
| --- | --- |
| `1.92.0` | `cargo +1.92.0 check --workspace --all-features` |
| `1.93.0` | `cargo +1.93.0 check --workspace --all-features` |
| `1.94.0` | `cargo +1.94.0 check --workspace --all-features` |
| `1.95.0` | `cargo +1.95.0 check --workspace --all-features` |
| `1.96.0` | `cargo +1.96.0 check --workspace --all-features` |
| `1.96.1` | `cargo +1.96.1 check --workspace --all-features` |
| `1.97.0` | `cargo +1.97.0 check --workspace --all-features` |
| `1.97.1` | full release gate |

`scripts/check_rust_version_matrix.sh` checks the complete table locally. CI
runs each version as an independent fail-fast-disabled matrix job so one failure
does not hide results from the remaining supported compilers.

v0.52.0 raised the MSRV from Rust 1.90 to 1.92. Rust 1.90 and 1.91 cannot
prove the higher-ranked lifetime bound for the client kernel's explicitly
`Send` future (rust-lang/rust#100013). Weakening that contract would prevent
generic cross-thread executor use, while Rust 1.92 and later compile it.

Pinned release tools, checked against crates.io on 2026-08-17:

| Tool | Version |
| --- | --- |
| `cargo-deny` | `0.20.2` |
| `cargo-audit` | `0.22.2` |
| `cargo-sbom` | `0.10.0` |
| `cargo-fuzz` | `0.13.2` |

The non-published fuzz harness separately pins
`nightly-2026-08-17` and `libfuzzer-sys 0.4.13`. Nightly is never used to
compile or test the supported published-crate matrix.

SBOM freshness checks also require `jq` `1.6` or newer. CI uses the runner's
system package; the canonicalization filter is covered by repository tests and
uses no version-specific behavior beyond stable sorting and key deletion.

`scripts/check_latest_tools.sh --local-only` verifies installed pins. Its
`--fetch` mode also compares every Cargo tool pin with crates.io and is required
by the release gate.

Before changing the pinned toolchain, check the current stable Rust release and
update this document, `README.md`, and `rust-toolchain.toml` together.

Tool updates must be checked against their declared Rust version and applied to
this document, `.github/workflows/ci.yml`, and `scripts/check_latest_tools.sh`
together.
