# Toolchain Policy

The workspace develops on stable Rust `1.97.0` and declares MSRV `1.90`.

Compatibility must be maintained for:

| Rust | Requirement |
| --- | --- |
| `1.90.0` | `cargo +1.90.0 check --workspace --all-features` |
| `1.91.0` | `cargo +1.91.0 check --workspace --all-features` |
| `1.92.0` | `cargo +1.92.0 check --workspace --all-features` |
| `1.93.0` | `cargo +1.93.0 check --workspace --all-features` |
| `1.94.0` | `cargo +1.94.0 check --workspace --all-features` |
| `1.95.0` | `cargo +1.95.0 check --workspace --all-features` |
| `1.96.0` | `cargo +1.96.0 check --workspace --all-features` |
| `1.96.1` | `cargo +1.96.1 check --workspace --all-features` |
| `1.97.0` | full release gate |

Pinned release tools, checked against crates.io on 2026-07-12:

| Tool | Version |
| --- | --- |
| `cargo-deny` | `0.20.2` |
| `cargo-audit` | `0.22.2` |
| `cargo-sbom` | `0.10.0` |

Before changing the pinned toolchain, check the current stable Rust release and
update this document, `README.md`, and `rust-toolchain.toml` together.

Tool updates must be checked against their declared Rust version and applied to
this document, `.github/workflows/ci.yml`, and `scripts/check_latest_tools.sh`
together.
