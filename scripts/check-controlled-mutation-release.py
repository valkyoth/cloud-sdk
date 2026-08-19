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
REQUIRED_SOURCE_PATHS = {
    "controlled-mutation-policy.json": b"100644",
    "scripts/check-controlled-mutation.py": b"100755",
    "scripts/check-controlled-mutation-release.py": b"100755",
    "scripts/check_controlled_mutation.sh": b"100755",
    "docs/CONTROLLED_MUTATION.md": b"100644",
}
MAX_EVIDENCE_BYTES = 65_536
MAX_POLICY_BYTES = 32_768


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


def parse_tree_entry(raw: bytes, path: str, expected_mode: bytes = b"100644") -> str:
    if not raw.endswith(b"\0") or raw.count(b"\0") != 1:
        raise ReleaseEvidenceError("committed path is missing or ambiguous")
    try:
        metadata, actual_path = raw[:-1].split(b"\t", 1)
        mode, object_type, object_id = metadata.split()
        decoded_path = actual_path.decode("ascii")
        decoded_id = object_id.decode("ascii")
    except (ValueError, UnicodeDecodeError) as error:
        raise ReleaseEvidenceError("committed path metadata is invalid") from error
    if (
        mode != expected_mode
        or object_type != b"blob"
        or decoded_path != path
        or len(decoded_id) not in {40, 64}
    ):
        raise ReleaseEvidenceError("committed path has the wrong type or mode")
    return decoded_id


def regular_blob_oid(
    revision: str, path: str, expected_mode: bytes = b"100644"
) -> str:
    try:
        raw = subprocess.check_output(
            ["git", "ls-tree", "-z", "--full-tree", revision, "--", path],
            cwd=ROOT,
        )
    except subprocess.CalledProcessError as error:
        raise ReleaseEvidenceError("committed path could not be inspected") from error
    return parse_tree_entry(raw, path, expected_mode)


def read_committed_blob(object_id: str, maximum: int) -> bytes:
    try:
        size_raw = subprocess.check_output(
            ["git", "cat-file", "-s", object_id], cwd=ROOT
        ).strip()
        size = int(size_raw)
    except (subprocess.CalledProcessError, ValueError) as error:
        raise ReleaseEvidenceError("committed evidence size is invalid") from error
    if size <= 0 or size > maximum:
        raise ReleaseEvidenceError("committed evidence size is invalid")
    try:
        raw = subprocess.check_output(
            ["git", "cat-file", "blob", object_id], cwd=ROOT
        )
    except subprocess.CalledProcessError as error:
        raise ReleaseEvidenceError("committed evidence could not be read") from error
    if len(raw) != size:
        raise ReleaseEvidenceError("committed evidence size changed")
    return raw


def committed_json(checker, revision: str, path: str, maximum: int) -> object:
    object_id = regular_blob_oid(revision, path)
    return checker.parse_json(read_committed_blob(object_id, maximum))


def validate_binding(source: str, paths: tuple[str, ...]) -> None:
    if not git_success("cat-file", "-e", f"{source}^{{commit}}"):
        raise ReleaseEvidenceError("qualified source commit was not found")
    if not git_success("merge-base", "--is-ancestor", source, "HEAD"):
        raise ReleaseEvidenceError("qualified source is not an ancestor of HEAD")
    if git_success("cat-file", "-e", f"{source}:{EVIDENCE_PATH.as_posix()}"):
        raise ReleaseEvidenceError("evidence must follow the qualified source commit")
    for path, expected_mode in REQUIRED_SOURCE_PATHS.items():
        try:
            regular_blob_oid(source, path, expected_mode)
        except ReleaseEvidenceError:
            raise ReleaseEvidenceError("qualified source lacks mutation controls")
    if set(paths) - ALLOWED_AFTER_SOURCE:
        raise ReleaseEvidenceError("qualified source changed after live mutation")
    if EVIDENCE_PATH.as_posix() not in paths:
        raise ReleaseEvidenceError("mutation evidence does not follow qualified source")
    try:
        regular_blob_oid("HEAD", EVIDENCE_PATH.as_posix())
    except ReleaseEvidenceError as error:
        raise ReleaseEvidenceError("mutation evidence is not a committed regular file") from error


def main() -> int:
    checker = load_checker()
    try:
        evidence = committed_json(
            checker,
            "HEAD",
            EVIDENCE_PATH.as_posix(),
            MAX_EVIDENCE_BYTES,
        )
        if not isinstance(evidence, dict):
            raise ReleaseEvidenceError("committed evidence root is invalid")
        source = evidence["source_commit"]
        if not isinstance(source, str) or not checker.HEX_COMMIT(source):
            raise ReleaseEvidenceError("committed source identity is invalid")
        validate_binding(source, changed_paths(source))
        policy = checker.validate_policy(
            committed_json(
                checker,
                source,
                "controlled-mutation-policy.json",
                MAX_POLICY_BYTES,
            )
        )
        checker.validate_evidence(evidence, policy)
    except (checker.EvidenceError, ReleaseEvidenceError, KeyError) as error:
        print(f"controlled mutation release: {error}", file=sys.stderr)
        return 1
    except (TypeError, ValueError, OverflowError):
        print(
            "controlled mutation release: evidence scalar type is invalid",
            file=sys.stderr,
        )
        return 1
    print("Controlled-mutation evidence is bound to unchanged qualified source.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
