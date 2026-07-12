#!/usr/bin/env sh
set -eu

scripts/validate-release-readiness.sh v0.14.0
scripts/checks.sh
scripts/check_serde_boundary.sh
scripts/check_sanitization_boundary.sh
scripts/check_hetzner_upstream.sh --local-only
scripts/check_hetzner_api_drift.py --fetch
scripts/check_iana_ipv6_registry.py --fetch
cargo test -p cloud-sdk-hetzner --all-features serde

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
