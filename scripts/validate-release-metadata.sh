#!/usr/bin/env sh
set -eu

strict=false
target=HEAD
case "${1:-}" in
"") ;;
--release)
    strict=true
    target="${2:-HEAD}"
    ;;
*)
    echo "usage: scripts/validate-release-metadata.sh [--release [TARGET]]" >&2
    exit 2
    ;;
esac

version=$(sed -n 's/^version = "\(.*\)"/\1/p' release-crates.toml | sed -n '1p')
if ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "release metadata: invalid release-crates.toml version $version" >&2
    exit 1
fi

for required in "release-notes/RELEASE_NOTES_${version}.md" docs/CRATE_VERSION_MATRIX.md; do
    if [ ! -s "$required" ]; then
        echo "release metadata: missing or empty $required" >&2
        exit 1
    fi
done

report="security/pentest/v${version}.md"
if [ ! -s "$report" ]; then
    if [ "$strict" = true ]; then
        echo "release metadata: missing or empty $report" >&2
        exit 1
    fi
    if git rev-parse --verify "v${version}^{commit}" >/dev/null 2>&1; then
        echo "release metadata: tagged v${version} is missing $report" >&2
        exit 1
    fi
    echo "release metadata: v${version} pentest report pending"
else
    if [ "$strict" = false ] && git rev-parse --verify "v${version}^{commit}" >/dev/null 2>&1; then
        target="v${version}"
    fi
    scripts/validate_pentest_binding.py --version "$version" --target "$target"
fi

if [ ! -x scripts/release_crates.py ] || [ ! -x scripts/test-release-crates.py ]; then
    echo "release metadata: missing executable independent crate release scripts" >&2
    exit 1
fi

scripts/release_crates.py --check >/dev/null
