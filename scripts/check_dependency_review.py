#!/usr/bin/env python3
"""Require lockfile-scoped review rows for every repository dependency graph."""

from __future__ import annotations

import re
import hashlib
import json
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


def package_identities(lock_text: str) -> dict[str, str]:
    """Hash complete records per name, including every version and source."""
    try:
        document = tomllib.loads(lock_text)
    except tomllib.TOMLDecodeError as error:
        raise ReviewError("dependency review: Cargo.lock is invalid TOML") from error
    packages = document.get("package")
    if not isinstance(packages, list) or not packages:
        raise ReviewError("dependency review: missing package inventory")
    result = {}
    seen = set()
    for package in packages:
        if not isinstance(package, dict):
            raise ReviewError("dependency review: invalid package record")
        name = package.get("name")
        version = package.get("version")
        source = package.get("source", "path")
        if not all(isinstance(value, str) and value for value in (name, version, source)):
            raise ReviewError("dependency review: Cargo.lock package is incomplete")
        key = (name, version, source)
        if key in seen:
            raise ReviewError("dependency review: duplicate package identity")
        seen.add(key)
        try:
            canonical = json.dumps(package, sort_keys=True, separators=(",", ":"))
        except (TypeError, ValueError) as error:
            raise ReviewError("dependency review: unsupported package value") from error
        result.setdefault(name, []).append(canonical)
    return {
        name: hashlib.sha256(json.dumps(sorted(records), separators=(",", ":")).encode()).hexdigest()
        for name, records in result.items()
    }


def identity_changes(
    previous: dict[str, str], current: dict[str, str]
) -> list[tuple[str, str, str]]:
    """Require fresh evidence for any complete package-record change."""
    return [(name, previous.get(name, "-"), current.get(name, "-"))
            for name in sorted(previous.keys() | current.keys())
            if previous.get(name) != current.get(name)]


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
            changes = identity_changes(
                package_identities(previous_lock(base, lock_name)), package_identities(current_text)
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
