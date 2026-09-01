#!/usr/bin/env python3
"""Regression tests for bounded inline OpenAPI structural traversal."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cratesio_openapi_schema import MAX_TRAVERSAL_DEPTH, validate_schema_dialects
from cratesio_source_error import SourceLockError


def callback_chain(length: int) -> dict:
    path_item: dict = {
        "get": {"responses": {"200": {"description": "terminal response"}}}
    }
    for index in range(length):
        path_item = {
            "get": {
                "callbacks": {
                    f"callback-{index}": {"{{$request.body#/url}}": path_item}
                }
            }
        }
    return path_item


def callback_document(length: int) -> dict:
    return {"paths": {"/root": callback_chain(length)}}


def content_header_chain(length: int) -> dict:
    content: dict = {"application/json": {"schema": {"type": "string"}}}
    for index in range(length):
        content = {
            "application/json": {
                "encoding": {
                    f"field-{index}": {
                        "headers": {f"header-{index}": {"content": content}}
                    }
                }
            }
        }
    return {"components": {"responses": {"Nested": {"content": content}}}}


def mixed_reference_and_callback_document(
    reference_length: int, callback_length: int
) -> dict:
    targets = {
        f"target-{index}": {"$ref": f"#/x-targets/target-{index + 1}"}
        for index in range(reference_length)
    }
    targets[f"target-{reference_length}"] = callback_chain(callback_length)
    return {
        "x-targets": targets,
        "paths": {"/root": {"$ref": "#/x-targets/target-0"}},
    }


class StructuralDepthTests(unittest.TestCase):
    def test_exact_inline_callback_depth_limit_is_admitted(self) -> None:
        validate_schema_dialects(callback_document(MAX_TRAVERSAL_DEPTH))

    def test_inline_callback_limit_plus_one_fails_closed(self) -> None:
        with self.assertRaisesRegex(
            SourceLockError, "OpenAPI traversal depth exceeds reviewed limit"
        ):
            validate_schema_dialects(callback_document(MAX_TRAVERSAL_DEPTH + 1))

    def test_long_inline_callback_chain_returns_a_controlled_error(self) -> None:
        try:
            validate_schema_dialects(callback_document(500))
        except RecursionError as error:
            self.fail(f"inline callback traversal escaped its depth guard: {error}")
        except SourceLockError as error:
            self.assertEqual(
                str(error), "OpenAPI traversal depth exceeds reviewed limit"
            )
        else:
            self.fail("long inline callback traversal was admitted")

    def test_depth_is_restored_between_independent_inline_chains(self) -> None:
        validate_schema_dialects(
            {
                "paths": {
                    "/left": callback_chain(MAX_TRAVERSAL_DEPTH),
                    "/right": callback_chain(MAX_TRAVERSAL_DEPTH),
                }
            }
        )

    def test_reference_and_inline_depth_share_one_budget(self) -> None:
        with self.assertRaisesRegex(
            SourceLockError, "OpenAPI traversal depth exceeds reviewed limit"
        ):
            validate_schema_dialects(
                mixed_reference_and_callback_document(
                    MAX_TRAVERSAL_DEPTH - 1, 2
                )
            )

    def test_exact_content_header_depth_limit_is_admitted(self) -> None:
        validate_schema_dialects(content_header_chain(MAX_TRAVERSAL_DEPTH // 2))

    def test_content_header_limit_plus_one_fails_closed(self) -> None:
        with self.assertRaisesRegex(
            SourceLockError, "OpenAPI traversal depth exceeds reviewed limit"
        ):
            validate_schema_dialects(
                content_header_chain((MAX_TRAVERSAL_DEPTH // 2) + 1)
            )

    def test_long_content_header_chain_returns_a_controlled_error(self) -> None:
        try:
            validate_schema_dialects(content_header_chain(500))
        except RecursionError as error:
            self.fail(f"content/header traversal escaped its depth guard: {error}")
        except SourceLockError as error:
            self.assertEqual(
                str(error), "OpenAPI traversal depth exceeds reviewed limit"
            )
        else:
            self.fail("long content/header traversal was admitted")


if __name__ == "__main__":
    unittest.main()
