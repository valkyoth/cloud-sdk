#!/usr/bin/env python3
"""Regression tests for bounded typed OpenAPI reference traversal."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cratesio_openapi_schema import MAX_TRAVERSAL_DEPTH, validate_schema_dialects
from cratesio_source_error import SourceLockError


def response_components(prefix: str, length: int) -> dict:
    responses = {
        f"{prefix}{index}": {
            "$ref": f"#/components/responses/{prefix}{index + 1}"
        }
        for index in range(length)
    }
    responses[f"{prefix}{length}"] = {"description": "terminal response"}
    return responses


def response_chain(length: int) -> dict:
    responses = response_components("Response", length)
    return {"components": {"responses": responses}}


class ReferenceDepthTests(unittest.TestCase):
    def test_exact_reference_depth_limit_is_admitted(self) -> None:
        validate_schema_dialects(response_chain(MAX_TRAVERSAL_DEPTH))

    def test_reference_depth_limit_plus_one_fails_closed(self) -> None:
        with self.assertRaisesRegex(
            SourceLockError, "OpenAPI traversal depth exceeds reviewed limit"
        ):
            validate_schema_dialects(response_chain(MAX_TRAVERSAL_DEPTH + 1))

    def test_reference_depth_is_restored_between_independent_chains(self) -> None:
        responses = response_components("Left", MAX_TRAVERSAL_DEPTH)
        responses.update(response_components("Right", MAX_TRAVERSAL_DEPTH))
        validate_schema_dialects({"components": {"responses": responses}})

    def test_long_acyclic_chain_returns_a_controlled_error(self) -> None:
        with self.assertRaisesRegex(
            SourceLockError, "OpenAPI traversal depth exceeds reviewed limit"
        ):
            validate_schema_dialects(response_chain(1_500))

    def test_reference_cycle_remains_bounded(self) -> None:
        validate_schema_dialects(
            {
                "components": {
                    "responses": {
                        "Left": {"$ref": "#/components/responses/Right"},
                        "Right": {"$ref": "#/components/responses/Left"},
                    }
                }
            }
        )


if __name__ == "__main__":
    unittest.main()
