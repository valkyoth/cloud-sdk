#!/usr/bin/env sh
set -eu

if ! command -v jq >/dev/null 2>&1; then
    echo "SBOM freshness: jq is required" >&2
    exit 1
fi
if ! cargo sbom --version >/dev/null 2>&1; then
    echo "SBOM freshness: cargo-sbom is required" >&2
    exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

generate_complete() {
    project="$1"
    manifest="$2"
    label="$3"
    cargo metadata --manifest-path "$manifest" --locked --all-features \
        --format-version 1 \
        >"$tmp/${label}.metadata.json"
    cargo sbom --project-directory "$project" --output-format spdx_json_2_3 \
        >"$tmp/${label}.base.json"
    scripts/complete_spdx_sbom.py "$tmp/${label}.metadata.json" \
        "$tmp/${label}.base.json" "$tmp/${label}.complete.json"
}

generate_complete . Cargo.toml cloud-sdk
generate_complete tests/reqwest-feature-unification \
    tests/reqwest-feature-unification/Cargo.toml reqwest-feature-unification
generate_complete fuzz fuzz/Cargo.toml fuzz

canonicalize() {
    jq -S -f scripts/canonicalize-sbom.jq "$1" >"$2"
}

compare_sbom() {
    committed="$1"
    generated="$2"
    label="$3"
    canonicalize "$committed" "$tmp/${label}.committed.json"
    canonicalize "$generated" "$tmp/${label}.generated.json"
    if ! diff -u "$tmp/${label}.committed.json" \
        "$tmp/${label}.generated.json"; then
        echo "SBOM freshness: stale committed evidence for ${label}" >&2
        exit 1
    fi
}

compare_sbom sbom/cloud-sdk.spdx.json \
    "$tmp/cloud-sdk.complete.json" cloud-sdk
compare_sbom sbom/reqwest-feature-unification.spdx.json \
    "$tmp/reqwest-feature-unification.complete.json" reqwest-feature-unification
compare_sbom sbom/fuzz.spdx.json "$tmp/fuzz.complete.json" fuzz

scripts/check_sbom_completeness.sh Cargo.toml \
    sbom/cloud-sdk.spdx.json cloud-sdk
scripts/check_sbom_completeness.sh tests/reqwest-feature-unification/Cargo.toml \
    sbom/reqwest-feature-unification.spdx.json reqwest-feature-unification
scripts/check_sbom_completeness.sh fuzz/Cargo.toml sbom/fuzz.spdx.json fuzz

echo "SBOM freshness: all three complete dependency graphs match locked metadata"
