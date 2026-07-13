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

cargo sbom --output-format spdx_json_2_3 >"$tmp/cloud-sdk.raw.json"
cargo sbom --project-directory tests/reqwest-feature-unification \
    --output-format spdx_json_2_3 >"$tmp/reqwest-fixture.raw.json"

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

compare_sbom sbom/cloud-sdk.spdx.json "$tmp/cloud-sdk.raw.json" cloud-sdk
compare_sbom sbom/reqwest-feature-unification.spdx.json \
    "$tmp/reqwest-fixture.raw.json" reqwest-feature-unification

echo "SBOM freshness: committed evidence matches both dependency graphs"
