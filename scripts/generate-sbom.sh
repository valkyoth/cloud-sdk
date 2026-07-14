#!/usr/bin/env sh
set -eu

mkdir -p sbom
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

generate_complete() {
    project="$1"
    manifest="$2"
    output="$3"
    label="$4"
    cargo metadata --manifest-path "$manifest" --locked --all-features \
        --format-version 1 \
        >"$tmp/${label}.metadata.json"
    cargo sbom --project-directory "$project" --output-format spdx_json_2_3 \
        >"$tmp/${label}.base.json"
    scripts/complete_spdx_sbom.py "$tmp/${label}.metadata.json" \
        "$tmp/${label}.base.json" "$output"
    scripts/check_sbom_completeness.sh "$manifest" "$output" "$label"
}

generate_complete . Cargo.toml sbom/cloud-sdk.spdx.json cloud-sdk
generate_complete tests/reqwest-feature-unification \
    tests/reqwest-feature-unification/Cargo.toml \
    sbom/reqwest-feature-unification.spdx.json reqwest-feature-unification
generate_complete fuzz fuzz/Cargo.toml sbom/fuzz.spdx.json fuzz
test -s sbom/cloud-sdk.spdx.json
test -s sbom/reqwest-feature-unification.spdx.json
test -s sbom/fuzz.spdx.json
