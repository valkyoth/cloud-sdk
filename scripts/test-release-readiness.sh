#!/usr/bin/env sh
set -eu

export PYTHONDONTWRITEBYTECODE=1

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
root="$(pwd)"

make_fixture() {
    name="$1"
    version="$2"
    stage="$3"
    baseline="$4"
    exceptional="${5:-false}"
    repo="$tmp/$name"
    mkdir -p "$repo/scripts" "$repo/release-notes" "$repo/security/pentest" "$repo/sbom"
    cp "$root/scripts/validate-release-readiness.sh" "$repo/scripts/"
    cp "$root/scripts/release_train.py" "$repo/scripts/"
    cp "$root/scripts/validate-release-train.py" "$repo/scripts/"
    cp "$root/scripts/test-release-readiness-fixture.py" "$repo/scripts/"
    (
        cd "$repo"
        git init -q
        git config user.email "release-readiness@example.invalid"
        git config user.name "Release Readiness Test"
        printf 'fixture\n' >README.md
        scripts/test-release-readiness-fixture.py \
            "$version" "$stage" "$baseline" "$exceptional"
        git add .
        git commit -q -m fixture
        git tag "v${baseline}"
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
    grep -q "$expected" "$tmp/stderr" || {
        echo "expected stderr to contain: $expected" >&2
        cat "$tmp/stderr" >&2
        exit 1
    }
}

write_sboms() {
    for name in cloud-sdk reqwest-feature-unification fuzz prepared-coverage-check; do
        printf '{"spdxVersion":"SPDX-2.3"}\n' >"sbom/${name}.spdx.json"
    done
}

write_internal_notes() {
    version="$1"
    checkpoint="$2"
    cat >"release-notes/RELEASE_NOTES_${version}.md" <<EOF
# Fixture ${version}
Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO ${checkpoint}
EOF
}

write_public_notes() {
    cat >"release-notes/RELEASE_NOTES_${1}.md" <<EOF
# Fixture ${1}
Security-Review: PASS
Pentest: PASS
Publication: PENDING
EOF
}

write_pentest() {
    report="security/pentest/v${1}.md"
    assessment="${4:-INCREMENTAL}"
    cat >"$report" <<EOF
Status: PASS
Assessment: ${assessment}
Baseline: v${2}
Range-End: v${1}
Reviewed-Commit: ${3}
Tester: Release Readiness Test
Scope: Changes after v${2} through v${1}.
Date: 2026-08-05
EOF
}

stage_candidate() {
    version="$1"
    baseline="$2"
    notes="$3"
    checkpoint="${4:-}"
    if [ "$notes" = internal ]; then
        write_internal_notes "$version" "$checkpoint"
    else
        write_public_notes "$version"
    fi
    write_sboms
    git add release-notes sbom
    git commit -q -m metadata
    reviewed="$(git rev-parse HEAD)"
    write_pentest "$version" "$baseline" "$reviewed"
    git add security
    git commit -q -m report
}

repo="$(make_fixture internal 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    stage_candidate 0.56.0 0.55.0 internal v0.60.0
    scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture missing-report 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    write_internal_notes 0.56.0 v0.60.0
    write_sboms
    git add release-notes sbom
    git commit -q -m metadata
    assert_fails_with "missing pentest report" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture wrong-deferral 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    write_internal_notes 0.56.0 v0.65.0
    write_sboms
    git add release-notes sbom
    git commit -q -m metadata
    assert_fails_with "must defer publication to v0.60.0" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture checkpoint 0.60.0 public 0.55.0)"
(
    cd "$repo"
    stage_candidate 0.60.0 0.55.0 public
    scripts/validate-release-readiness.sh v0.60.0
)

repo="$(make_fixture missing-public-report 0.60.0 public 0.55.0)"
(
    cd "$repo"
    write_public_notes 0.60.0
    write_sboms
    git add release-notes sbom
    git commit -q -m metadata
    assert_fails_with "missing pentest report" \
        scripts/validate-release-readiness.sh v0.60.0
)

repo="$(make_fixture dirty 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    stage_candidate 0.56.0 0.55.0 internal v0.60.0
    printf 'dirty\n' >>README.md
    assert_fails_with "worktree must be clean" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture scratch 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    printf 'temporary\n' >PENTEST.md
    assert_fails_with "root PENTEST.md is temporary scratch input" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture missing-sbom 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    write_internal_notes 0.56.0 v0.60.0
    git add release-notes
    git commit -q -m metadata
    assert_fails_with "missing or empty SBOM" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture uncommitted-report 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    write_internal_notes 0.56.0 v0.60.0
    write_sboms
    printf 'security/pentest/v0.56.0.md\n' >.gitignore
    git add .gitignore release-notes sbom
    git commit -q -m metadata
    write_pentest 0.56.0 0.55.0 "$(git rev-parse HEAD)"
    assert_fails_with "pentest report must be committed" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture wrong-reviewed 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    write_internal_notes 0.56.0 v0.60.0
    write_sboms
    write_pentest 0.56.0 0.55.0 0000000000000000000000000000000000000000
    git add release-notes sbom security
    git commit -q -m report
    assert_fails_with "reviewed commit .* was not found" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture mismatched-tag 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    stage_candidate 0.56.0 0.55.0 internal v0.60.0
    git tag v0.56.0
    printf 'later\n' >>README.md
    git commit -qam later
    assert_fails_with "existing tag v0.56.0 does not point at HEAD" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture wrong-assessment 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    write_internal_notes 0.56.0 v0.60.0
    write_sboms
    git add release-notes sbom
    git commit -q -m metadata
    write_pentest 0.56.0 0.55.0 "$(git rev-parse HEAD)" CUMULATIVE
    git add security
    git commit -q -m report
    assert_fails_with "development release requires Assessment: INCREMENTAL" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture conflicting-notes 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    stage_candidate 0.56.0 0.55.0 internal v0.60.0
    printf 'Pentest: REQUIRED\n' >>release-notes/RELEASE_NOTES_0.56.0.md
    git add release-notes
    git commit -q -m conflict
    assert_fails_with "exactly one Pentest field" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture conflicting-report 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    stage_candidate 0.56.0 0.55.0 internal v0.60.0
    printf 'Status: FAIL\n' >>security/pentest/v0.56.0.md
    git add security
    git commit -q -m conflict
    assert_fails_with "exactly one Status field" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture wrong-baseline 0.56.0 internal 0.55.0)"
(
    cd "$repo"
    write_internal_notes 0.56.0 v0.60.0
    write_sboms
    git add release-notes sbom
    git commit -q -m metadata
    write_pentest 0.56.0 0.50.0 "$(git rev-parse HEAD)"
    git add security
    git commit -q -m report
    assert_fails_with "pentest report baseline must be v0.55.0" \
        scripts/validate-release-readiness.sh v0.56.0
)

repo="$(make_fixture targeted 0.57.0 internal 0.55.0 true)"
(
    cd "$repo"
    write_internal_notes 0.57.0 v0.60.0
    write_sboms
    git add release-notes sbom
    git commit -q -m metadata
    write_pentest 0.57.0 0.55.0 "$(git rev-parse HEAD)" TARGETED
    git add security
    git commit -q -m report
    scripts/validate-release-readiness.sh v0.57.0
)

echo "16 staged release readiness tests passed."
