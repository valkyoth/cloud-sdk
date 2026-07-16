#!/usr/bin/env python3
"""Regression tests for the checked-response operation coverage gate."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "scripts" / "check_response_operation_coverage.py"
SPEC = importlib.util.spec_from_file_location("response_coverage", CHECKER)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load response coverage checker")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def row(operation: str = "`get_actions`", status: str = "implemented") -> str:
    return (
        "| cloud | Actions | GET | `/actions` | "
        f"{operation} | `cloud_sdk_hetzner::actions` | no | no | "
        f"action-list | no | {status} |\n"
    )


class ResponseOperationCoverageTests(unittest.TestCase):
    def test_unwraps_canonical_markdown_operation_identifiers(self) -> None:
        self.assertEqual(MODULE.active_operations(row()), {"get_actions"})

    def test_rejects_noncanonical_operation_cells(self) -> None:
        with self.assertRaisesRegex(ValueError, "not canonical inline code"):
            MODULE.active_operations(row("get_actions"))

    def test_rejects_duplicate_and_incomplete_active_operations(self) -> None:
        with self.assertRaisesRegex(ValueError, "duplicate active operation"):
            MODULE.active_operations(row() + row())
        with self.assertRaisesRegex(ValueError, "active operation is not implemented"):
            MODULE.active_operations(row(status="planned"))


if __name__ == "__main__":
    unittest.main()
