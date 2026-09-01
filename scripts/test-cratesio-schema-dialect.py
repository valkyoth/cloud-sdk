#!/usr/bin/env python3
"""Regression tests for context-aware OpenAPI schema-dialect admission."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cratesio_openapi_schema import (
    OAS_31_DIALECT,
    validate_schema_dialects,
    validate_schema_tree,
)
from cratesio_source_error import SourceLockError
from cratesio_source_lock import operation_rows


BAD_DIALECT = "https://attacker.invalid/dialect"


def custom_schema() -> dict:
    return {"$schema": BAD_DIALECT}


def custom_content() -> dict:
    return {"application/json": {"schema": custom_schema()}}


def source_lock_document(schema: dict, example: dict | None = None) -> dict:
    media = {"schema": schema}
    if example is not None:
        media["example"] = example
    return {
        "openapi": "3.1.0",
        "paths": {
            "/api/v1/fixture": {
                "get": {
                    "operationId": "fixture",
                    "responses": {
                        "200": {
                            "description": "ok",
                            "content": {"application/json": media},
                        }
                    },
                }
            }
        },
        "components": {
            "securitySchemes": {
                "api_token": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "authorization",
                },
                "cookie": {
                    "type": "apiKey",
                    "in": "cookie",
                    "name": "cargo_session",
                },
                "trustpub_token": {"type": "http", "scheme": "bearer"},
            }
        },
    }


class SchemaDialectTests(unittest.TestCase):
    def test_absent_and_exact_root_dialects_are_admitted(self) -> None:
        validate_schema_dialects({})
        validate_schema_dialects({"jsonSchemaDialect": OAS_31_DIALECT})

    def test_custom_and_non_string_root_dialects_are_rejected(self) -> None:
        for value in (BAD_DIALECT, None, 1, {}):
            with self.subTest(value=value), self.assertRaises(SourceLockError):
                validate_schema_dialects({"jsonSchemaDialect": value})

    def test_custom_dialects_fail_at_known_schema_positions(self) -> None:
        documents = (
            {"components": {"schemas": {"Model": custom_schema()}}},
            {
                "components": {
                    "schemas": {
                        "Model": {
                            "properties": {"value": {"$schema": BAD_DIALECT}}
                        }
                    }
                }
            },
            {
                "paths": {
                    "/x": {
                        "get": {
                            "parameters": [{"schema": custom_schema()}]
                        }
                    }
                }
            },
            {
                "paths": {
                    "/x": {
                        "get": {
                            "responses": {
                                "200": {
                                    "content": custom_content()
                                }
                            }
                        }
                    }
                }
            },
            {"components": {"headers": {"x": {"schema": custom_schema()}}}},
            {
                "components": {
                    "requestBodies": {"Body": {"content": custom_content()}}
                }
            },
            {
                "components": {
                    "responses": {"Reply": {"headers": {"x": {"schema": custom_schema()}}}}
                }
            },
            {
                "components": {
                    "pathItems": {"Item": {"parameters": [{"schema": custom_schema()}]}}
                }
            },
            {
                "paths": {
                    "/x": {"post": {"requestBody": {"content": custom_content()}}}
                }
            },
            {
                "paths": {
                    "/x": {
                        "get": {
                            "responses": {
                                "200": {"headers": {"x": {"schema": custom_schema()}}}
                            }
                        }
                    }
                }
            },
            {
                "paths": {
                    "/x": {
                        "get": {
                            "responses": {
                                "200": {
                                    "content": {
                                        "application/json": {
                                            "encoding": {
                                                "field": {
                                                    "headers": {
                                                        "x": {"schema": custom_schema()}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            {
                "paths": {
                    "/x": {
                        "get": {
                            "callbacks": {
                                "done": {
                                    "{$request.body#/url}": {
                                        "parameters": [{"schema": custom_schema()}]
                                    }
                                }
                            }
                        }
                    }
                }
            },
            {"webhooks": {"event": {"parameters": [{"schema": custom_schema()}]}}},
        )
        for index, document in enumerate(documents):
            with self.subTest(index=index), self.assertRaises(SourceLockError):
                validate_schema_dialects(document)

    def test_instance_data_and_schema_property_names_are_not_controls(self) -> None:
        payload = {
            "$dynamicRef": "customer-payload-reference",
            "$schema": "customer-payload-value",
        }
        schema = {
            "type": "object",
            "properties": {
                "$dynamicRef": {"type": "string"},
                "$schema": {"type": "string"},
            },
            "example": payload,
            "default": payload,
            "const": payload,
            "enum": [payload],
        }
        document = {
            "components": {
                "examples": {"Payload": {"value": payload}},
                "schemas": {"Model": schema},
            },
            "paths": {
                "/x": {
                    "get": {
                        "responses": {
                            "200": {
                                "content": {
                                    "application/json": {
                                        "schema": schema,
                                        "example": payload,
                                        "examples": {"sample": {"value": payload}},
                                    }
                                }
                            }
                        }
                    }
                }
            },
        }
        validate_schema_dialects(document)

    def test_dynamic_references_are_rejected_only_in_schema_objects(self) -> None:
        for value in ("https://attacker.invalid/schema", "#node", None, 1, {}):
            with self.subTest(value=value), self.assertRaises(SourceLockError):
                validate_schema_tree({"$dynamicRef": value})
        with self.assertRaises(SourceLockError):
            operation_rows(
                source_lock_document({"$dynamicRef": "https://attacker.invalid/schema"})
            )
        operation_rows(
            source_lock_document(
                {"type": "object"}, {"$dynamicRef": "customer-payload-reference"}
            )
        )

    def test_boolean_and_exact_nested_schemas_are_admitted(self) -> None:
        validate_schema_tree(True)
        validate_schema_tree(False)
        validate_schema_tree({"properties": {"value": {"$schema": OAS_31_DIALECT}}})

    def test_malformed_recursive_schema_containers_are_rejected(self) -> None:
        for schema in ({"allOf": {}}, {"properties": []}, {"items": "invalid"}):
            with self.subTest(schema=schema), self.assertRaises(SourceLockError):
                validate_schema_tree(schema)


if __name__ == "__main__":
    result = unittest.main(exit=False)
    if result.result.wasSuccessful():
        print(f"{result.result.testsRun} crates.io schema-dialect tests passed.")
    raise SystemExit(not result.result.wasSuccessful())
