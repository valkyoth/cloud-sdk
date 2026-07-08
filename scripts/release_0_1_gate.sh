#!/usr/bin/env sh
set -eu

scripts/checks.sh

if cargo deny --version >/dev/null 2>&1; then
    cargo deny check
else
    echo "release gate: cargo-deny not installed; install the pinned version before tagging" >&2
fi

if cargo audit --version >/dev/null 2>&1; then
    cargo audit
else
    echo "release gate: cargo-audit not installed; install the pinned version before tagging" >&2
fi
