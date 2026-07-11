#!/usr/bin/env sh
set -eu

scripts/checks.sh
scripts/check_hetzner_upstream.sh --local-only
scripts/check_hetzner_api_drift.py --fetch
scripts/check_iana_ipv6_registry.py --fetch
# Full-suite coverage already ran in scripts/checks.sh; this is the v0.12 targeted rerun.
cargo test -p cloud-sdk-hetzner --all-features dns_zones

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
