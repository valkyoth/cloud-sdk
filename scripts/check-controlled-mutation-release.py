#!/usr/bin/env python3
"""Bind v0.100 mutation evidence to an unchanged qualified source commit."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]
CHECKER_PATH = ROOT / "scripts" / "check-controlled-mutation.py"
EVIDENCE_PATH = Path("security/mutation/v0.100.0.json")
ALLOWED_AFTER_SOURCE = {
    EVIDENCE_PATH.as_posix(),
    "security/pentest/v0.100.0.md",
    "release-notes/RELEASE_NOTES_0.100.0.md",
}
REQUIRED_SOURCE_PATHS = (
    "controlled-mutation-policy.json",
    "scripts/check-controlled-mutation.py",
    "scripts/check-controlled-mutation-release.py",
    "scripts/check_controlled_mutation.sh",
    "docs/CONTROLLED_MUTATION.md",
)


class ReleaseEvidenceError(Exception):
    """Static, payload-free release evidence failure."""


def load_checker():
    specification = importlib.util.spec_from_file_location(
        "controlled_mutation", CHECKER_PATH
    )
    if specification is None or specification.loader is None:
        raise ReleaseEvidenceError("evidence validator could not be loaded")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def git_success(*arguments: str) -> bool:
    return (
        subprocess.run(
            ["git", *arguments],
            cwd=ROOT,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode
        == 0
    )


def changed_paths(source: str) -> tuple[str, ...]:
    try:
        raw = subprocess.check_output(
            ["git", "diff", "--name-only", "-z", source, "HEAD"], cwd=ROOT
        )
        return tuple(
            item.decode("ascii") for item in raw.split(b"\0") if item
        )
    except (subprocess.CalledProcessError, UnicodeDecodeError) as error:
        raise ReleaseEvidenceError("source change inventory failed") from error


def validate_binding(source: str, paths: tuple[str, ...]) -> None:
    if not git_success("cat-file", "-e", f"{source}^{{commit}}"):
        raise ReleaseEvidenceError("qualified source commit was not found")
    if not git_success("merge-base", "--is-ancestor", source, "HEAD"):
        raise ReleaseEvidenceError("qualified source is not an ancestor of HEAD")
    if git_success("cat-file", "-e", f"{source}:{EVIDENCE_PATH.as_posix()}"):
        raise ReleaseEvidenceError("evidence must follow the qualified source commit")
    for path in REQUIRED_SOURCE_PATHS:
        if not git_success("cat-file", "-e", f"{source}:{path}"):
            raise ReleaseEvidenceError("qualified source lacks mutation controls")
    if set(paths) - ALLOWED_AFTER_SOURCE:
        raise ReleaseEvidenceError("qualified source changed after live mutation")
    if EVIDENCE_PATH.as_posix() not in paths:
        raise ReleaseEvidenceError("mutation evidence does not follow qualified source")
    if not git_success("cat-file", "-e", f"HEAD:{EVIDENCE_PATH.as_posix()}"):
        raise ReleaseEvidenceError("mutation evidence is not committed")


def main() -> int:
    checker = load_checker()
    try:
        policy = checker.load_policy()
        evidence = checker.read_json(
            ROOT / EVIDENCE_PATH, policy["maximum_evidence_bytes"]
        )
        checker.validate_evidence(evidence, policy)
        source = evidence["source_commit"]
        validate_binding(source, changed_paths(source))
    except (checker.EvidenceError, ReleaseEvidenceError, KeyError) as error:
        print(f"controlled mutation release: {error}", file=sys.stderr)
        return 1
    print("Controlled-mutation evidence is bound to unchanged qualified source.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
