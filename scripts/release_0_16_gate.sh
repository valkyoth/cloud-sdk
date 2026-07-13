#!/usr/bin/env sh
set -eu

scripts/validate-release-readiness.sh v0.16.0
scripts/checks.sh
scripts/check_reqwest_boundary.sh
scripts/check_rust_version_matrix.sh
scripts/check_hetzner_upstream.sh --local-only
scripts/check_hetzner_api_drift.py --fetch
scripts/check_iana_ipv6_registry.py --fetch

if ! cargo deny --version >/dev/null 2>&1; then
    echo "release gate: cargo-deny not installed; install the pinned version before tagging" >&2
    exit 1
fi
cargo deny check
cargo deny --manifest-path tests/reqwest-feature-unification/Cargo.toml \
    --config deny.toml --locked check advisories licenses sources

if ! cargo audit --version >/dev/null 2>&1; then
    echo "release gate: cargo-audit not installed; install the pinned version before tagging" >&2
    exit 1
fi
cargo audit
cargo audit --no-fetch --file tests/reqwest-feature-unification/Cargo.lock
