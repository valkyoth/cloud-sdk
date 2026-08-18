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


def test_review_evidence_is_scoped_to_the_requested_release() -> None:
    historical_row = "| `changed` | `1.0.0` | `2.0.0` | reviewed |"
    review = "\n".join(
        [
            "## v0.95.0",
            "",
            historical_row,
            "",
            "## v0.96.0",
            "",
            "No dependency changes.",
        ]
    )
    current = MODULE.review_section(review, "0.96.0")
    assert historical_row not in current
    assert MODULE.missing_rows([("changed", "1.0.0", "2.0.0")], current) == [
        ("changed", "1.0.0", "2.0.0")
    ]


def test_missing_or_invalid_release_sections_fail_closed() -> None:
    for version in ("0.97.0", "0.96.0\n## v0.95.0"):
        try:
            MODULE.review_section("## v0.96.0\n\nreviewed\n", version)
        except MODULE.ReviewError:
            pass
        else:
            raise AssertionError("invalid dependency-review section was accepted")


def main() -> None:
    test_change_inventory()
    test_exact_review_rows()
    test_review_evidence_is_scoped_to_the_requested_release()
    test_missing_or_invalid_release_sections_fail_closed()
    print("4 dependency-review regression groups passed.")


if __name__ == "__main__":
    main()
