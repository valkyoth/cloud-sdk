#!/usr/bin/env python3
"""Regression tests for the API-matrix coverage gate."""

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check_api_matrix_coverage.py"
HEADER = """# Matrix

## Operations

| API | Group | Method | Path | Operation | Owner | Pagination | Sorting | Action | Deprecated | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
"""


def row(*, deprecated: str = "no", status: str = "implemented") -> str:
    """Build one fixture operation row."""
    return (
        "| cloud | Actions | GET | `/actions` | `get_actions` | "
        "`cloud_sdk_hetzner::actions` | no | no | action-list | "
        f"{deprecated} | {status} |\n"
    )


def run_check(
    contents: str, *, expected_counts: tuple[int, int, int] | None = None
) -> subprocess.CompletedProcess[str]:
    """Run the checker against a temporary matrix."""
    with tempfile.TemporaryDirectory() as directory:
        matrix = Path(directory) / "API_MATRIX.md"
        matrix.write_text(contents, encoding="utf-8")
        command = [str(CHECKER), str(matrix)]
        if expected_counts is not None:
            total, non_deprecated, deprecated = expected_counts
            command.extend(
                [
                    "--expected-total",
                    str(total),
                    "--expected-non-deprecated",
                    str(non_deprecated),
                    "--expected-deprecated",
                    str(deprecated),
                ]
            )
        return subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )


class ApiMatrixCoverageTests(unittest.TestCase):
    """Exercise complete, incomplete, deprecated, and malformed matrices."""

    def test_accepts_implemented_non_deprecated_and_deferred_deprecated_rows(self) -> None:
        result = run_check(
            HEADER
            + row(status="implemented-v0.26")
            + row(deprecated="yes", status="deferred-deprecated")
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("1 non-deprecated operations implemented", result.stdout)

    def test_rejects_planned_non_deprecated_rows(self) -> None:
        result = run_check(HEADER + row(status="planned"))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("non-deprecated operations remain incomplete", result.stderr)
        self.assertIn("is planned", result.stdout)

    def test_rejects_deferred_non_deprecated_rows(self) -> None:
        result = run_check(HEADER + row(status="deferred"))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("is deferred", result.stdout)

        near_match = run_check(HEADER + row(status="implemented-later"))
        self.assertNotEqual(near_match.returncode, 0)
        self.assertIn("is implemented-later", near_match.stdout)

    def test_rejects_missing_and_changed_tables(self) -> None:
        missing = run_check("# Matrix\n")
        self.assertNotEqual(missing.returncode, 0)
        self.assertIn("missing Operations section", missing.stderr)

        changed = run_check(HEADER.replace("| Status |", "| State |") + row())
        self.assertNotEqual(changed.returncode, 0)
        self.assertIn("columns changed unexpectedly", changed.stderr)

    def test_rejects_changed_source_locked_counts(self) -> None:
        matrix = HEADER + row() + row(deprecated="yes", status="deferred-deprecated")
        accepted = run_check(matrix, expected_counts=(2, 1, 1))
        self.assertEqual(accepted.returncode, 0, accepted.stderr)

        rejected = run_check(matrix, expected_counts=(3, 2, 1))
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("operation counts changed unexpectedly", rejected.stderr)

    def test_rejects_oversized_matrix_before_parsing(self) -> None:
        result = run_check("x" * ((2 * 1024 * 1024) + 1))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("matrix exceeds the local size limit", result.stderr)


if __name__ == "__main__":
    unittest.main()
