#!/usr/bin/env sh
set -eu

version=$(sed -n 's/^version = "\(.*\)"/\1/p' release-crates.toml | sed -n '1p')
if ! printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "release metadata: invalid release-crates.toml version $version" >&2
    exit 1
fi

for required in "release-notes/RELEASE_NOTES_${version}.md" "security/pentest/v${version}.md" docs/CRATE_VERSION_MATRIX.md; do
    if [ ! -s "$required" ]; then
        echo "release metadata: missing or empty $required" >&2
        exit 1
    fi
done

check_pentest_report() {
    report="security/pentest/v${1}.md"
    if [ ! -s "$report" ]; then
        echo "release metadata: missing or empty $report" >&2
        exit 1
    fi

    if ! grep -Eq '^Status: PASS$' "$report"; then
        echo "release metadata: $report is not Status: PASS" >&2
        exit 1
    fi

    reviewed_commit=$(sed -n 's/^Reviewed-Commit:[[:space:]]*//p' "$report")
    if ! printf '%s\n' "$reviewed_commit" | grep -Eq '^[0-9a-f]{40}$'; then
        echo "release metadata: $report missing full 40-char Reviewed-Commit" >&2
        exit 1
    fi

    for field in Tester Scope; do
        value=$(sed -n "s/^${field}:[[:space:]]*//p" "$report")
        if ! printf '%s\n' "$value" | grep -Eq '\S'; then
            echo "release metadata: $report has blank $field" >&2
            exit 1
        fi
    done

    date_value=$(sed -n 's/^Date:[[:space:]]*//p' "$report")
    if ! printf '%s\n' "$date_value" | grep -Eq '^[0-9]{4}-[0-9]{2}-[0-9]{2}$'; then
        echo "release metadata: $report missing Date: YYYY-MM-DD" >&2
        exit 1
    fi
}

check_pentest_report "$version"

if [ ! -x scripts/release_crates.py ] || [ ! -x scripts/test-release-crates.py ]; then
    echo "release metadata: missing executable independent crate release scripts" >&2
    exit 1
fi

scripts/release_crates.py --check >/dev/null
