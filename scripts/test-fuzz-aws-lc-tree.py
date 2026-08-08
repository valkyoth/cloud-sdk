#!/usr/bin/env python3
"""Regression tests for fuzz AWS-LC dependency-tree normalization."""

from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check-fuzz-aws-lc-tree.py"


def run(tree: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECKER)],
        cwd=ROOT,
        input=tree,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def test_colored_repeated_packages_are_one_exact_graph() -> None:
    result = run(
        "\x1b[1;32maws-lc-rs v1.17.3\x1b[0m\n"
        "aws-lc-rs v1.17.3 (*)\n"
        "aws-lc-sys v0.43.0\n"
        "\x1b[36maws-lc-sys v0.43.0 (*)\x1b[0m\n"
    )
    assert result.returncode == 0, result


def test_changed_or_additional_versions_fail_closed() -> None:
    result = run(
        "aws-lc-rs v1.17.3\n"
        "aws-lc-rs v1.18.0\n"
        "aws-lc-sys v0.43.0\n"
    )
    assert result.returncode == 1, result
    assert "aws-lc-rs v1.18.0" in result.stderr


def test_missing_packages_fail_closed() -> None:
    result = run("aws-lc-rs v1.17.3\n")
    assert result.returncode == 1, result


def main() -> None:
    tests = (
        test_colored_repeated_packages_are_one_exact_graph,
        test_changed_or_additional_versions_fail_closed,
        test_missing_packages_fail_closed,
    )
    for test in tests:
        test()
    print(f"{len(tests)} fuzz AWS-LC tree regression tests passed.")


if __name__ == "__main__":
    main()
