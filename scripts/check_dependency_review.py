#!/usr/bin/env python3
"""Require lockfile-scoped review rows for every repository dependency graph."""

from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCKFILES = (
    "Cargo.lock",
    "fuzz/Cargo.lock",
    "tests/reqwest-feature-unification/Cargo.lock",
    "tools/prepared-coverage-check/Cargo.lock",
)


class ReviewError(Exception):
    """A dependency-review input or inventory is incomplete."""


def package_versions(lock_text: str) -> dict[str, set[str]]:
    """Return all locked versions grouped by package name."""
    try:
        document = tomllib.loads(lock_text)
    except tomllib.TOMLDecodeError as error:
        raise ReviewError("dependency review: Cargo.lock is invalid TOML") from error
    result: dict[str, set[str]] = {}
    for package in document.get("package", []):
        name = package.get("name")
        version = package.get("version")
        if not isinstance(name, str) or not isinstance(version, str):
            raise ReviewError("dependency review: Cargo.lock package is incomplete")
        result.setdefault(name, set()).add(version)
    return result


def version_changes(
    previous: dict[str, set[str]], current: dict[str, set[str]]
) -> list[tuple[str, str, str]]:
    """Pair removed and added versions into deterministic review rows."""
    changes: list[tuple[str, str, str]] = []
    for name in sorted(previous.keys() | current.keys()):
        removed = sorted(previous.get(name, set()) - current.get(name, set()))
        added = sorted(current.get(name, set()) - previous.get(name, set()))
        width = max(len(removed), len(added))
        for index in range(width):
            old = removed[index] if index < len(removed) else "-"
            new = added[index] if index < len(added) else "-"
            changes.append((name, old, new))
    return changes


def missing_rows(
    changes: list[tuple[str, str, str]], review_text: str, lock_name: str = "Cargo.lock"
) -> list[tuple[str, str, str]]:
    """Return lockfile changes absent from the review's exact table rows."""
    return [
        change
        for change in changes
        if not any(line.startswith(
            f"| `{lock_name}` | `{change[0]}` | `{change[1]}` | `{change[2]}` |"
        ) for line in review_text.splitlines())
    ]


def review_section(review_text: str, version: str) -> str:
    """Return only the dependency evidence for one release version."""
    if re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version) is None:
        raise ReviewError("dependency review: release version is invalid")
    marker = f"## v{version}\n"
    _, found, remainder = review_text.partition(marker)
    if not found:
        raise ReviewError(f"dependency review: missing section v{version}")
    return remainder.split("\n## ", 1)[0]


def previous_lock(base: str, lock_name: str) -> str:
    """Read Cargo.lock from one local reviewed Git ref without invoking a shell."""
    result = subprocess.run(
        ["git", "show", f"{base}:{lock_name}"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise ReviewError(f"dependency review: cannot read {lock_name} from {base}")
    return result.stdout


def main(arguments: list[str]) -> int:
    if len(arguments) != 3:
        print(
            "usage: check_dependency_review.py "
            "<previous-tag> <release-version> <review-document>",
            file=sys.stderr,
        )
        return 2
    base, release_version, review_name = arguments
    review_path = (ROOT / review_name).resolve()
    docs_root = (ROOT / "docs").resolve()
    if docs_root not in review_path.parents:
        print("dependency review: review document must be under docs/", file=sys.stderr)
        return 2
    try:
        review_text = review_section(
            review_path.read_text(encoding="utf-8"), release_version
        )
        missing = []
        count = 0
        for lock_name in LOCKFILES:
            current_text = (ROOT / lock_name).read_text(encoding="utf-8")
            changes = version_changes(
                package_versions(previous_lock(base, lock_name)), package_versions(current_text)
            )
            count += len(changes)
            missing.extend((lock_name, *row) for row in missing_rows(changes, review_text, lock_name))
    except (OSError, UnicodeError, ReviewError) as error:
        print(str(error), file=sys.stderr)
        return 1
    if missing:
        for lock_name, name, old, new in missing:
            print(
                f"dependency review: missing {lock_name} row {name} {old} -> {new}",
                file=sys.stderr,
            )
        return 1
    print(f"Dependency review inventories {count} changes across {len(LOCKFILES)} lockfiles.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
