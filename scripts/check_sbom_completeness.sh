#!/usr/bin/env sh
set -eu

manifest="${1:-}"
sbom="${2:-}"
label="${3:-}"
if [ -z "$manifest" ] || [ -z "$sbom" ] || [ -z "$label" ] || [ "$#" -ne 3 ]; then
    echo "usage: scripts/check_sbom_completeness.sh MANIFEST SBOM LABEL" >&2
    exit 2
fi
if ! command -v jq >/dev/null 2>&1; then
    echo "SBOM completeness: jq is required" >&2
    exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
cargo metadata --manifest-path "$manifest" --locked --all-features \
    --format-version 1 \
    >"$tmp/metadata.json"

jq -er '.packages[] | "\(.name)@\(.version)"' "$tmp/metadata.json" |
    LC_ALL=C sort >"$tmp/expected"
jq -er '.packages[] | "\(.name)@\(.versionInfo)"' "$sbom" |
    LC_ALL=C sort >"$tmp/actual"

duplicates="$(uniq -d "$tmp/expected")"
if [ -n "$duplicates" ]; then
    echo "SBOM completeness: ambiguous Cargo package identities for ${label}" >&2
    printf '%s\n' "$duplicates" >&2
    exit 1
fi
duplicates="$(uniq -d "$tmp/actual")"
if [ -n "$duplicates" ]; then
    echo "SBOM completeness: duplicate SPDX packages for ${label}" >&2
    printf '%s\n' "$duplicates" >&2
    exit 1
fi

missing="$(comm -23 "$tmp/expected" "$tmp/actual")"
if [ -n "$missing" ]; then
    echo "SBOM completeness: ${label} is missing locked packages" >&2
    printf '%s\n' "$missing" >&2
    exit 1
fi
unexpected="$(comm -13 "$tmp/expected" "$tmp/actual")"
if [ -n "$unexpected" ]; then
    echo "SBOM completeness: ${label} contains unexpected packages" >&2
    printf '%s\n' "$unexpected" >&2
    exit 1
fi

echo "SBOM completeness: ${label} inventories every locked Cargo package"
