#!/usr/bin/env python3
"""Regression tests for the dependency-review lockfile inventory."""

from __future__ import annotations

import importlib.util
from pathlib import Path

SCRIPT = Path(__file__).with_name("check_dependency_review.py")
SPEC = importlib.util.spec_from_file_location("dependency_review", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def lock(packages: list[tuple[str, str]]) -> str:
    body = ['version = 4', '']
    for name, version in packages:
        body.extend(
            [
                '[[package]]',
                f'name = "{name}"',
                f'version = "{version}"',
                '',
            ]
        )
    return "\n".join(body)


def test_change_inventory() -> None:
    previous = MODULE.package_versions(
        lock([("changed", "1.0.0"), ("removed", "1.0.0"), ("multi", "1.0.0")])
    )
    current = MODULE.package_versions(
        lock([("changed", "1.1.0"), ("added", "1.0.0"), ("multi", "2.0.0")])
    )
    assert MODULE.version_changes(previous, current) == [
        ("added", "-", "1.0.0"),
        ("changed", "1.0.0", "1.1.0"),
        ("multi", "1.0.0", "2.0.0"),
        ("removed", "1.0.0", "-"),
    ]


def test_exact_review_rows() -> None:
    changes = [("one", "1.0.0", "2.0.0"), ("two", "-", "1.0.0")]
    review = "\n".join(
        [
            "| Package | Previous | Current | Review |",
            "| --- | --- | --- | --- |",
            "| `one` | `1.0.0` | `2.0.0` | reviewed |",
        ]
    )
    assert MODULE.missing_rows(changes, review) == [("two", "-", "1.0.0")]
    review += "\n| `two` | `-` | `1.0.0` | reviewed |"
    assert MODULE.missing_rows(changes, review) == []


def main() -> None:
    test_change_inventory()
    test_exact_review_rows()
    print("2 dependency-review regression groups passed.")


if __name__ == "__main__":
    main()
