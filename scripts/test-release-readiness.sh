#!/usr/bin/env sh
set -eu

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
source_script="$(pwd)/scripts/validate-release-readiness.sh"

make_fixture() {
    name="$1"
    repo="$tmp/$name"
    mkdir -p "$repo/scripts" "$repo/release-notes" "$repo/security/pentest" "$repo/sbom"
    cp "$source_script" "$repo/scripts/validate-release-readiness.sh"
    (
        cd "$repo"
        git init -q
        git config user.email "release-readiness@example.invalid"
        git config user.name "Release Readiness Test"
        printf 'fixture\n' >README.md
        git add README.md scripts/validate-release-readiness.sh
        git commit -q -m "fixture"
    )
    printf '%s\n' "$repo"
}

assert_fails_with() {
    expected="$1"
    shift
    if "$@" >"$tmp/stdout" 2>"$tmp/stderr"; then
        echo "expected command to fail: $*" >&2
        exit 1
    fi
    if ! grep -q "$expected" "$tmp/stderr"; then
        echo "expected stderr to contain: $expected" >&2
        cat "$tmp/stderr" >&2
        exit 1
    fi
}

write_release_notes() {
    printf '# Release %s\n' "$1" >"release-notes/RELEASE_NOTES_${1}.md"
}

write_sbom() {
    printf '{"spdxVersion":"SPDX-2.3"}\n' >sbom/cloud-sdk.spdx.json
    printf '{"spdxVersion":"SPDX-2.3"}\n' \
        >sbom/reqwest-feature-unification.spdx.json
    printf '{"spdxVersion":"SPDX-2.3"}\n' >sbom/fuzz.spdx.json
}

write_pentest() {
    cat >"security/pentest/${1}.md" <<EOF
Status: PASS
Reviewed-Commit: ${2}
Tester: Release Readiness Test
Scope: Fixture release metadata.
Date: 2026-07-11
EOF
}

repo="$(make_fixture bad-tag)"
(
    cd "$repo"
    assert_fails_with "usage: scripts/validate-release-readiness.sh vX.Y.Z" \
        scripts/validate-release-readiness.sh "0.2.0"
)

repo="$(make_fixture mismatched-tag)"
(
    cd "$repo"
    git tag v0.2.0
    printf 'later\n' >>README.md
    git commit -qam "later"
    assert_fails_with "existing tag v0.2.0 does not point at HEAD" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture scratch-pentest)"
(
    cd "$repo"
    printf 'scratch\n' >PENTEST.md
    assert_fails_with "root PENTEST.md is temporary scratch input" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture modified-tracked-file)"
(
    cd "$repo"
    printf 'modified\n' >>README.md
    assert_fails_with "release readiness: worktree must be clean" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture untracked-file)"
(
    cd "$repo"
    printf 'untracked\n' >UNTRACKED.txt
    assert_fails_with "release readiness: worktree must be clean" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture missing-release-notes)"
(
    cd "$repo"
    assert_fails_with "missing release notes" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture missing-sbom)"
(
    cd "$repo"
    write_release_notes "0.2.0"
    git add release-notes
    git commit -q -m "release notes"
    assert_fails_with "missing or empty SBOM" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture missing-report)"
(
    cd "$repo"
    write_release_notes "0.2.0"
    write_sbom
    git add release-notes sbom
    git commit -q -m "release metadata"
    assert_fails_with "missing pentest report" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture missing-fixture-sbom)"
(
    cd "$repo"
    write_release_notes "0.2.0"
    printf '{"spdxVersion":"SPDX-2.3"}\n' >sbom/cloud-sdk.spdx.json
    git add release-notes sbom
    git commit -q -m "partial release metadata"
    assert_fails_with "missing or empty SBOM: sbom/reqwest-feature-unification.spdx.json" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture missing-fuzz-sbom)"
(
    cd "$repo"
    write_release_notes "0.2.0"
    printf '{"spdxVersion":"SPDX-2.3"}\n' >sbom/cloud-sdk.spdx.json
    printf '{"spdxVersion":"SPDX-2.3"}\n' \
        >sbom/reqwest-feature-unification.spdx.json
    git add release-notes sbom
    git commit -q -m "partial release metadata"
    assert_fails_with "missing or empty SBOM: sbom/fuzz.spdx.json" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture uncommitted-report)"
(
    cd "$repo"
    reviewed_commit="$(git rev-parse HEAD)"
    write_release_notes "0.2.0"
    write_sbom
    printf 'security/pentest/v0.2.0.md\n' >.gitignore
    git add .gitignore release-notes sbom
    git commit -q -m "release metadata"
    write_pentest "v0.2.0" "$reviewed_commit"
    assert_fails_with "pentest report must be committed" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture wrong-reviewed-commit)"
(
    cd "$repo"
    write_release_notes "0.2.0"
    write_sbom
    write_pentest "v0.2.0" "0000000000000000000000000000000000000000"
    git add release-notes sbom security/pentest/v0.2.0.md
    git commit -q -m "report"
    assert_fails_with "reviewed commit .* was not found" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture unrelated-reviewed-commit)"
(
    cd "$repo"
    reviewed_commit="$(git rev-parse HEAD)"
    git checkout -q --orphan unrelated
    git rm -q -rf .
    mkdir -p scripts release-notes security/pentest sbom
    cp "$source_script" scripts/validate-release-readiness.sh
    printf 'unrelated\n' >README.md
    write_release_notes "0.2.0"
    write_sbom
    write_pentest "v0.2.0" "$reviewed_commit"
    git add .
    git commit -q -m "unrelated report"
    assert_fails_with "reviewed commit .* is not an ancestor of HEAD" \
        scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture mixed-release-commit)"
(
    cd "$repo"
    write_release_notes "0.2.0"
    write_sbom
    git add release-notes sbom
    git commit -q -m "release metadata"
    reviewed_commit="$(git rev-parse HEAD)"
    write_pentest "v0.2.0" "$reviewed_commit"
    printf 'changed\n' >>README.md
    git add README.md security/pentest/v0.2.0.md
    git commit -q -m "report plus final metadata"
    scripts/validate-release-readiness.sh "v0.2.0"
)

repo="$(make_fixture ready)"
(
    cd "$repo"
    write_release_notes "0.2.0"
    write_sbom
    git add release-notes sbom
    git commit -q -m "release metadata"
    reviewed_commit="$(git rev-parse HEAD)"
    write_pentest "v0.2.0" "$reviewed_commit"
    git add security/pentest/v0.2.0.md
    git commit -q -m "report"
    scripts/validate-release-readiness.sh "v0.2.0"
    git tag v0.2.0
    scripts/validate-release-readiness.sh "v0.2.0"
)

echo "15 release readiness tests passed."
