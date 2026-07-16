#!/usr/bin/env python3
"""Regression tests for checked-response lock generation."""

from __future__ import annotations

import importlib.util
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts" / "generate_response_operations.py"
SPEC = importlib.util.spec_from_file_location("response_generator", GENERATOR)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load response operation generator")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def specification(operation_id: str) -> dict[str, object]:
    operation = {
        "operationId": operation_id,
        "responses": {"204": {"description": "empty"}},
    }
    return {"paths": {"/test": {"delete": operation}}}


class ResponseOperationGeneratorTests(unittest.TestCase):
    def test_rejects_tsv_unsafe_operation_identifiers(self) -> None:
        for operation_id in ["bad\tid", "bad\nid", "bad\rid", "bad\0id"]:
            with self.assertRaisesRegex(ValueError, "TSV-unsafe"):
                MODULE.rows("cloud", specification(operation_id))

    def test_rejects_tsv_unsafe_generated_fields(self) -> None:
        rows = [
            ("cloud", f"operation-{index}", "204", "empty", "-", "-")
            for index in range(MODULE.EXPECTED_ACTIVE)
        ]
        rows[0] = ("cloud", "operation-0", "204", "empty", "bad\troot", "-")
        with self.assertRaisesRegex(ValueError, "TSV-unsafe"):
            MODULE.render(rows)


if __name__ == "__main__":
    unittest.main()
