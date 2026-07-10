#!/usr/bin/env sh
set -eu

cat <<'MSG'
Advisory current-version check.

Pinned tools used by CI:
- Rust: 1.97.0
- cargo-deny: 0.20.2
- cargo-audit: 0.22.2
- cargo-sbom: 0.10.0

Before updating pins, check the official Rust release channel and crates.io for
these tools, then update rust-toolchain.toml, .github/workflows/ci.yml, and
docs/toolchain-policy.md together.
MSG
