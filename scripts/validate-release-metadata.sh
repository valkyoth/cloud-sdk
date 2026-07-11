#!/usr/bin/env sh
set -eu

strict=false
case "${1:-}" in
"") ;;
--release)
    strict=true
    ;;
*)
    echo "usage: scripts/validate-release-metadata.sh [--release]" >&2
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

if [ "$strict" = true ]; then
    scripts/validate-release-readiness.sh "v${version}"
else
    echo "release metadata: v${version} pentest report checked only by release readiness"
fi

if [ ! -x scripts/release_crates.py ] || [ ! -x scripts/test-release-crates.py ] \
    || [ ! -x scripts/validate-release-readiness.sh ] \
    || [ ! -x scripts/test-release-readiness.sh ]; then
    echo "release metadata: missing executable independent crate release scripts" >&2
    exit 1
fi

scripts/release_crates.py --check >/dev/null
