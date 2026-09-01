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
            "$ref": "customer-payload-reference",
            "$schema": "customer-payload-value",
        }
        schema = {
            "type": "object",
            "properties": {
                "$dynamicRef": {"type": "string"},
                "$ref": {"type": "string"},
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

    def test_payload_references_are_admitted_by_the_source_lock(self) -> None:
        operation_rows(
            source_lock_document(
                {"type": "object"}, {"$ref": "customer-payload-reference"}
            )
        )

    def test_schema_references_must_be_local_resolvable_strings(self) -> None:
        document = source_lock_document({"$ref": "#/components/schemas/Fixture"})
        document["components"]["schemas"] = {"Fixture": {"type": "object"}}
        operation_rows(document)

        for reference in (
            "https://attacker.invalid/schema",
            "#/components/schemas/Missing",
            None,
            1,
            {},
        ):
            document = source_lock_document({"$ref": reference})
            with self.subTest(reference=reference), self.assertRaises(SourceLockError):
                operation_rows(document)

    def test_schema_references_validate_targets_in_schema_context(self) -> None:
        document = {
            "components": {
                "schemas": {
                    "Entry": {"$ref": "#/components/examples/Hidden/value"}
                },
                "examples": {
                    "Hidden": {
                        "value": {"$ref": "https://attacker.invalid/schema"}
                    }
                },
            }
        }
        with self.assertRaises(SourceLockError):
            validate_schema_dialects(document)

    def test_json_pointers_support_arrays_percent_encoding_and_escapes(self) -> None:
        document = {
            "components": {
                "schemas": {
                    "Choice Name/Version~1": {
                        "oneOf": [{"type": "string"}, {"type": "integer"}]
                    },
                    "ArrayEntry": {
                        "$ref": "#/components/schemas/Choice%20Name~1Version~01/oneOf/0"
                    },
                    "ObjectEntry": {
                        "$ref": "#/components/schemas/Choice%20Name~1Version~01"
                    },
                }
            }
        }
        validate_schema_dialects(document)

    def test_malformed_json_pointer_boundaries_fail_closed(self) -> None:
        references = (
            "#/components/schemas/Choice/oneOf/01",
            "#/components/schemas/Choice/oneOf/-",
            "#/components/schemas/Choice/oneOf/1",
            "#/components/schemas/Choice/oneOf/%",
            "#/components/schemas/Choice/oneOf/%GG",
            "#/components/schemas/Choice/oneOf/%FF",
            "#/components/schemas/Choice~2",
        )
        for reference in references:
            document = {
                "components": {
                    "schemas": {
                        "Choice": {"oneOf": [{"type": "string"}]},
                        "Entry": {"$ref": reference},
                    }
                }
            }
            with self.subTest(reference=reference), self.assertRaises(SourceLockError):
                validate_schema_dialects(document)

    def test_local_reference_cycles_are_bounded(self) -> None:
        validate_schema_dialects(
            {
                "components": {
                    "schemas": {
                        "Left": {"$ref": "#/components/schemas/Right"},
                        "Right": {"$ref": "#/components/schemas/Left"},
                    },
                    "responses": {
                        "Left": {"$ref": "#/components/responses/Right"},
                        "Right": {"$ref": "#/components/responses/Left"},
                    },
                }
            }
        )

    def test_typed_local_reference_to_a_concrete_target_is_admitted(self) -> None:
        validate_schema_dialects(
            {
                "components": {
                    "responses": {
                        "Concrete": {
                            "description": "ok",
                            "content": {
                                "application/json": {"schema": {"type": "object"}}
                            },
                        },
                        "Alias": {"$ref": "#/components/responses/Concrete"},
                    }
                }
            }
        )

    def test_typed_local_references_validate_their_resolved_targets(self) -> None:
        external = {"$ref": "https://attacker.invalid/object"}
        references = (
            {"paths": {"/x": {"$ref": "#/x-hidden"}}},
            {
                "paths": {
                    "/x": {
                        "get": {
                            "parameters": [{"$ref": "#/x-hidden"}]
                        }
                    }
                }
            },
            {
                "paths": {
                    "/x": {
                        "post": {"requestBody": {"$ref": "#/x-hidden"}}
                    }
                }
            },
            {
                "paths": {
                    "/x": {
                        "get": {
                            "responses": {"200": {"$ref": "#/x-hidden"}}
                        }
                    }
                }
            },
            {"components": {"headers": {"Header": {"$ref": "#/x-hidden"}}}},
            {"components": {"callbacks": {"Call": {"$ref": "#/x-hidden"}}}},
            {"components": {"examples": {"Sample": {"$ref": "#/x-hidden"}}}},
            {"components": {"links": {"Next": {"$ref": "#/x-hidden"}}}},
            {
                "components": {
                    "securitySchemes": {"Auth": {"$ref": "#/x-hidden"}}
                }
            },
        )
        for index, document in enumerate(references):
            document["x-hidden"] = external
            with self.subTest(index=index), self.assertRaises(SourceLockError):
                validate_schema_dialects(document)

    def test_typed_reference_targets_must_be_objects(self) -> None:
        with self.assertRaises(SourceLockError):
            validate_schema_dialects(
                {
                    "x-hidden": "not-an-object",
                    "paths": {"/x": {"$ref": "#/x-hidden"}},
                }
            )

    def test_external_references_fail_at_reference_object_positions(self) -> None:
        reference = {"$ref": "https://attacker.invalid/object"}
        documents = (
            {"paths": {"/x": reference}},
            {"paths": {"/x": {"get": {"parameters": [reference]}}}},
            {"paths": {"/x": {"post": {"requestBody": reference}}}},
            {
                "paths": {
                    "/x": {"get": {"responses": {"200": reference}}}
                }
            },
            {"components": {"headers": {"Header": reference}}},
            {"components": {"callbacks": {"Callback": reference}}},
            {"components": {"examples": {"Example": reference}}},
            {"components": {"links": {"Link": reference}}},
            {"components": {"securitySchemes": {"Auth": reference}}},
            {
                "paths": {
                    "/x": {
                        "get": {
                            "responses": {
                                "200": {
                                    "links": {"next": reference},
                                    "content": {
                                        "application/json": {
                                            "examples": {"sample": reference}
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            },
        )
        for index, document in enumerate(documents):
            with self.subTest(index=index), self.assertRaises(SourceLockError):
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
