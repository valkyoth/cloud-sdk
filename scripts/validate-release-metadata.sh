#!/usr/bin/env sh
set -eu

version=$(sed -n 's/^version = "\(.*\)"/\1/p' release-crates.toml | sed -n '1p')
if [ "$version" != "0.1.0" ]; then
    echo "release metadata: expected release-crates.toml version 0.1.0, got $version" >&2
    exit 1
fi

for required in release-notes/RELEASE_NOTES_0.1.0.md security/pentest/v0.1.0.md docs/CRATE_VERSION_MATRIX.md; do
    if [ ! -s "$required" ]; then
        echo "release metadata: missing or empty $required" >&2
        exit 1
    fi
done

if [ ! -x scripts/release_crates.py ] || [ ! -x scripts/test-release-crates.py ]; then
    echo "release metadata: missing executable independent crate release scripts" >&2
    exit 1
fi

scripts/release_crates.py --check >/dev/null
