#!/usr/bin/env sh
set -eu

for required in SECURITY.md deny.toml docs/threat-model.md docs/security-controls.md docs/supply-chain-security.md docs/unsafe-policy.md; do
    if [ ! -s "$required" ]; then
        echo "security policy: missing or empty $required" >&2
        exit 1
    fi
done

if ! grep -Fq 'unsafe_code = "forbid"' Cargo.toml; then
    echo "security policy: workspace must forbid unsafe code" >&2
    exit 1
fi

if ! grep -Fq 'MIT OR Apache-2.0' Cargo.toml; then
    echo "security policy: workspace license must be MIT OR Apache-2.0" >&2
    exit 1
fi
