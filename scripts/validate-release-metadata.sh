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

scripts/check_publishable_readmes.sh

patch="${version##*.}"
if [ "$patch" = "0" ]; then
    gate_version="${version%.*}"
else
    gate_version="$version"
fi
gate="scripts/release_$(printf '%s' "$gate_version" | tr . _)_gate.sh"
if [ ! -x "$gate" ]; then
    echo "release metadata: missing executable $gate" >&2
    exit 1
fi
readiness_call="scripts/validate-release-readiness.sh v${version}"
if [ "$(grep -Fxc "$readiness_call" "$gate")" -ne 2 ]; then
    echo "release metadata: $gate must enforce release readiness at entry and exit" >&2
    exit 1
fi
if ! grep -Fq 'reviewed_head="$(git rev-parse HEAD)"' "$gate" \
    || ! grep -Fq 'if [ "$(git rev-parse HEAD)" != "$reviewed_head" ]; then' "$gate"; then
    echo "release metadata: $gate must bind checks to one unchanged HEAD" >&2
    exit 1
fi

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
