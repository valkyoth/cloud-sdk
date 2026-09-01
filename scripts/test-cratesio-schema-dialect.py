#!/usr/bin/env python3
"""Regression tests for crates.io OpenAPI schema-dialect admission."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cratesio_openapi_schema import OAS_31_DIALECT, validate_schema_dialects
from cratesio_source_error import SourceLockError


class SchemaDialectTests(unittest.TestCase):
    def test_absent_and_exact_root_dialects_are_admitted(self) -> None:
        validate_schema_dialects({})
        validate_schema_dialects({"jsonSchemaDialect": OAS_31_DIALECT})

    def test_custom_and_non_string_root_dialects_are_rejected(self) -> None:
        for value in ("https://attacker.invalid/dialect", None, 1, {}):
            with self.subTest(value=value), self.assertRaises(SourceLockError):
                validate_schema_dialects({"jsonSchemaDialect": value})

    def test_exact_nested_schema_dialect_is_admitted(self) -> None:
        validate_schema_dialects(
            {"components": {"schemas": [{"$schema": OAS_31_DIALECT}]}}
        )

    def test_nested_schema_dialect_overrides_are_rejected(self) -> None:
        for value in ("https://attacker.invalid/dialect", None, 1, {}):
            document = {"components": {"schemas": [{"$schema": value}]}}
            with self.subTest(value=value), self.assertRaises(SourceLockError):
                validate_schema_dialects(document)


if __name__ == "__main__":
    result = unittest.main(exit=False)
    if result.result.wasSuccessful():
        print(f"{result.result.testsRun} crates.io schema-dialect tests passed.")
    raise SystemExit(not result.result.wasSuccessful())
