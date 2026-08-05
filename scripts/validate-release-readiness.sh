#!/usr/bin/env sh
set -eu

fail() {
    echo "release readiness: $1" >&2
    exit 1
}

tag="${1:-}"
case "$tag" in
v[0-9]*.[0-9]*.[0-9]*) ;;
*)
    echo "usage: scripts/validate-release-readiness.sh vX.Y.Z" >&2
    exit 2
    ;;
esac

version="${tag#v}"
release_notes="release-notes/RELEASE_NOTES_${version}.md"
pentest_report="security/pentest/${tag}.md"

scripts/validate-release-train.py >/dev/null
context="$(
    python3 -c \
        'import tomllib; r = tomllib.load(open("release-crates.toml", "rb"))["release"]; print("|".join((r["version"], r["baseline"], r["review_baseline"], r["stage"], str(r["exceptional"]).lower())))'
)"
IFS='|' read -r planned_version baseline review_baseline stage exceptional <<EOF
${context}
EOF
test "$planned_version" = "$version" ||
    fail "release plan version must be ${version}"

test ! -f PENTEST.md ||
    fail "root PENTEST.md is temporary scratch input and must be removed"
status="$(git status --porcelain=v1 --untracked-files=all)"
if [ -n "$status" ]; then
    printf '%s\n' "$status" >&2
    fail "worktree must be clean"
fi
if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    tagged_commit="$(git rev-list -n 1 "$tag")"
    head_commit="$(git rev-parse HEAD)"
    test "$tagged_commit" = "$head_commit" ||
        fail "existing tag ${tag} does not point at HEAD"
fi
test -f "$release_notes" || fail "missing release notes: ${release_notes}"

for sbom in \
    sbom/cloud-sdk.spdx.json \
    sbom/reqwest-feature-unification.spdx.json \
    sbom/fuzz.spdx.json \
    sbom/prepared-coverage-check.spdx.json; do
    test -s "$sbom" || fail "missing or empty SBOM: ${sbom}"
done

if [ "$stage" != "internal" ] && [ "$stage" != "public" ]; then
    fail "unknown release classification: stage=${stage} exceptional=${exceptional}"
fi
checkpoint="$(
    python3 -c \
        'import sys; minor = int(sys.argv[1].split(".")[1]); nxt = ((minor // 5) + 1) * 5; print("v1.0.0" if nxt >= 100 else f"v0.{nxt}.0")' \
        "$version"
)"
grep -Fxq 'Security-Review: PASS' "$release_notes" ||
    fail "release notes must record Security-Review: PASS"
grep -Fxq 'Pentest: PASS' "$release_notes" ||
    fail "release notes must record Pentest: PASS"
if [ "$stage" = "internal" ]; then
    grep -Fxq "Publication: DEFERRED TO ${checkpoint}" "$release_notes" ||
        fail "internal release notes must defer publication to ${checkpoint}"
else
    grep -Fxq 'Publication: PENDING' "$release_notes" ||
        fail "public release notes must record Publication: PENDING"
fi
test -f "$pentest_report" || fail "missing pentest report: ${pentest_report}"
git cat-file -e "HEAD:${pentest_report}" 2>/dev/null ||
    fail "pentest report must be committed in tag candidate: ${pentest_report}"
grep -q '^Status: PASS$' "$pentest_report" || fail "pentest status must be PASS"
grep -Eq '^Reviewed-Commit: [0-9a-f]{40}$' "$pentest_report" ||
    fail "pentest report requires Reviewed-Commit"
grep -Eq '^Tester: .+' "$pentest_report" || fail "pentest report requires Tester"
grep -Eq '^Scope: .+' "$pentest_report" || fail "pentest report requires Scope"
grep -Eq '^Date: [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$pentest_report" ||
    fail "pentest report requires Date"

if [ "$review_baseline" != "$version" ]; then
    grep -Fxq "Baseline: v${review_baseline}" "$pentest_report" ||
        fail "pentest report baseline must be v${review_baseline}"
    grep -Fxq "Range-End: ${tag}" "$pentest_report" ||
        fail "pentest report range end must be ${tag}"
    assessment="$(sed -n 's/^Assessment: //p' "$pentest_report")"
    if [ "${version%%.*}" != "0" ]; then
        test "$assessment" = "FULL" ||
            fail "stable release requires Assessment: FULL"
    elif [ "$exceptional" = "true" ]; then
        case "$assessment" in TARGETED | INCREMENTAL | FULL) ;;
        *) fail "exceptional pentest assessment is invalid" ;;
        esac
    else
        test "$assessment" = "INCREMENTAL" ||
            fail "development release requires Assessment: INCREMENTAL"
    fi
fi

reviewed_commit="$(sed -n 's/^Reviewed-Commit: //p' "$pentest_report")"
git cat-file -e "${reviewed_commit}^{commit}" 2>/dev/null ||
    fail "reviewed commit ${reviewed_commit} was not found"
git merge-base --is-ancestor "$reviewed_commit" HEAD ||
    fail "reviewed commit ${reviewed_commit} is not an ancestor of HEAD"
git cat-file -e "v${review_baseline}^{commit}" 2>/dev/null ||
    fail "review baseline tag v${review_baseline} was not found"
git merge-base --is-ancestor "v${review_baseline}" "$reviewed_commit" ||
    fail "reviewed commit does not descend from v${review_baseline}"
if [ "$stage" = "public" ]; then
    milestones="$(
        python3 -c \
            'import tomllib; print(" ".join(tomllib.load(open("release-crates.toml", "rb"))["release"]["cumulative_milestones"]))'
    )"
    for milestone in $milestones; do
        test "$milestone" = "$version" && continue
        report="security/pentest/v${milestone}.md"
        git cat-file -e "v${milestone}:${report}" 2>/dev/null ||
            fail "published train lacks tagged pentest evidence for v${milestone}"
    done
    echo "${tag} pentest and public release metadata are ready"
else
    echo "${tag} pentest passed; crates.io publication remains deferred to ${checkpoint}"
fi
