#!/usr/bin/env sh
set -eu

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

require_clean_tree() {
    status="$(git status --porcelain=v1 --untracked-files=all)"
    if [ -n "$status" ]; then
        echo "release readiness: worktree must be clean" >&2
        printf '%s\n' "$status" >&2
        exit 1
    fi
}

if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    tagged_commit="$(git rev-list -n 1 "$tag")"
    head_commit="$(git rev-parse HEAD)"
    if [ "$tagged_commit" != "$head_commit" ]; then
        echo "existing tag ${tag} does not point at HEAD" >&2
        exit 1
    fi
fi

if [ -f PENTEST.md ]; then
    echo "root PENTEST.md is temporary scratch input and must be removed" >&2
    exit 1
fi

require_clean_tree

if [ ! -f "$release_notes" ]; then
    echo "missing release notes: ${release_notes}" >&2
    exit 1
fi

if [ ! -s sbom/cloud-sdk.spdx.json ]; then
    echo "missing or empty SBOM: sbom/cloud-sdk.spdx.json" >&2
    exit 1
fi

if [ ! -s sbom/reqwest-feature-unification.spdx.json ]; then
    echo "missing or empty SBOM: sbom/reqwest-feature-unification.spdx.json" >&2
    exit 1
fi

if [ ! -s sbom/fuzz.spdx.json ]; then
    echo "missing or empty SBOM: sbom/fuzz.spdx.json" >&2
    exit 1
fi

if [ ! -f "$pentest_report" ]; then
    echo "missing pentest report: ${pentest_report}" >&2
    exit 1
fi

if ! git cat-file -e "HEAD:${pentest_report}" 2>/dev/null; then
    echo "pentest report must be committed in tag candidate: ${pentest_report}" >&2
    exit 1
fi

grep -q '^Status: PASS$' "$pentest_report"
grep -Eq '^Reviewed-Commit: [0-9a-f]{40}$' "$pentest_report"
grep -Eq '^Tester: .+' "$pentest_report"
grep -Eq '^Scope: .+' "$pentest_report"
grep -Eq '^Date: [0-9]{4}-[0-9]{2}-[0-9]{2}$' "$pentest_report"

reviewed_commit="$(sed -n 's/^Reviewed-Commit: //p' "$pentest_report")"
if ! git cat-file -e "${reviewed_commit}^{commit}" 2>/dev/null; then
    echo "reviewed commit ${reviewed_commit} was not found" >&2
    exit 1
fi

if ! git merge-base --is-ancestor "$reviewed_commit" HEAD; then
    echo "reviewed commit ${reviewed_commit} is not an ancestor of HEAD" >&2
    exit 1
fi
