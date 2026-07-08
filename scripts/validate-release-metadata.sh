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

    head_commit=$(git rev-parse HEAD)
    if [ "$reviewed_commit" != "$head_commit" ]; then
        if ! git merge-base --is-ancestor "$reviewed_commit" "$head_commit"; then
            echo "release metadata: pentest reviewed $reviewed_commit, which is not an ancestor of HEAD $head_commit" >&2
            exit 1
        fi
        changed_after_review=$(git diff --name-only "$reviewed_commit" "$head_commit")
        if [ "$changed_after_review" != "$report" ]; then
            echo "release metadata: only $report may change after Reviewed-Commit" >&2
            printf '%s\n' "$changed_after_review" >&2
            exit 1
        fi
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

check_pentest_report "0.1.0"

if [ ! -x scripts/release_crates.py ] || [ ! -x scripts/test-release-crates.py ]; then
    echo "release metadata: missing executable independent crate release scripts" >&2
    exit 1
fi

scripts/release_crates.py --check >/dev/null
