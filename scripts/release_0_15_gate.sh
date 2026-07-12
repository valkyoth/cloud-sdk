#!/usr/bin/env sh
set -eu

scripts/validate-release-readiness.sh v0.15.0
scripts/checks.sh
scripts/check_testkit_boundary.sh
scripts/check_rust_version_matrix.sh
scripts/check_hetzner_upstream.sh --local-only
scripts/check_hetzner_api_drift.py --fetch
scripts/check_iana_ipv6_registry.py --fetch
cargo test -p cloud-sdk-hetzner --all-features \
    serde_boundary_consumes_provider_neutral_adversarial_corpus

if ! cargo deny --version >/dev/null 2>&1; then
    echo "release gate: cargo-deny not installed; install the pinned version before tagging" >&2
    exit 1
fi
cargo deny check

if ! cargo audit --version >/dev/null 2>&1; then
    echo "release gate: cargo-audit not installed; install the pinned version before tagging" >&2
    exit 1
fi
cargo audit
