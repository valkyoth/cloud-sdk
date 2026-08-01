#!/usr/bin/env python3
"""Require a versioned review row for every root Cargo.lock version change."""

from __future__ import annotations

import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


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
    changes: list[tuple[str, str, str]], review_text: str
) -> list[tuple[str, str, str]]:
    """Return lockfile changes absent from the review's exact table rows."""
    return [
        change
        for change in changes
        if f"| `{change[0]}` | `{change[1]}` | `{change[2]}` |" not in review_text
    ]


def previous_lock(base: str) -> str:
    """Read Cargo.lock from one local reviewed Git ref without invoking a shell."""
    result = subprocess.run(
        ["git", "show", f"{base}:Cargo.lock"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise ReviewError(f"dependency review: cannot read Cargo.lock from {base}")
    return result.stdout


def main(arguments: list[str]) -> int:
    if len(arguments) != 2:
        print(
            "usage: check_dependency_review.py <previous-tag> <review-document>",
            file=sys.stderr,
        )
        return 2
    base, review_name = arguments
    review_path = (ROOT / review_name).resolve()
    docs_root = (ROOT / "docs").resolve()
    if docs_root not in review_path.parents:
        print("dependency review: review document must be under docs/", file=sys.stderr)
        return 2
    try:
        current_text = (ROOT / "Cargo.lock").read_text(encoding="utf-8")
        review_text = review_path.read_text(encoding="utf-8")
        changes = version_changes(
            package_versions(previous_lock(base)), package_versions(current_text)
        )
        missing = missing_rows(changes, review_text)
    except (OSError, UnicodeError, ReviewError) as error:
        print(str(error), file=sys.stderr)
        return 1
    if missing:
        for name, old, new in missing:
            print(
                f"dependency review: missing Cargo.lock row {name} {old} -> {new}",
                file=sys.stderr,
            )
        return 1
    print(f"Dependency review inventories {len(changes)} root lockfile changes.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
